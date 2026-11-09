//! A word's verdict, and why: every rule that can accept a word, in the order a check asks them.
//!
//! The same function answers a check and `orthra explain`, so what explain says decided a word is
//! what decided it.

use std::{borrow::Cow, cell::OnceCell};

use orthra_hosts::SpanKind;
use orthra_lexicon::{Lexicon, Verdict};
use orthra_packs::Packs;
use orthra_segment::Word;
use orthra_text::{TextRange, TextSize, script::Script};
use rustc_hash::FxHashSet;

/// Which rule decided a word's verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// A words file lists it: the project's `.orthra/words.txt`, or the user's.
    WordsFile,
    /// The file's own `orthra: ignore` directive lists it, or it is part of the directive.
    Directive,
    /// It is a word of the file's language: `async`, `usize` in Rust.
    HostVocabulary,
    /// The run it was cut from is a word: `Postgre` out of `PostgreSQL`.
    WholeRun,
    /// This file declares it, and this is not the declaration.
    DeclaredHere,
    /// The project declares it, in a manifest or another file, and this is not the declaration.
    DeclaredByProject,
    /// The packs said so: a dictionary, a vocabulary, a rare word, a known typo, or nothing.
    Packs,
}

/// Everything a verdict reads beyond the word itself, for one document.
pub(crate) struct Judge<'a> {
    pub(crate) packs: &'a Packs,
    pub(crate) words_files: &'a FxHashSet<Box<str>>,
    pub(crate) directives: &'a Directives,
    pub(crate) host_words: Option<&'a FxHashSet<Box<str>>>,
    /// This document's declarations, read now and never kept: see `Engine::declared`.
    pub(crate) local: &'a FxHashSet<Box<str>>,
    pub(crate) declared: &'a FxHashSet<Box<str>>,
}

impl Judge<'_> {
    /// The verdict on `word`, found in a span of kind `place`, and the rule that gave it.
    pub(crate) fn verdict(
        &self,
        lexicon: &mut Lexicon,
        text: &str,
        word: &Word,
        place: SpanKind,
    ) -> (Verdict, Why) {
        // Most words are lower case already: borrow them, and allocate only for the rest.
        let written = unescaped(&text[word.range]);
        let lower: Cow<'_, str> = if written.chars().any(char::is_uppercase) {
            written.to_lowercase().into()
        } else {
            Cow::Borrowed(&*written)
        };
        // A words file's words, the file's own `orthra: ignore` list, and the host's words are
        // words wherever they are written. So is the directive itself. What a person listed is a
        // word in every part of a run it names whole: `TruStage` is cut in two, its first part
        // alone is a known typo of `true`, and a list can only name what it was written as.
        // The run is lower-cased only for a word cut from one that a list does not name itself.
        // Both are compared in the form the packs check a word in, whatever its apostrophe.
        let form = self.packs.normalized(word.script, &lower);
        let run = OnceCell::new();
        let listed = |list: &FxHashSet<Box<str>>| {
            list.contains(&*form)
                || word.run != word.range
                    && !list.is_empty()
                    && list.contains(&**run.get_or_init(|| {
                        let lower = unescaped(&text[word.run]).to_lowercase();
                        self.packs.normalized(word.script, &lower).into_owned()
                    }))
        };
        if listed(self.words_files) {
            return (Verdict::Correct, Why::WordsFile);
        }
        if listed(&self.directives.words) || self.directives.covers(word.range) {
            return (Verdict::Correct, Why::Directive);
        }
        if self.host_words.is_some_and(|words| words.contains(&*lower)) {
            return (Verdict::Correct, Why::HostVocabulary);
        }
        let packs = self.packs;
        // A run a pack accepts whole is accepted in every part: `PostgreSQL` is a word, and
        // `Postgre` cut out of it is not a typo.
        if word.run != word.range {
            let id = lexicon.intern(&unescaped(&text[word.run]));
            if lexicon.verdict(id, |written| judge(packs, word.script, written)) == Verdict::Correct
            {
                return (Verdict::Correct, Why::WholeRun);
            }
        }
        let id = lexicon.intern(&written);
        let verdict = lexicon.verdict(id, |written| judge(packs, word.script, written));
        // What the project declares is accepted everywhere but at a declaration, which is still
        // held to known typos: a typo declared once is reported once, where it was written. And
        // only an unknown word is: a known typo the project declares, in a test's `fn
        // fix_recieve`, would otherwise be accepted in every file of it (`CLAUDE.md` §5).
        if verdict == Verdict::Unknown && place != SpanKind::Declaration {
            if self.local.contains(&*lower) {
                return (Verdict::Correct, Why::DeclaredHere);
            }
            if self.declared.contains(&*lower) {
                return (Verdict::Correct, Why::DeclaredByProject);
            }
        }
        (verdict, Why::Packs)
    }
}

