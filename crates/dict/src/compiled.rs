//! A dictionary compiled from Hunspell's files to every word form they make, held as an FST.
//!
//! What spellbook does at each lookup -- strip a suffix, find the stem, check its flags -- is done
//! once, when the pack is installed. dict_uk's 353 211 stems and 92 suffix classes make 3 555 574
//! forms, and an FST holds them in 3.9 MB, where spellbook's tables are 37 MB of memory (measured
//! 2026-11-06). Checking a word is a lookup. A correction is any form within two edits, found by
//! walking the FST with a Levenshtein automaton ([`within`]) in a few milliseconds, where
//! spellbook's n-gram pass took 150 to 300 ms a word on the same dictionary.
//!
//! A word is accepted by the rules spellbook applies, in its order: input conversion (`ICONV`),
//! ignored characters (`IGNORE`), numbers, a trailing full stop, `BREAK`, and the casings Hunspell
//! allows (`Київ` as `КИЇВ`, `вода` as `Вода`, not `київ`). A dictionary that needs what a list of
//! forms cannot say does not compile, and says why ([`aff`]).

use fst::{IntoStreamer, Map, Set, Streamer};

use crate::{Candidate, DictError, Dictionary, edits};

mod aff;
mod within;

#[cfg(test)]
mod tests;

use aff::{Aff, Lookup};
use within::Within;

/// What [`build`] writes, as the FST's type. A file of another format is refused, not misread:
/// changing how forms are made changes this, and every `install.toml` whose pack is compiled
/// gets a new version, so the next start installs the pack again.
pub const FORMAT: u64 = 1;

/// Longer words are not words: spellbook's limit, in bytes.
const LONGEST_WORD: usize = 360;

/// Words this short, in characters, are corrected within one edit, not two: within two, `мена`
/// would be close to most four-letter words.
const SHORT: usize = 4;

/// The most forms one search collects, whatever the query: a bound on the work, not a ranking.
const MOST_FOUND: usize = 2000;

/// How much of the forms one pass of [`build`] sorts at a time, in bytes. The forms are made once
/// per pass and only those in its range kept, so building never holds every form at once: dict_uk's
/// are 86 MB of text, and the server installs a pack while it runs.
const PASS_BYTES: usize = 8 << 20;

/// How many leading bytes of a form choose its pass: two Cyrillic letters.
const KEY_BYTES: usize = 4;

pub struct CompiledDict {
    forms: Set<Bytes>,
    /// What else is known of some forms ([`UNSUGGESTED`], [`UPPER_ONLY`]).
    marks: Option<Map<Bytes>>,
    lookup: Lookup,
}

/// A mark: the form is a word and never a correction, `NOSUGGEST`'s.
const UNSUGGESTED: u64 = 1;

/// A mark: the form is not a word, and its letters all in capitals are. Hunspell and spellbook add
/// such a form for a word of mixed case, `Github` for `GitHub`, so that `GITHUB` is a word and
/// `Github` is not.
const UPPER_ONLY: u64 = 2;

/// What [`build`] makes: every form, and the marks of the forms that have one.
#[derive(Debug)]
pub struct Compiled {
    pub forms: Vec<u8>,
    /// Empty when no form has a mark.
    pub marks: Vec<u8>,
}

/// A compiled dictionary's bytes: an installed pack's, read from its file, or a built-in pack's,
/// compiled when orthra was and embedded in the binary, which costs no memory until it is read
/// and then only the pages read.
#[derive(Debug)]
pub enum Bytes {
    Owned(Vec<u8>),
    Embedded(&'static [u8]),
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            Bytes::Owned(bytes) => bytes,
            Bytes::Embedded(bytes) => bytes,
        }
    }
}

