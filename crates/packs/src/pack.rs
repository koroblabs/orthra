//! One pack: its `pack.toml`, and what that names.

use std::{
    borrow::Cow,
    fmt,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use orthra_dict::{
    Candidate, Dictionary,
    compiled::{Bytes, CompiledDict},
    hunspell::HunspellDict,
};
use orthra_text::script::Script;

#[cfg(test)]
mod compiled_english;
#[cfg(test)]
mod tests;

use crate::{
    frequency::Frequencies,
    installed::{self, Install},
    typos::{Fixes, KnownTypos},
    vocabulary::Vocabulary,
    word_list::WordList,
};

/// A pack that could not be read. Its message names the pack and the field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackError(pub String);

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PackError {}

#[derive(Debug)]
pub struct Pack {
    tag: String,
    script: Script,
    /// Read on the first word of the pack's script, and not before.
    dictionaries: Lazy,
    /// How often each word is written, read on the first correction asked for.
    frequency: Option<LazyFrequencies>,
    typos: Arc<KnownTypos>,
    /// Words accepted beside the dictionary: `kubectl`, `PostgreSQL`, `toolchain`. Also what a
    /// correction is offered from beside it, `codebase` for `clodebase`.
    vocabulary: Arc<Vocabulary>,
    /// Words accepted as the rare dictionary's are and never offered: lists too long to read as a
    /// dictionary, `rare-words`.
    rare_words: Vec<WordList>,
    /// Prefixes, in lower case, a word the pack accepts may take and stay a word, accepted as the
    /// rare words are: `un` makes `unconfigured` of `configured`.
    prefixes: Vec<Box<str>>,
    /// Endings a vocabulary word may take and stay a word, accepted as the rare words are: `s`
    /// makes `datagrams` of `datagram`. Only the vocabulary's: no dictionary inflects it.
    vocabulary_suffixes: Vec<Box<str>>,
    word_internal: Vec<char>,
    /// Each character on the left is checked as the one on the right; the finding keeps the original.
    normalize: Vec<(char, char)>,
    /// Lower-cased labels that, with a colon, start a comment line naming people: `author`.
    name_labels: Vec<Box<str>>,
    /// Lower-cased words that start a comment line naming people, colon or not: `copyright`.
    name_lines: Vec<Box<str>>,
}

impl Pack {
    /// The built-in pack `tag`, embedded from `packs/<tag>/` by the build script, so that a fresh
    /// install checks its languages with nothing to set up.
    pub fn builtin(tag: &str) -> Result<Pack, PackError> {
        Pack::builtin_sharing(tag, &mut Shared::default())
    }

    /// The same, sharing what it parses with the packs loaded beside it: en-GB names en-US's
    /// known typos and vocabulary, and the two hold one copy of each.
    pub(crate) fn builtin_sharing(tag: &str, shared: &mut Shared) -> Result<Pack, PackError> {
        Pack::load_sharing(tag, None, shared)?.ok_or_else(|| {
            PackError(format!(
                "{tag} is installed, not built in: `orthra pack install {tag}`"
            ))
        })
    }

    /// Pack `tag`: built in, or installed under `packs`, its definition embedded and its dictionary
    /// from the installed files. `None` for a pack that is installed and is not there: a language
    /// the settings name that has to be fetched, which is not the same as one orthra has no pack
    /// for.
    pub(crate) fn load_sharing(
        tag: &str,
        packs: Option<&Path>,
        shared: &mut Shared,
    ) -> Result<Option<Pack>, PackError> {
        if !crate::builtin::PACKS.contains(&tag) {
            return Err(PackError(format!(
                "no pack for {tag:?}; the packs are {:?}",
                crate::builtin::PACKS
            )));
        }
        let installed = match Install::of(tag).transpose()? {
            Some(install) => match packs.filter(|packs| install.is_installed(packs)) {
                Some(packs) => Some(install.dir(packs)),
                None => return Ok(None),
            },
            None => None,
        };
        let file = |name: &str| match Pack::builtin_file(tag, name) {
            Some(text) => Some(Text::Embedded(text)),
            None => match Pack::builtin_compiled(tag, name) {
                Some(bytes) => Some(Text::Compiled(bytes)),
                None => installed
                    .as_deref()
                    .and_then(|dir| installed::file(dir, name))
                    .map(Text::Installed),
            },
        };
        let toml = Pack::builtin_file(tag, "pack.toml")
            .expect("the build script embeds pack.toml with every pack");
        Pack::from_files_sharing(toml, file, shared)
            .map(Some)
            .map_err(|PackError(why)| PackError(format!("packs/{tag}: {why}")))
    }

    /// A built-in file, named relative to pack `tag`'s directory: `typos.tsv`, or a neighbor's
    /// `../en-US/typos.tsv`.
    fn builtin_file(tag: &str, name: &str) -> Option<&'static str> {
        let mut parts = vec![tag];
        for part in name.split('/') {
            match part {
                ".." => {
                    parts.pop()?;
                }
                "." | "" => {}
                part => parts.push(part),
            }
        }
        let key = parts.join("/");
        crate::builtin::FILES
            .iter()
            .find(|(path, _)| *path == key)
            .map(|(_, text)| *text)
    }

    /// A built-in pack's compiled dictionary `name`, compiled when orthra was: `en_US.fst`.
    fn builtin_compiled(tag: &str, name: &str) -> Option<&'static [u8]> {
        let key = format!("{tag}/{name}");
        crate::builtin::COMPILED
            .iter()
            .find(|(path, _)| *path == key)
            .map(|(_, bytes)| *bytes)
    }

    /// A pack from its `pack.toml` and a way to read the files it names. Built-in and installed
    /// packs go through this one reader, so they cannot come to read a field two ways.
    ///
    /// The files are `'static` because the known typos are read where they lie, never copied.
    pub fn from_files(
        toml_text: &str,
        file: impl Fn(&str) -> Option<&'static str>,
    ) -> Result<Pack, PackError> {
        Pack::from_files_sharing(
            toml_text,
            |name| file(name).map(Text::Embedded),
            &mut Shared::default(),
        )
    }

    fn from_files_sharing(
        toml_text: &str,
        file: impl Fn(&str) -> Option<Text>,
        shared: &mut Shared,
    ) -> Result<Pack, PackError> {
        let table: toml::Table = toml_text
            .parse()
            .map_err(|err| PackError(format!("pack.toml does not parse: {err}")))?;
        let text = |key: &str| {
            table
                .get(key)
                .and_then(toml::Value::as_str)
                .ok_or_else(|| PackError(format!("pack.toml has no `{key}`")))
        };
        let read = |name: &str| {
            file(name)
                .ok_or_else(|| PackError(format!("pack.toml names {name}, which is not there")))
        };
        // The known typos and the vocabulary are part of a pack's definition, which orthra carries;
        // only a dictionary is installed.
        let embedded = |name: &str| match read(name)? {
            Text::Embedded(text) => Ok(text),
            Text::Installed(_) | Text::Compiled(_) => Err(PackError(format!(
                "{name} is installed, and only a dictionary may be"
            ))),
        };
        let one_char = |value: &str| {
            let mut chars = value.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Ok(c),
                _ => Err(PackError(format!("{value:?} is not one character"))),
            }
        };

        let tag = text("tag")?.to_owned();
        let script = Script::from_code(text("script")?)
            .ok_or_else(|| PackError("unknown `script`".to_owned()))?;
        let dictionary = |key: &str| {
            table
                .get("dictionary")
                .and_then(|d| d.get(key))
                .and_then(toml::Value::as_str)
        };
        let source = match (dictionary("hunspell"), dictionary("compiled")) {
            (Some(base), None) => Source::Hunspell {
                aff: read(&format!("{base}.aff"))?,
                dic: read(&format!("{base}.dic"))?,
                rare: match dictionary("rare") {
                    Some(rare) => Some(read(&format!("{rare}.dic"))?),
                    None => None,
                },
            },
            (None, Some(base)) => {
                // Compiled when the pack is installed, or when orthra is for a built-in pack, and
                // never a text: a `.fst` among a pack's files is not one to read.
                let compiled = |name: &str| -> Result<Fsts, PackError> {
                    let forms = read(&format!("{name}.fst"))?;
                    if let Text::Embedded(_) = forms {
                        return Err(PackError(format!(
                            "{name}.fst is compiled from {name}.dic, not written by hand"
                        )));
                    }
                    Ok(Fsts {
                        forms,
                        marks: file(&format!("{name}.marks.fst")),
                    })
                };
                Source::Compiled {
                    aff: read(&format!("{base}.aff"))?,
                    main: compiled(base)?,
                    rare: dictionary("rare").map(compiled).transpose()?,
                }
            }
            _ => {
                return Err(PackError(
                    "pack.toml names one of `dictionary.hunspell` and `dictionary.compiled`"
                        .to_owned(),
                ));
            }
        };
        let dictionaries = Lazy {
            source,
            loaded: OnceLock::new(),
        };
        let frequency = match table.get("frequency").and_then(toml::Value::as_str) {
            Some(name) => Some(LazyFrequencies {
                text: read(name)?,
                loaded: OnceLock::new(),
            }),
            None => None,
        };
        let typos = match table.get("typos").and_then(toml::Value::as_str) {
            Some(name) => shared
                .typos(embedded(name)?)
                .map_err(|why| PackError(format!("{name}: {why}")))?,
            None => Arc::default(),
        };
        let mut texts = Vec::new();
        for name in table
            .get("vocabulary")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            let name = name
                .as_str()
                .ok_or_else(|| PackError("`vocabulary` lists a non-string".to_owned()))?;
            texts.push(embedded(name)?);
        }
        let vocabulary = shared.vocabulary(&texts);
        let strings = |key: &str| {
            table
                .get(key)
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .map(|value| {
                    value
                        .as_str()
                        .ok_or_else(|| PackError(format!("`{key}` lists a non-string")))
                })
                .collect::<Result<Vec<&str>, PackError>>()
        };
        let rare_words = strings("rare-words")?
            .into_iter()
            .map(|name| embedded(name).map(WordList::new))
            .collect::<Result<_, _>>()?;
        let lowered = |key: &str| -> Result<Vec<Box<str>>, PackError> {
            Ok(strings(key)?
                .into_iter()
                .filter(|affix| !affix.is_empty())
                .map(|affix| affix.to_lowercase().into_boxed_str())
                .collect())
        };
        let prefixes = lowered("prefixes")?;
        let vocabulary_suffixes = lowered("vocabulary-suffixes")?;
        let word_internal = table
            .get("word-internal")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .map(|v| one_char(v.as_str().unwrap_or_default()))
            .collect::<Result<_, _>>()?;
        let normalize = table
            .get("normalize")
            .and_then(toml::Value::as_table)
            .into_iter()
            .flatten()
            .map(|(from, to)| Ok((one_char(from)?, one_char(to.as_str().unwrap_or_default())?)))
            .collect::<Result<_, PackError>>()?;
        let lowered_list = |key: &str| -> Vec<Box<str>> {
            table
                .get(key)
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(toml::Value::as_str)
                .map(|label| label.to_lowercase().into_boxed_str())
                .collect()
        };
        let name_labels = lowered_list("name-labels");
        let name_lines = lowered_list("name-lines");
        Ok(Pack {
            tag,
            script,
            dictionaries,
            frequency,
            typos,
            vocabulary,
            rare_words,
            prefixes,
            vocabulary_suffixes,
            word_internal,
            normalize,
            name_labels,
            name_lines,
        })
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn script(&self) -> Script {
        self.script
    }

    pub fn word_internal(&self) -> &[char] {
        &self.word_internal
    }

    /// Labels that, followed by a colon, start a comment line naming people: `Author:`.
    pub fn name_labels(&self) -> &[Box<str>] {
        &self.name_labels
    }

    /// Words that start a comment line naming people, colon or not: `Copyright`.
    pub fn name_lines(&self) -> &[Box<str>] {
        &self.name_lines
    }

    /// Whether the dictionary or the vocabulary accepts `word`, after normalization (`’` checked as
    /// `'`). The vocabulary ignores case: `Kubectl` at the start of a sentence is still `kubectl`.
    pub fn accepts(&self, word: &str) -> bool {
        let word = self.normalized(word);
        self.vocabulary.accepts(&word) || self.dictionaries.get().main.check(&word)
    }

    /// Whether the pack's rare words accept `word`, after normalization: its rare dictionary, its
    /// `rare-words`, or one of its prefixes on a word it accepts. Asked only once neither
    /// [`Pack::accepts`] nor any pack's known typos have answered: an uncommon word is accepted,
    /// and an uncommon word that is also a common misspelling (`incudes`, `algin`) is not.
    pub fn accepts_rare(&self, word: &str) -> bool {
        let word = self.normalized(word);
        self.dictionaries
            .get()
            .rare
            .as_ref()
            .is_some_and(|rare| rare.check(&word))
            || self.rare_words.iter().any(|list| list.accepts(&word))
            || self.derived(&word)
    }

    /// Whether `word` is made of a word the pack accepts, of four letters or more, and one of its
    /// affixes: a prefix on any such word (`un` on `configured`, `sub` on `pool`), or a vocabulary
    /// suffix on a vocabulary word no dictionary has (`datagram` and `s`).
    ///
    /// A dictionary lets a word take only the affixes it is marked for, and software English makes
    /// its own: on the projects beside orthra, 275 of the 2 551 words reported in files people
    /// wrote had a prefix (`unwiped`, `refetched`, `multihost`, 2026-11-08). A shorter base made
    /// `recod` of `cod` and `reson` of `son`. A suffix goes only on a word the vocabulary has and
    /// the dictionary does not, which nothing else inflects: the vocabulary holds plain English too,
    /// and on `information` and `feedback` the suffix made words of `informations` and `feedbacks`.
    fn derived(&self, word: &str) -> bool {
        const BASE: usize = 4;
        let long_enough = |base: &str| base.chars().filter(|c| c.is_alphabetic()).count() >= BASE;
        let prefixed = self.prefixes.iter().any(|prefix| {
            let Some((at, _)) = word.char_indices().nth(prefix.chars().count()) else {
                return false;
            };
            let (head, base) = word.split_at(at);
            head.chars().flat_map(char::to_lowercase).eq(prefix.chars())
                && long_enough(base)
                && self.accepts(base)
        });
        prefixed
            || self.vocabulary_suffixes.iter().any(|suffix| {
                let length = suffix.chars().count();
                let Some((at, _)) = word.char_indices().rev().nth(length - 1) else {
                    return false;
                };
                let (base, tail) = word.split_at(at);
                tail.chars().flat_map(char::to_lowercase).eq(suffix.chars())
                    && long_enough(base)
                    && self.vocabulary.accepts(base)
                    && !self.dictionaries.get().main.check(base)
            })
    }

    /// The fixes for `word` if it is a known typo, after normalization, whatever its case.
    pub fn known_typo(&self, word: &str) -> Option<Fixes<'static>> {
        self.typos.get(&self.normalized(word))
    }

    /// Corrections for `word`, best first, at most `limit`, written with the apostrophe `word` was
    /// written with.
    ///
    /// With the pack's frequencies, the nearest come first and, among those as near, the most
    /// written, a correction split in two last. The dictionary's own order put the right fix
    /// first 41.7 % of the time on UA-GEC; by frequency alone, 47.1 % (2026-11-06).
    pub fn suggest(&self, word: &str, limit: usize, out: &mut Vec<String>) {
        let normalized = self.normalized(word);
        let mut candidates: Vec<Candidate> = Vec::new();
        self.dictionaries
            .get()
            .main
            .suggest(&normalized, &mut candidates);
        let from_dictionary = candidates.len();
        self.vocabulary_near(&normalized, &mut candidates);
        // The nearest first; among those as near, one in the case the word is written in before a
        // name (`leter` gets `letter` before `Lester`), the most written first where the pack knows
        // how often words are, one that keeps the word's first letter, which a typo seldom
        // changes, and the dictionary's before the vocabulary's, each in its own order.
        let frequencies = self.frequency.as_ref().map(LazyFrequencies::get);
        let lower = word.chars().next().is_some_and(char::is_lowercase);
        let first = normalized.chars().next().map(|c| c.to_lowercase().next());
        let mut ranked: Vec<(usize, Candidate)> = candidates.into_iter().enumerate().collect();
        ranked.sort_by_cached_key(|(at, candidate)| {
            (
                candidate.edits,
                candidate.word.contains(' '),
                lower
                    && candidate
                        .word
                        .chars()
                        .next()
                        .is_some_and(char::is_uppercase),
                frequencies.map_or(0, |frequencies| {
                    frequencies.rank(&candidate.word).unwrap_or(u32::MAX)
                }),
                candidate
                    .word
                    .chars()
                    .next()
                    .map(|c| c.to_lowercase().next())
                    != first,
                *at >= from_dictionary,
                *at,
            )
        });
        // A correction one edit away is offered without the ones two away, as Hunspell does:
        // `Schedulor` gets `Scheduler`, not `Schedules` and three more beside it. On the typo
        // sample it cost one typo's right fix in the first five, of 1 618 (2026-11-09).
        let nearest = ranked.first().map(|(_, candidate)| candidate.edits);
        let mut corrections: Vec<String> = ranked
            .into_iter()
            .filter(|(_, candidate)| nearest != Some(1) || candidate.edits == 1)
            .take(limit)
            .map(|(_, candidate)| candidate.word)
            .collect();
        // `мʼята` gets `обʼєкт`, not `об'єкт`: each normalized character the word was written with
        // is put back.
        for (from, to) in &self.normalize {
            if word.contains(*from) {
                for correction in &mut corrections {
                    *correction = correction.replace(*to, &from.to_string());
                }
            }
        }
        out.extend(corrections.into_iter().take(limit));
    }

    /// The vocabulary's words within an edit of `word` (four letters or fewer) or two, that are not
    /// among `candidates` already, added to them in the casing `word` is written in. On the typo
    /// sample this put the right fix first for 0.8 points more of the typos, and in the first five
    /// for 1.1 (2026-11-06): `clodebase` is `codebase`, which no dictionary has.
    ///
    /// A word the dictionary accepts is offered only when nothing nearer is. The languages' lists
    /// hold plain English (`schedule`, `download`), and offered from them at any distance,
    /// `Schedulor` got `Schedule` beside `Scheduler`, which the context tier then put first: the
    /// model lost 46 typos it had right and won 31 (2026-11-06). The dictionary does have
    /// `JavaScript`, and does not suggest it for `javascrpt`: nothing is nearer, so it is offered.
    fn vocabulary_near(&self, word: &str, candidates: &mut Vec<Candidate>) {
        let main = &self.dictionaries.get().main;
        let nearest = candidates.iter().map(|candidate| candidate.edits).min();
        let lower = word.to_lowercase();
        let near: u32 = if lower.chars().count() <= 4 { 1 } else { 2 };
        let title = word.chars().next().is_some_and(char::is_uppercase)
            && word.chars().skip(1).all(|c| !c.is_uppercase());
        let mut entries = Vec::new();
        self.vocabulary.within(&lower, near, &mut entries);
        let mut found: Vec<Candidate> = Vec::new();
        for (entry, edits) in entries {
            if edits == 0 || edits > near {
                continue;
            }
            if nearest.is_some_and(|nearest| nearest < edits) && main.check(entry) {
                continue;
            }
            let shown = if title && entry.chars().all(|c| !c.is_uppercase()) {
                let mut chars = entry.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            } else {
                entry.to_owned()
            };
            let known = |other: &Candidate| other.word.to_lowercase() == shown.to_lowercase();
            if !candidates.iter().any(known) && !found.iter().any(known) {
                found.push(Candidate { word: shown, edits });
            }
        }
        candidates.extend(found);
    }

    /// Whether the dictionaries have been read: by a word of the pack's script, or by
    /// [`Pack::read_dictionaries`].
    pub fn is_loaded(&self) -> bool {
        self.dictionaries.loaded.get().is_some()
    }

    /// Reads the dictionaries now, for a measurement that wants their cost on its own.
    pub fn read_dictionaries(&self) {
        self.dictionaries.get();
    }

    /// Reads the dictionaries now and says why they do not parse, rather than failing a check
    /// later: what an install asks of the files it has just written.
    pub fn try_read_dictionaries(&self) -> Result<(), PackError> {
        if self.dictionaries.loaded.get().is_none() {
            let read = self.dictionaries.parse().map_err(PackError)?;
            let _ = self.dictionaries.loaded.set(read);
        }
        Ok(())
    }

    fn normalized<'w>(&self, word: &'w str) -> Cow<'w, str> {
        if !word
            .chars()
            .any(|c| self.normalize.iter().any(|(from, _)| *from == c))
        {
            return Cow::Borrowed(word);
        }
        Cow::Owned(
            word.chars()
                .map(|c| {
                    self.normalize
                        .iter()
                        .find(|(from, _)| *from == c)
                        .map_or(c, |(_, to)| *to)
                })
                .collect(),
        )
    }
}

