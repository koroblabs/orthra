//! Packs that are installed rather than built in, and where an installed pack lives.
//!
//! A pack whose directory under `packs/` holds an `install.toml` is installed: orthra carries its
//! definition, `pack.toml` and the known typos and vocabulary it names, and fetches the rest
//! (its dictionary) from the source `install.toml` pins by version and SHA-256. uk-UA is one,
//! because dict_uk is GPL-3.0 and orthra does not carry it. An installed pack lives in
//! `<packs>/<tag>/<version>/`, so a pack installed for another version of orthra's pin is not read
//! as this one.
//!
//! A pack whose `pack.toml` says `dictionary = { compiled = "uk_UA" }` also has the forms its
//! Hunspell files make, compiled once when it is installed into `uk_UA.fst` ([`compile`]).
//!
//! Installing, which fetches and writes, is `orthra-install`'s. This module only reads: the
//! definition, and an installed pack's files, the dictionaries on the first word of its script.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::pack::PackError;

#[cfg(test)]
mod tests;

/// What `install.toml` says: where a pack's files come from and what they must hash to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Install {
    pub tag: String,
    /// The version installed, and the directory it is installed in.
    pub version: String,
    pub sources: Vec<Source>,
    /// The Hunspell dictionary compiled when it is installed, by its base name: `uk_UA` is
    /// `uk_UA.aff` and `uk_UA.dic` from the sources, compiled to `uk_UA.fst`.
    pub compiled: Option<String>,
}

/// One download a pack's files come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub url: String,
    /// The download's SHA-256, in hex.
    pub sha256: String,
    /// The license its files are under, and where it is stated.
    pub license: String,
    pub take: Take,
}

/// What a download gives the pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Take {
    /// An archive: each pack file, and the archive entry it is taken from.
    Archive(Vec<(String, String)>),
    /// The download is the pack file, named so.
    File(String),
}

impl Source {
    /// The pack files this source gives.
    pub fn files(&self) -> Vec<&str> {
        match &self.take {
            Take::Archive(files) => files.iter().map(|(file, _)| file.as_str()).collect(),
            Take::File(file) => vec![file.as_str()],
        }
    }
}

impl Install {
    /// The installed pack `tag`'s definition, or `None` for a pack that is built in, or none at all.
    pub fn of(tag: &str) -> Option<Result<Install, PackError>> {
        let text = crate::builtin::FILES
            .iter()
            .find(|(path, _)| *path == format!("{tag}/install.toml"))
            .map(|(_, text)| *text)?;
        Some(Install::parse(tag, text).and_then(|mut install| {
            install.compiled = compiled(tag)?;
            Ok(install)
        }))
    }

    /// Every pack that is installed rather than built in, by tag.
    pub fn tags() -> Vec<&'static str> {
        crate::builtin::PACKS
            .iter()
            .copied()
            .filter(|tag| Install::of(tag).is_some())
            .collect()
    }

    fn parse(tag: &str, text: &str) -> Result<Install, PackError> {
        Install::read(tag, &format!("packs/{tag}/install.toml"), text)
    }

    /// The download definition `text`, read from `origin` (for its errors), of what is installed
    /// as `name`: a pack's `install.toml`, or a model's, which is written the same way.
    pub fn read(name: &str, origin: &str, text: &str) -> Result<Install, PackError> {
        let tag = name;
        let fail = |why: String| PackError(format!("{origin}: {why}"));
        let table: toml::Table = text.parse().map_err(|err| fail(format!("{err}")))?;
        let version = table
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| fail("no `version`".to_owned()))?
            .to_owned();
        let sources = table
            .get("source")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| fail("no `[[source]]`".to_owned()))?
            .iter()
            .map(|source| Source::parse(source).map_err(fail))
            .collect::<Result<_, _>>()?;
        Ok(Install {
            tag: tag.to_owned(),
            version,
            sources,
            compiled: None,
        })
    }

    /// Every file of the pack, from all its sources.
    pub fn files(&self) -> Vec<&str> {
        self.sources.iter().flat_map(Source::files).collect()
    }

    /// The file the compiled dictionary is written to, if the pack has one: `uk_UA.fst`.
    pub fn compiled_file(&self) -> Option<String> {
        self.compiled.as_ref().map(|base| format!("{base}.fst"))
    }

    /// The licenses of what is installed, each once.
    pub fn licenses(&self) -> Vec<&str> {
        let mut licenses: Vec<&str> = Vec::new();
        for source in &self.sources {
            if !licenses.contains(&source.license.as_str()) {
                licenses.push(&source.license);
            }
        }
        licenses
    }

    /// Where this version of the pack is installed under `packs`.
    pub fn dir(&self, packs: &Path) -> PathBuf {
        packs.join(&self.tag).join(&self.version)
    }

    /// Whether every file of the pack is there, the compiled dictionary included.
    pub fn is_installed(&self, packs: &Path) -> bool {
        let dir = self.dir(packs);
        self.files()
            .into_iter()
            .map(str::to_owned)
            .chain(self.compiled_file())
            .all(|file| dir.join(file).is_file())
    }
}