impl CompiledDict {
    /// The dictionary [`build`] wrote as `forms` and `unsuggested`, with the `.aff` it was built
    /// from, whose conversions every word checked goes through.
    pub fn new(aff: &str, forms: Bytes, marks: Option<Bytes>) -> Result<CompiledDict, DictError> {
        let read = |bytes: Bytes| {
            let fst = fst::raw::Fst::new(bytes).map_err(|err| {
                DictError(format!("a compiled dictionary that does not read: {err}"))
            })?;
            if fst.fst_type() != FORMAT {
                return Err(DictError(format!(
                    "a compiled dictionary of format {}, and this orthra reads {FORMAT}: install \
                     the pack again",
                    fst.fst_type()
                )));
            }
            Ok(fst)
        };
        Ok(CompiledDict {
            forms: Set::from(read(forms)?),
            marks: marks.map(read).transpose()?.map(Map::from),
            lookup: Aff::parse(aff)?.lookup,
        })
    }

    /// Whether `form` has `mark`.
    fn marked(&self, form: &str, mark: u64) -> bool {
        self.marks
            .as_ref()
            .and_then(|marks| marks.get(form))
            .is_some_and(|marks| marks & mark != 0)
    }

    /// How many forms it holds.
    pub fn len(&self) -> usize {
        self.forms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.forms.is_empty()
    }

    fn has(&self, form: &str) -> bool {
        self.forms.contains(form)
    }

    /// `word` as spellbook converts it before anything else: `ICONV`, longest pattern first at
    /// each place, scanning on after what was put in.
    fn convert(&self, word: &str) -> String {
        let mut out = String::with_capacity(word.len());
        let mut rest = word;
        'scan: while let Some(c) = rest.chars().next() {
            for (from, to) in &self.lookup.conversions {
                if let Some(after) = rest.strip_prefix(from.as_str()) {
                    out.push_str(to);
                    rest = after;
                    continue 'scan;
                }
            }
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
        out
    }

    fn spell_break(&self, word: &str, depth: usize) -> bool {
        if self.spell_casing(word) {
            return true;
        }
        if depth == 9 {
            return false;
        }
        let breaks = &self.lookup.breaks;
        breaks.start.iter().any(|pattern| {
            word.strip_prefix(pattern.as_str())
                .is_some_and(|rest| self.spell_break(rest, depth + 1))
        }) || breaks.end.iter().any(|pattern| {
            word.strip_suffix(pattern.as_str())
                .is_some_and(|rest| self.spell_break(rest, depth + 1))
        }) || breaks.middle.iter().any(|pattern| {
            word.split_once(pattern.as_str())
                .is_some_and(|(left, right)| {
                    !right.is_empty()
                        && self.spell_break(left, depth + 1)
                        && self.spell_break(right, depth + 1)
                })
        })
    }

    fn spell_casing(&self, word: &str) -> bool {
        match Casing::of(word) {
            Casing::None | Casing::Mixed => self.has(word),
            Casing::Title => self.has(word) || self.has(&word.to_lowercase()),
            Casing::Upper => {
                self.has(word)
                    || self.apostrophe_upper(word)
                    || self.has(&title(word))
                    || self.marked(&title(word), UPPER_ONLY)
                    || self.has(&word.to_lowercase())
            }
        }
    }

    /// `SANT'ELIA` as `Sant'Elia` or `sant'Elia`: spellbook's rule for a capitalized word with an
    /// apostrophe inside it.
    fn apostrophe_upper(&self, word: &str) -> bool {
        let Some(at) = word.find('\'').filter(|at| *at != word.len() - 1) else {
            return false;
        };
        let (before, after) = word.split_at(at + 1);
        let after = title(after);
        self.has(&format!("{}{after}", before.to_lowercase()))
            || self.has(&format!("{}{after}", title(before)))
    }

    /// Every form within `edits` of `query`, collected into `found`.
    fn search(&self, query: &str, edits: usize, found: &mut Vec<String>) {
        let Some(within) = Within::new(query, edits) else {
            return;
        };
        let mut stream = self.forms.search(within).into_stream();
        while let Some(form) = stream.next() {
            if found.len() >= MOST_FOUND {
                return;
            }
            if let Ok(form) = std::str::from_utf8(form) {
                found.push(form.to_owned());
            }
        }
    }
}