/// A pack's dictionaries, read on the first word of the pack's script and not before, so a language
/// a project enables and no document writes in costs nothing. uk-UA's compiled dictionary is 3.9 MB
/// and 0.02 s to read, where its Hunspell files were 37 MB and half a second (measured
/// 2026-11-06). English's are compiled when orthra is and read where they are embedded: no heap,
/// where spellbook's tables for en-US and en-GB were 12.8 MB of it (2026-11-09).
///
/// The texts are a built-in pack's, embedded and tested by its smoke test, so reading them cannot
/// fail at runtime; a pack installed from a file is read once when it is installed, for the same
/// reason.
struct Lazy {
    source: Source,
    loaded: OnceLock<Dictionaries>,
}

/// Where a pack's dictionary is read from: `pack.toml`'s `dictionary`, a table and not a fallback.
enum Source {
    /// A Hunspell `.aff` and `.dic`, read as they are by spellbook.
    Hunspell {
        aff: Text,
        dic: Text,
        /// Read with the main dictionary's affixes: its words are lines of the same family.
        rare: Option<Text>,
    },
    /// The forms a Hunspell dictionary makes, compiled when the pack was installed or, for a
    /// built-in pack, when orthra was, and the `.aff` for what a checked word goes through first.
    Compiled {
        aff: Text,
        main: Fsts,
        rare: Option<Fsts>,
    },
}