/// The names `words` declares in `text`, lower-cased.
pub(crate) fn local_declarations(
    text: &str,
    words: &[Word],
    places: &[SpanKind],
) -> FxHashSet<Box<str>> {
    words
        .iter()
        .zip(places)
        .filter(|(_, place)| **place == SpanKind::Declaration)
        .map(|(word, _)| text[word.range].to_lowercase().into_boxed_str())
        .collect()
}

/// What a file says to ignore: `orthra: ignore qzxv frobnicate`, in a comment anywhere in it. The
/// file says it, so its findings stay a function of its text (`CLAUDE.md` §1.2).
pub const IGNORE_DIRECTIVE: &str = "orthra: ignore";

/// What follows [`IGNORE_DIRECTIVE`] in a directive that ignores every word of the line after it,
/// whatever the words: `// orthra: ignore-next-line`.
pub const NEXT_LINE: &str = "-next-line";

/// A file's ignore directives: the words they list, and where they are written.
#[derive(Default)]
pub(crate) struct Directives {
    words: FxHashSet<Box<str>>,
    /// Each directive from its start to the end of its line: never prose, whatever it says.
    lines: Vec<(TextSize, TextSize)>,
}

/// A listed word as it is looked up: lower-cased, in the form the packs for its script check it in.
pub(crate) fn listed_form(packs: &Packs, word: &str) -> Box<str> {
    let lower = word.to_lowercase();
    packs
        .normalized(Script::of(&lower), &lower)
        .into_owned()
        .into_boxed_str()
}

impl Directives {
    pub(crate) fn read(text: &str, packs: &Packs) -> Directives {
        let mut directives = Directives::default();
        // Most files have none, and are read past with one search.
        for (at, _) in text.match_indices(IGNORE_DIRECTIVE) {
            let rest = &text[at + IGNORE_DIRECTIVE.len()..];
            let line = rest.split('\n').next().unwrap_or_default();
            let end = at + IGNORE_DIRECTIVE.len() + line.len();
            if let (Ok(start), Ok(end)) = (TextSize::try_from(at), TextSize::try_from(end)) {
                directives.lines.push((start, end));
            }
            // `orthra: ignore-next-line`: the line after it is the directive's too, whole.
            if line.starts_with(NEXT_LINE) {
                let next = (end + 1).min(text.len());
                let next_end = text[next..].find('\n').map_or(text.len(), |at| next + at);
                if let (Ok(start), Ok(end)) =
                    (TextSize::try_from(next), TextSize::try_from(next_end))
                {
                    directives.lines.push((start, end));
                }
                continue;
            }
            directives.words.extend(
                line.split(|c: char| c.is_whitespace() || c == ',')
                    .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
                    .filter(|word| word.chars().any(char::is_alphabetic))
                    .map(|word| listed_form(packs, word)),
            );
        }
        directives
    }

    /// Whether `range` is inside a directive, or the line after an `ignore-next-line`.
    fn covers(&self, range: TextRange) -> bool {
        self.lines
            .iter()
            .any(|(start, end)| *start <= range.start() && range.end() <= *end)
    }
}

/// A word as a pack reads it: `didn\'t`, segmented from a single-quoted string, is `didn't`.
fn unescaped(written: &str) -> Cow<'_, str> {
    if written.contains('\\') {
        written.replace("\\'", "'").replace("\\’", "’").into()
    } else {
        written.into()
    }
}

/// What the packs for `script` say about `word`. A dictionary's acceptance outranks the typo list:
/// a word the dictionary knows is a word, and reporting it would be noise. A rare word, from the
/// larger dictionary, is the other way round: SCOWL's own notes warn that its uncommon words are
/// often misspellings of common ones, so the typo list outranks it. On the typo sample, accepting
/// rare words as dictionary words lost 0.2 and 0.6 points of recall; 137 known typos were among them.
fn judge(packs: &Packs, script: Script, word: &str) -> Verdict {
    let mut asked = false;
    for pack in packs.for_script(script) {
        asked = true;
        if pack.accepts(word) {
            return Verdict::Correct;
        }
    }
    if !asked {
        return Verdict::NotChecked;
    }
    if packs
        .for_script(script)
        .any(|pack| pack.known_typo(word).is_some())
    {
        Verdict::KnownTypo
    } else if packs.for_script(script).any(|pack| pack.accepts_rare(word)) {
        Verdict::Correct
    } else {
        Verdict::Unknown
    }
}