impl Dictionary for CompiledDict {
    fn check(&self, word: &str) -> bool {
        if word.len() > LONGEST_WORD {
            return false;
        }
        let word = self.convert(word);
        if word.is_empty() {
            return true;
        }
        let trimmed = word.trim_end_matches('.');
        let abbreviated = trimmed.len() != word.len();
        if is_number(trimmed) {
            return true;
        }
        let trimmed: String = trimmed
            .chars()
            .filter(|c| !self.lookup.ignore.contains(c))
            .collect();
        self.spell_break(&trimmed, 0) || (abbreviated && self.spell_break(&word, 0))
    }

    /// Every form within two edits of `word` (one, for a word of [`SHORT`] characters or fewer),
    /// in the casing `word` is written in, nearest first and alphabetical among equals: the pack
    /// orders equals by how often each is written.
    fn suggest(&self, word: &str, out: &mut Vec<Candidate>) {
        let word: String = self
            .convert(word)
            .chars()
            .filter(|c| !self.lookup.ignore.contains(c))
            .collect();
        let casing = Casing::of(&word);
        let lower = word.to_lowercase();
        let allowed = if lower.chars().count() <= SHORT { 1 } else { 2 };
        let mut found = Vec::new();
        self.search(&lower, allowed, &mut found);
        // A name is in the dictionary capitalized: `Латунского` finds `Латунського` this way.
        match casing {
            Casing::None => {}
            Casing::Title | Casing::Mixed => self.search(&word, allowed, &mut found),
            Casing::Upper => self.search(&title(&word), allowed, &mut found),
        }
        let mut candidates: Vec<Candidate> = found
            .into_iter()
            .map(|form| match casing {
                Casing::Title if Casing::of(&form) == Casing::None => title(&form),
                Casing::Upper => form.to_uppercase(),
                _ => form,
            })
            .filter(|form| *form != word)
            .filter(|form| !self.marked(form, UNSUGGESTED))
            .map(|form| Candidate {
                edits: edits::between(&word, &form),
                word: form,
            })
            .collect();
        candidates.sort_by(|a, b| a.edits.cmp(&b.edits).then_with(|| a.word.cmp(&b.word)));
        candidates.dedup_by(|a, b| a.word == b.word);
        out.extend(candidates);
    }
}

impl std::fmt::Debug for CompiledDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledDict")
            .field("forms", &self.forms.len())
            .finish_non_exhaustive()
    }
}

/// Compiles a Hunspell dictionary to the forms it makes, as [`CompiledDict::new`] reads them.
///
/// The forms are made in passes, each keeping those whose first bytes fall in its range, so that
/// no more than about [`PASS_BYTES`] of them are held at once; the FST takes them in order, pass
/// after pass. A stem long enough that no suffix of its reaches its first bytes makes forms in one
/// range only, and the other passes skip it whole, so the stems are expanded about twice in all:
/// dict_uk compiles in 1.9 s (2026-11-06), 4.2 s before that and before its suffixes were tested
/// in groups.
pub fn build(aff: &str, dic: &str) -> Result<Compiled, DictError> {
    build_in(aff, dic, PASS_BYTES)
}