impl Source {
    fn parse(value: &toml::Value) -> Result<Source, String> {
        let field = |key: &str| {
            value
                .get(key)
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("a `[[source]]` with no `{key}`"))
        };
        let take = match (value.get("files"), value.get("file")) {
            (Some(files), None) => Take::Archive(
                files
                    .as_table()
                    .ok_or("`files` is not a table")?
                    .iter()
                    .map(|(file, entry)| {
                        entry
                            .as_str()
                            .map(|entry| (file.clone(), entry.to_owned()))
                            .ok_or_else(|| format!("`files.{file}` is not a string"))
                    })
                    .collect::<Result<_, _>>()?,
            ),
            (None, Some(file)) => {
                Take::File(file.as_str().ok_or("`file` is not a string")?.to_owned())
            }
            _ => {
                return Err("a `[[source]]` names either `files` (an archive) or `file`".to_owned());
            }
        };
        Ok(Source {
            url: field("url")?,
            sha256: field("sha256")?.to_ascii_lowercase(),
            license: field("license")?,
            take,
        })
    }
}

/// The base name of pack `tag`'s compiled dictionary, as its `pack.toml` names it.
fn compiled(tag: &str) -> Result<Option<String>, PackError> {
    let Some((_, text)) = crate::builtin::FILES
        .iter()
        .find(|(path, _)| *path == format!("{tag}/pack.toml"))
    else {
        return Ok(None);
    };
    let table: toml::Table = text
        .parse()
        .map_err(|err| PackError(format!("packs/{tag}/pack.toml: {err}")))?;
    Ok(table
        .get("dictionary")
        .and_then(|d| d.get("compiled"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned))
}

/// The forms a Hunspell dictionary makes, compiled to what a `compiled` pack reads: what an
/// install writes beside the files it fetched. About a second for dict_uk, and never all the forms
/// in memory at once.
pub fn compile(aff: &str, dic: &str) -> Result<Vec<u8>, PackError> {
    orthra_dict::compiled::build(aff, dic).map_err(|err| PackError(err.0))
}

/// The installed pack's file `name` in `dir`, if it is there.
pub(crate) fn file(dir: &Path, name: &str) -> Option<PathBuf> {
    Some(dir.join(name)).filter(|path| path.is_file())
}

/// An installed pack's file, read when it is first needed: its dictionary, on the first word of its
/// script. It was read whole when it was installed, so a failure now is a file changed since, and
/// says which.
pub(crate) fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| unreadable(path, &err))
}

/// The same, for a file that is not text: a compiled dictionary.
pub(crate) fn read_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|err| unreadable(path, &err))
}

fn unreadable(path: &Path, err: &std::io::Error) -> ! {
    panic!(
        "{}: an installed pack's file that cannot be read now ({err}); `orthra pack install` it again",
        path.display()
    )
}
