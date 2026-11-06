//! Installing a pack that orthra does not carry: its archive, checked against the SHA-256 its
//! `install.toml` pins, unpacked where installed packs are read from, its dictionary compiled if
//! its `pack.toml` says so, and read once before it is put there.
//!
//! The only crate that knows an archive format or a hash. What it is handed is a pack's tag and
//! the archive's bytes; it never sees a document, so nothing a person wrote can leave through it.

use std::{
    fmt, fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    time::Duration,
};

use orthra_packs::Packs;
pub use orthra_packs::installed::{Install, Source, Take, compile};
use sha2::{Digest, Sha256};

pub mod models;
#[cfg(test)]
mod tests;

/// An install that did not happen, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallError(pub String);

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for InstallError {}

/// What `install.toml` pins for pack `tag`: an error for a pack that is built in, or none at all.
pub fn definition(tag: &str) -> Result<Install, InstallError> {
    match Install::of(tag) {
        Some(install) => install.map_err(|err| InstallError(err.0)),
        None => Err(InstallError(format!(
            "{tag} is not a pack to install: the installable ones are {:?}",
            Install::tags()
        ))),
    }
}

/// What `source` gives, fetched from where it says. The request names the file and nothing else: no
/// document, no word, no setting goes with it. What comes back is checked by [`install`] before
/// anything is written.
pub fn fetch(source: &Source) -> Result<Vec<u8>, InstallError> {
    let failed = |why: String| InstallError(format!("{} could not be fetched: {why}", source.url));
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                .build(),
        )
        .proxy(ureq::Proxy::try_from_env())
        .timeout_global(Some(Duration::from_secs(FETCH_TIMEOUT)))
        .build()
        .into();
    let mut response = agent
        .get(&source.url)
        .call()
        .map_err(|err| failed(err.to_string()))?;
    response
        .body_mut()
        .with_config()
        .limit(DOWNLOAD_LIMIT)
        .read_to_vec()
        .map_err(|err| failed(err.to_string()))
}

/// The longest a fetch may take, in seconds: a pack's files are a few megabytes.
const FETCH_TIMEOUT: u64 = 300;

/// The most a download may be: dict_uk's archive is 1.6 MB, so this is room, not a target.
const DOWNLOAD_LIMIT: u64 = 64 * 1024 * 1024;

/// Installs pack `tag` under `packs`, each of its sources' bytes from `get`: [`fetch`], or a file
/// the person has. Nothing is written unless every download hashes to the SHA-256 its source pins,
/// every archive holds the files the pack takes from it, a compiled dictionary compiles, and the
/// dictionary they make parses. A version already there is replaced whole. Returns the directory
/// the pack is in.
pub fn install(
    tag: &str,
    get: impl FnMut(&Source) -> Result<Vec<u8>, InstallError>,
    packs: &Path,
) -> Result<PathBuf, InstallError> {
    install_with(&definition(tag)?, get, packs)
}

/// [`install`], from a definition in hand.
fn install_with(
    install: &Install,
    mut get: impl FnMut(&Source) -> Result<Vec<u8>, InstallError>,
    packs: &Path,
) -> Result<PathBuf, InstallError> {
    let tag = install.tag.as_str();
    // Written beside where it goes, then moved there, so a reader never sees half a pack.
    let target = install.dir(packs);
    let partial = target.with_extension(format!("partial-{}", std::process::id()));
    let failed = |why: String| {
        let _ = fs::remove_dir_all(&partial);
        InstallError(format!("{tag}: {why}"))
    };
    fs::create_dir_all(&partial).map_err(|err| failed(format!("{}: {err}", partial.display())))?;
    for source in &install.sources {
        let bytes = get(source).map_err(|err| failed(err.0))?;
        let digest = hex(&Sha256::digest(&bytes));
        if digest != source.sha256 {
            return Err(failed(format!(
                "{} has SHA-256 {digest}, and {} {} pins {}",
                source.url, install.tag, install.version, source.sha256
            )));
        }
        let write = |file: &str, bytes: &[u8]| {
            fs::write(partial.join(file), bytes)
                .map_err(|err| failed(format!("{}: {err}", partial.join(file).display())))
        };
        match &source.take {
            Take::File(file) => write(file, &bytes)?,
            Take::Archive(files) => {
                let mut zip = zip::ZipArchive::new(Cursor::new(&bytes))
                    .map_err(|err| failed(format!("{} does not open: {err}", source.url)))?;
                for (file, entry) in files {
                    let mut read = Vec::new();
                    zip.by_name(entry)
                        .map_err(|err| failed(format!("the archive has no {entry}: {err}")))?
                        .read_to_end(&mut read)
                        .map_err(|err| failed(format!("{entry} does not read: {err}")))?;
                    write(file, &read)?;
                }
            }
        }
    }
    if let (Some(base), Some(file)) = (&install.compiled, install.compiled_file()) {
        let read = |name: String| {
            fs::read_to_string(partial.join(&name))
                .map_err(|err| failed(format!("{name}, to compile: {err}")))
        };
        let compiled = orthra_packs::installed::compile(
            &read(format!("{base}.aff"))?,
            &read(format!("{base}.dic"))?,
        )
        .map_err(|err| failed(format!("{base} does not compile: {}", err.0)))?;
        fs::write(partial.join(&file), compiled)
            .map_err(|err| failed(format!("{}: {err}", partial.join(&file).display())))?;
    }
    if target.exists() {
        fs::remove_dir_all(&target)
            .map_err(|err| failed(format!("{}: {err}", target.display())))?;
    }
    fs::rename(&partial, &target).map_err(|err| failed(format!("{}: {err}", target.display())))?;

    // Read once now, so a dictionary that does not parse is this install's error and not a
    // check's, later. A pack that fails is not left installed.
    let read = Packs::load(&[tag], Some(packs))
        .map_err(|err| err.0)
        .and_then(|(loaded, _)| match loaded.iter().next() {
            Some(pack) => pack.try_read_dictionaries().map_err(|err| err.0),
            None => Err("not found where it was just written".to_owned()),
        });
    if let Err(why) = read {
        let _ = fs::remove_dir_all(&target);
        return Err(InstallError(format!(
            "{tag}: the installed dictionary does not read: {why}"
        )));
    }
    Ok(target)
}

/// The SHA-256 of `bytes`, in lower-case hex: what a source pins.
pub fn sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// Bytes as lower-case hex.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