/// [`build`], in passes of about `pass_bytes`.
fn build_in(aff: &str, dic: &str, pass_bytes: usize) -> Result<Compiled, DictError> {
    let aff = Aff::parse(aff)?;
    let failed = |err: fst::Error| DictError(format!("the forms do not compile: {err}"));

    // How many bytes of forms start with each key, to cut the passes, and the marks, which are
    // few: the forms of the stems never offered, and a mixed-case form's capitalized one.
    let mut sizes: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    let mut unsuggested = std::collections::BTreeSet::new();
    let mut marks: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    aff.forms(
        dic,
        |_, _| true,
        |form, offered| {
            *sizes.entry(key(form)).or_default() += form.len();
            if !offered {
                unsuggested.insert(form.to_owned());
            }
            if Casing::of(form) == Casing::Mixed {
                *marks.entry(title(form)).or_default() |= UPPER_ONLY;
            }
        },
    )?;
    // A form another stem makes, and offers, is offered.
    let mut offered_too = std::collections::HashSet::new();
    let mut keys: Vec<(u32, usize)> = sizes.into_iter().collect();
    keys.sort_unstable();
    let mut passes: Vec<(u32, u32)> = Vec::new();
    let mut size = 0;
    for (key, bytes) in keys {
        match passes.last_mut() {
            Some((_, last)) if size + bytes <= pass_bytes => {
                *last = key;
                size += bytes;
            }
            _ => {
                passes.push((key, key));
                size = bytes;
            }
        }
    }

    let mut builder = fst::raw::Builder::new_type(Vec::new(), FORMAT).map_err(failed)?;
    let mut text = String::new();
    let mut spans: Vec<(u32, u32)> = Vec::new();
    for (first, last) in passes {
        text.clear();
        spans.clear();
        let range = first..=last;
        aff.forms(
            dic,
            |stem, reach| stem.len() < KEY_BYTES + reach || range.contains(&key(stem)),
            |form, offered| {
                if offered && unsuggested.contains(form) {
                    offered_too.insert(form.to_owned());
                }
                if range.contains(&key(form)) {
                    let start = text.len();
                    text.push_str(form);
                    // Under 4 GB: a pass is at most PASS_BYTES and one key's forms.
                    spans.push((start as u32, form.len() as u32));
                }
            },
        )?;
        let span =
            |(start, len): (u32, u32)| &text.as_bytes()[start as usize..(start + len) as usize];
        spans.sort_unstable_by(|a, b| span(*a).cmp(span(*b)));
        spans.dedup_by(|a, b| span(*a) == span(*b));
        for at in &spans {
            builder.add(span(*at)).map_err(failed)?;
        }
    }
    for form in unsuggested
        .iter()
        .filter(|form| !offered_too.contains(*form))
    {
        *marks.entry(form.clone()).or_default() |= UNSUGGESTED;
    }
    let mut marked = fst::raw::Builder::new_type(Vec::new(), FORMAT).map_err(failed)?;
    for (form, mark) in &marks {
        marked.insert(form.as_bytes(), *mark).map_err(failed)?;
    }
    Ok(Compiled {
        forms: builder.into_inner().map_err(failed)?,
        marks: if marks.is_empty() {
            Vec::new()
        } else {
            marked.into_inner().map_err(failed)?
        },
    })
}

/// The leading bytes of `form` that choose its pass, as a number whose order is the forms' order.
fn key(form: &str) -> u32 {
    let mut key = [0; KEY_BYTES];
    let bytes = &form.as_bytes()[..form.len().min(KEY_BYTES)];
    key[..bytes.len()].copy_from_slice(bytes);
    u32::from_be_bytes(key)
}

/// Hunspell's casing classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Casing {
    /// No capital: `вода`.
    None,
    /// The first letter alone: `Київ`.
    Title,
    /// Every letter: `НАТО`.
    Upper,
    /// Anything else: `iPhone`, `McDonald`. Checked as written.
    Mixed,
}

impl Casing {
    fn of(word: &str) -> Casing {
        let (mut upper, mut lower) = (0, 0);
        for c in word.chars() {
            upper += usize::from(c.is_uppercase());
            lower += usize::from(c.is_lowercase());
        }
        let first_upper = word.chars().next().is_some_and(char::is_uppercase);
        match (upper, lower) {
            (0, _) => Casing::None,
            (1, _) if first_upper => Casing::Title,
            (_, 0) => Casing::Upper,
            _ => Casing::Mixed,
        }
    }
}

/// The first character upper-cased and the rest lower-cased, as spellbook title-cases.
fn title(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first
            .to_uppercase()
            .chain(chars.flat_map(char::to_lowercase))
            .collect(),
        None => String::new(),
    }
}

/// A number as spellbook accepts one: digits, with single `.`, `,` or `-` between them, and an
/// optional leading minus.
fn is_number(word: &str) -> bool {
    let word = word.strip_prefix('-').unwrap_or(word);
    let mut separated = true;
    for c in word.chars() {
        match c {
            '0'..='9' => separated = false,
            '.' | '-' | ',' if !separated => separated = true,
            _ => return false,
        }
    }
    !separated
}
