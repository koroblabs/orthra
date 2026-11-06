use std::{
    fs,
    io::{Cursor, Write},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use orthra_packs::{
    Packs,
    installed::{Install, Source, Take},
};
use sha2::{Digest, Sha256};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use super::{InstallError, definition, hex, install_with};

/// A directory of the test's own to install into: never the person's.
fn packs_dir() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "orthra-install-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn zip_of(files: &[(&str, &str)]) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, text) in files {
        zip.start_file(*name, options).unwrap();
        zip.write_all(text.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// uk-UA's definition, each source pinned to the digest of the test's own bytes for it: the
/// dictionary's archive from `files`, and a frequency list.
fn pinned(files: &[(&str, &str)]) -> (Install, Vec<Vec<u8>>) {
    let downloads = vec![zip_of(files), "привіт 10\n".as_bytes().to_vec()];
    let mut install = definition("uk-UA").unwrap();
    for (source, bytes) in install.sources.iter_mut().zip(&downloads) {
        source.sha256 = hex(&Sha256::digest(bytes));
    }
    (install, downloads)
}

/// Hands out each source's bytes, in the order the sources are asked for.
fn from(downloads: Vec<Vec<u8>>) -> impl FnMut(&Source) -> Result<Vec<u8>, InstallError> {
    let mut downloads = downloads.into_iter();
    move |_| Ok(downloads.next().expect("a download per source"))
}

const AFF: &str = "SET UTF-8\n";
const DIC: &str = "2\nпривіт\nм'ята\n";

#[test]
fn every_source_that_hashes_as_pinned_is_installed_and_read() {
    let packs = packs_dir();
    let (install, downloads) = pinned(&[("uk_UA.aff", AFF), ("uk_UA.dic", DIC), ("README", "x")]);
    assert!(matches!(install.sources[1].take, Take::File(_)));
    let dir = install_with(&install, from(downloads), &packs).unwrap();
    assert_eq!(dir, install.dir(&packs));
    let mut written: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    written.sort();
    assert_eq!(
        written,
        ["uk_50k.txt", "uk_UA.aff", "uk_UA.dic", "uk_UA.fst"],
        "only the pack's files, and the dictionary they compile to"
    );
    let (loaded, missing) = Packs::load(&["uk-UA"], Some(&packs)).unwrap();
    assert!(missing.is_empty());
    assert!(loaded.iter().next().unwrap().accepts("мʼята"));
    fs::remove_dir_all(&packs).unwrap();
}

#[test]
fn a_source_that_does_not_hash_as_pinned_is_refused_and_nothing_is_left() {
    let packs = packs_dir();
    let (mut install, downloads) = pinned(&[("uk_UA.aff", AFF), ("uk_UA.dic", DIC)]);
    install.sources[1].sha256 = "0".repeat(64);
    let err = install_with(&install, from(downloads), &packs).unwrap_err();
    assert!(err.0.contains("uk_50k.txt has SHA-256 "), "{err}");
    let left: Vec<_> = fs::read_dir(packs.join("uk-UA")).unwrap().collect();
    assert!(left.is_empty(), "no partial directory: {left:?}");
    fs::remove_dir_all(&packs).unwrap();
}

#[test]
fn an_archive_without_a_file_the_pack_takes_leaves_nothing_behind() {
    let packs = packs_dir();
    let (install, downloads) = pinned(&[("uk_UA.aff", AFF)]);
    let err = install_with(&install, from(downloads), &packs).unwrap_err();
    assert!(
        err.0.starts_with("uk-UA: the archive has no uk_UA.dic"),
        "{err}"
    );
    let left: Vec<_> = fs::read_dir(packs.join("uk-UA")).unwrap().collect();
    assert!(left.is_empty(), "no partial directory: {left:?}");
    fs::remove_dir_all(&packs).unwrap();
}

#[test]
fn a_dictionary_that_does_not_parse_is_not_left_installed() {
    let packs = packs_dir();
    let (install, downloads) =
        pinned(&[("uk_UA.aff", "FLAG long\nSFX AB Y 1\n"), ("uk_UA.dic", DIC)]);
    let err = install_with(&install, from(downloads), &packs).unwrap_err();
    expect_test::expect!["uk-UA: uk_UA does not compile: .aff line 2: SFX has too few rows"]
        .assert_eq(&err.0);
    assert!(!install.is_installed(&packs));
    let left: Vec<_> = fs::read_dir(packs.join("uk-UA")).unwrap().collect();
    assert!(left.is_empty(), "no partial directory: {left:?}");
    let _ = fs::remove_dir_all(&packs);
}