/// One compiled dictionary's files: its forms, and its forms' marks if any has one.
struct Fsts {
    forms: Text,
    marks: Option<Text>,
}

impl Fsts {
    fn read(&self, aff: &str) -> Result<Box<dyn Dictionary>, String> {
        let bytes = |text: &Text| match text {
            Text::Compiled(bytes) => Bytes::Embedded(bytes),
            text => Bytes::Owned(text.read_bytes()),
        };
        CompiledDict::new(aff, bytes(&self.forms), self.marks.as_ref().map(bytes))
            .map(|dictionary| Box::new(dictionary) as Box<dyn Dictionary>)
            .map_err(|err| err.0)
    }
}

/// A pack's frequencies and the file they are read from, on the first correction asked for.
struct LazyFrequencies {
    text: Text,
    loaded: OnceLock<Frequencies>,
}

impl LazyFrequencies {
    fn get(&self) -> &Frequencies {
        self.loaded
            .get_or_init(|| Frequencies::parse(self.text.read()))
    }
}

impl fmt::Debug for LazyFrequencies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyFrequencies")
            .field("loaded", &self.loaded.get().is_some())
            .finish_non_exhaustive()
    }
}

/// A file of a pack: embedded in the binary, or an installed pack's, read when it is first needed.
#[derive(Debug, Clone)]
pub(crate) enum Text {
    Embedded(&'static str),
    /// A built-in pack's compiled dictionary, compiled when orthra was and embedded.
    Compiled(&'static [u8]),
    Installed(PathBuf),
}

impl Text {
    fn read(&self) -> Cow<'static, str> {
        match self {
            Text::Embedded(text) => Cow::Borrowed(text),
            Text::Compiled(bytes) => String::from_utf8_lossy(bytes),
            Text::Installed(path) => Cow::Owned(installed::read(path)),
        }
    }

    /// An installed file's bytes; an embedded one's as they are.
    fn read_bytes(&self) -> Vec<u8> {
        match self {
            Text::Embedded(text) => text.as_bytes().to_vec(),
            Text::Compiled(bytes) => bytes.to_vec(),
            Text::Installed(path) => installed::read_bytes(path),
        }
    }
}

#[derive(Debug)]
struct Dictionaries {
    main: Box<dyn Dictionary>,
    /// Words a larger dictionary adds, uncommon and so also likely misspellings of common ones:
    /// accepted only where the known typos do not name the word ([`Pack::accepts_rare`]).
    rare: Option<Box<dyn Dictionary>>,
}

impl Lazy {
    fn get(&self) -> &Dictionaries {
        self.loaded.get_or_init(|| {
            self.parse().unwrap_or_else(|err| {
                panic!(
                    "a pack dictionary that does not parse, which a built-in pack's smoke test and \
                     an installed pack's install refuse: {err}"
                )
            })
        })
    }

