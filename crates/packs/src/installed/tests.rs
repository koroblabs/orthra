use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use expect_test::expect;

use super::Install;
use crate::Packs;

/// A directory of the test's own to install packs in: never the person's.
fn packs_dir() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "orthra-packs-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

#[test]
fn uk_ua_is_installed_from_dict_uk_and_english_is_built_in() {
    let uk = Install::of("uk-UA").expect("uk-UA is installed").unwrap();
    let sources: Vec<String> = uk
        .sources
        .iter()
        .map(|source| {
            format!(
                "{}\n  sha256 {}\n  {}\n  {:?}",
                source.url, source.sha256, source.license, source.take
            )
        })
        .collect();
    expect![[r#"
        uk-UA 6.8.6.2, compiled from uk_UA to uk_UA.fst
        https://github.com/brown-uk/dict_uk/releases/download/v6.8.6/hunspell-uk_UA_6.8.6.zip
          sha256 043ae50d3a30beda0d4aa67562fdbee576e2f862eb6120357c1e5ae5ed3d5e9e
          GPL-3.0, https://github.com/brown-uk/dict_uk
          Archive([("uk_UA.aff", "uk_UA.aff"), ("uk_UA.dic", "uk_UA.dic")])
        https://raw.githubusercontent.com/hermitdave/FrequencyWords/525f9b560de45753a5ea01069454e72e9aa541c6/content/2018/uk/uk_50k.txt
          sha256 881686fe66a168b79f6a9ee2f6cbbf944cfb12bfb7040913ef535cb29bd86ec9
          CC-BY-SA-4.0, https://github.com/hermitdave/FrequencyWords (from OpenSubtitles 2018)
          File("uk_50k.txt")"#]]
    .assert_eq(&format!(
        "{} {}, compiled from {} to {}\n{}",
        uk.tag,
        uk.version,
        uk.compiled.as_deref().unwrap_or("-"),
        uk.compiled_file().unwrap_or_default(),
        sources.join("\n")
    ));
    assert_eq!(Install::tags(), ["uk-UA"]);
    assert!(Install::of("en-US").is_none(), "en-US is built in");
}

#[test]
fn a_language_not_installed_is_missing_and_not_an_error() {
    let dir = packs_dir();
    let (packs, missing) = Packs::load(&["en-US", "uk-UA"], Some(&dir)).unwrap();
    assert_eq!(
        packs.iter().map(|pack| pack.tag()).collect::<Vec<_>>(),
        ["en-US"]
    );
    assert_eq!(missing, ["uk-UA"]);
    let (_, missing) = Packs::load(&["uk-UA"], None).unwrap();
    assert_eq!(
        missing,
        ["uk-UA"],
        "nowhere to look is not installed either"
    );
    assert!(
        Packs::load(&["xx-XX"], Some(&dir)).is_err(),
        "a language with no pack at all is still an error"
    );
}

#[test]
fn an_installed_pack_is_read_from_its_directory_on_its_first_word() {
    let dir = packs_dir();
    let uk = Install::of("uk-UA").unwrap().unwrap();
    let installed = uk.dir(&dir);
    fs::create_dir_all(&installed).unwrap();
    let (aff, dic) = ("SET UTF-8\n", "2\nпривіт\nм'ята\n");
    fs::write(installed.join("uk_UA.aff"), aff).unwrap();
    fs::write(installed.join("uk_UA.dic"), dic).unwrap();
    fs::write(installed.join("uk_50k.txt"), "привіт 10\n").unwrap();
    assert!(
        !uk.is_installed(&dir),
        "a pack whose dictionary is not compiled is not installed"
    );
    fs::write(
        installed.join("uk_UA.fst"),
        super::compile(aff, dic).unwrap().forms,
    )
    .unwrap();
    assert!(uk.is_installed(&dir));

    let (packs, missing) = Packs::load(&["uk-UA"], Some(&dir)).unwrap();
    assert!(missing.is_empty());
    let pack = packs.iter().next().unwrap();
    assert!(!pack.is_loaded(), "loading the pack reads no dictionary");
    let answers: Vec<String> = ["привіт", "привт", "мʼята", "м’ята"]
        .iter()
        .map(|word| format!("{word} {}", pack.accepts(word)))
        .collect();
    expect!["привіт true, привт false, мʼята true, м’ята true"].assert_eq(&answers.join(", "));
    assert!(pack.is_loaded());
    fs::remove_dir_all(&dir).unwrap();
}

/// Each format a compiled dictionary has had, and the version of each compiled pack built in it.
/// A pack keeps its version's directory, so a new format with the same version would be read from
/// a file of the old one, refused, and not installed again: a new format needs a new version of
/// every compiled pack, written here.
const COMPILED_IN: &[(u64, &str, &str)] = &[(1, "uk-UA", "6.8.6.2")];

#[test]
fn a_new_compiled_format_comes_with_a_new_version_of_every_compiled_pack() {
    let format = orthra_dict::compiled::FORMAT;
    for tag in Install::tags() {
        let install = Install::of(tag).unwrap().unwrap();
        if install.compiled.is_none() {
            continue;
        }
        assert!(
            COMPILED_IN.contains(&(format, tag, install.version.as_str())),
            "{tag} {} is not listed for format {format} in COMPILED_IN",
            install.version
        );
        let older = COMPILED_IN
            .iter()
            .filter(|(old, pack, _)| *old != format && *pack == tag);
        for (old, _, version) in older {
            assert_ne!(
                *version, install.version,
                "{tag} {version} was compiled in format {old} too: give it a new version"
            );
        }
    }
}