    fn parse(&self) -> Result<Dictionaries, String> {
        match &self.source {
            Source::Hunspell { aff, dic, rare } => {
                let aff = aff.read();
                let read = |dic: &Text| -> Result<Box<dyn Dictionary>, String> {
                    HunspellDict::new(&aff, &dic.read())
                        .map(|dictionary| Box::new(dictionary) as Box<dyn Dictionary>)
                        .map_err(|err| err.0)
                };
                Ok(Dictionaries {
                    main: read(dic)?,
                    rare: rare.as_ref().map(read).transpose()?,
                })
            }
            Source::Compiled { aff, main, rare } => {
                let aff = aff.read();
                Ok(Dictionaries {
                    main: main.read(&aff)?,
                    rare: rare.as_ref().map(|rare| rare.read(&aff)).transpose()?,
                })
            }
        }
    }
}

impl fmt::Debug for Lazy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lazy")
            .field("loaded", &self.loaded.get().is_some())
            .finish_non_exhaustive()
    }
}

/// What packs loaded together have parsed, so that two naming one file hold one copy of it.
///
/// A file is known by its text: the same embedded file is the same `&'static str` to every pack
/// that names it, so its address and length identify it with no path to resolve. An installed
/// pack's own files are other texts, and share nothing, which is correct.
#[derive(Debug, Default)]
pub(crate) struct Shared {
    typos: Vec<(TextId, Arc<KnownTypos>)>,
    vocabularies: Vec<(Vec<TextId>, Arc<Vocabulary>)>,
}

/// A file's text, by its address and length.
type TextId = (usize, usize);

impl Shared {
    fn identity(text: &str) -> TextId {
        (text.as_ptr() as usize, text.len())
    }

    fn typos(&mut self, text: &'static str) -> Result<Arc<KnownTypos>, String> {
        let id = Shared::identity(text);
        if let Some((_, typos)) = self.typos.iter().find(|(seen, _)| *seen == id) {
            return Ok(Arc::clone(typos));
        }
        let typos = Arc::new(KnownTypos::parse(text)?);
        self.typos.push((id, Arc::clone(&typos)));
        Ok(typos)
    }

    fn vocabulary(&mut self, texts: &[&'static str]) -> Arc<Vocabulary> {
        let ids: Vec<TextId> = texts.iter().map(|text| Shared::identity(text)).collect();
        if let Some((_, words)) = self.vocabularies.iter().find(|(seen, _)| *seen == ids) {
            return Arc::clone(words);
        }
        let words = Arc::new(Vocabulary::new(texts));
        self.vocabularies.push((ids, Arc::clone(&words)));
        words
    }
}
