//! The pipeline: a document becomes findings.
//!
//! ```text
//! text ─► host ─► spans ─► segment ─► words ─► verdicts (cached) ─► checks ─► findings
//! ```
//!
//! This is the only API the server sees, the way `phrost-view` is the only one phrost's features
//! see (`docs/01-architecture.md` §1). Suggestions are not computed here: they are for one word,
//! on request, and cost hundreds of times what a check does.

use std::{borrow::Cow, fmt, sync::Arc};

use judge::{Directives, Judge};
use named_lines::NameLabels;
use orthra_checks::{Check, DocFacts, Strings};
use orthra_hosts::{
    Host, Hosts, Span, SpanKind,
    test_files::{is_generated, is_test_path},
};
use orthra_lexicon::{Lexicon, Verdict};
use orthra_packs::{Packs, locale_path};
use orthra_segment::{Rules, Word};
use orthra_text::{TextSize, script::Script};
use rustc_hash::{FxHashMap, FxHashSet};

pub use judge::{IGNORE_DIRECTIVE, NEXT_LINE, Why};
pub use kept::Parsed;
pub use orthra_checks::unknown_word::Silenced;
pub use orthra_checks::{CheckCode, Finding};
/// The context tier's contract, for the server to hold a scorer by: the engine is the only crate
/// the server sees.
pub use orthra_context as context;

mod judge;
mod kept;
mod named_lines;
mod passage;
#[cfg(test)]
mod tests;
mod tool_output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError(pub String);

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EngineError {}

pub struct Engine {
    packs: Arc<Packs>,
    hosts: Arc<Hosts>,
    checks: Arc<[Box<dyn Check>]>,
    word_internal: Vec<(char, Script)>,
    name_labels: NameLabels,
    lexicon: Lexicon,
    /// What the project declares, lower-cased: its manifests' names and the names its files declare,
    /// as learned through [`Engine::declare_name`] and [`Engine::learn`]. A document's own
    /// declarations are not kept here; they are read again on every check, so an edit that renames
    /// a declaration cannot leave the old name accepted.
    /// Shared with this engine's forks, and copied only when one of them learns more.
    declared: Arc<FxHashSet<Box<str>>>,
    /// Each host's vocabulary, lower-cased: `async` and `serde` are words in a Rust file.
    host_words: Arc<FxHashMap<&'static str, FxHashSet<Box<str>>>>,
    /// The words files' words, lower-cased: the user's and the project's. Somebody wrote each one
    /// down on purpose, so it is a word everywhere, a declaration and a known typo included.
    words: FxHashSet<Box<str>>,
    /// The languages enabled whose packs are to be installed: see [`Engine::with_installed`].
    missing: Vec<String>,
}

impl Engine {
    /// An engine with the built-in packs for `languages`: `["en-US"]`.
    pub fn new(languages: &[&str]) -> Result<Engine, EngineError> {
        let packs = Packs::builtin(languages).map_err(|err| EngineError(err.0))?;
        Ok(Engine::with_packs(Arc::new(packs)))
    }

    /// An engine with the packs for `languages`, built in or installed under `packs`. A language
    /// whose pack is installed and not there yet checks nothing until it is, and is named by
    /// [`Engine::missing_packs`]; a language with no pack at all is an error.
    pub fn with_installed(
        languages: &[&str],
        packs: Option<&std::path::Path>,
    ) -> Result<Engine, EngineError> {
        let (loaded, missing) = Packs::load(languages, packs).map_err(|err| EngineError(err.0))?;
        let mut engine = Engine::with_packs(Arc::new(loaded));
        engine.missing = missing;
        Ok(engine)
    }

    /// The languages enabled whose packs are installed rather than built in, and not installed yet.
    pub fn missing_packs(&self) -> &[String] {
        &self.missing
    }

    /// An engine sharing packs already loaded. A pack is read once per process; an engine per
    /// document is cheap.
    pub fn with_packs(packs: Arc<Packs>) -> Engine {
        let hosts = Hosts::builtin();
        let host_words = hosts
            .iter()
            .map(|host| {
                let words = host
                    .vocabulary()
                    .lines()
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(|word| word.to_lowercase().into_boxed_str())
                    .collect();
                (host.id(), words)
            })
            .collect();
        Engine {
            word_internal: packs.word_internal(),
            name_labels: NameLabels::new(packs.name_labels(), packs.name_lines()),
            packs,
            host_words: Arc::new(host_words),
            hosts: Arc::new(hosts),
            checks: orthra_checks::builtin().into(),
            lexicon: Lexicon::default(),
            declared: Arc::default(),
            words: FxHashSet::default(),
            missing: Vec::new(),
        }
    }

    /// The words files' words, in place of any given before: a words file that changed is read
    /// again whole. They are checked before the verdict cache, so nothing cached goes stale.
    pub fn set_words<'w>(&mut self, words: impl IntoIterator<Item = &'w str>) {
        let packs = &self.packs;
        self.words = words
            .into_iter()
            .map(|word| judge::listed_form(packs, word))
            .collect();
    }

    /// The conventions of the host for `language_id`: its escapes, its line comment.
    pub fn conventions(&self, language_id: &str) -> Option<orthra_hosts::Conventions> {
        self.hosts
            .for_language(language_id)
            .map(|host| host.conventions())
    }

    /// The language id of the host that claims `path`, if one does.
    pub fn language_for_path(&self, path: &str) -> Option<&'static str> {
        self.hosts.for_path(path).map(|host| host.language_ids()[0])
    }

    /// The findings for the file at `path`: its host, and whether it is a test file, both read off
    /// the path. A path no host claims has none.
    pub fn check_file(&mut self, path: &str, text: &str) -> Vec<Finding> {
        let Some(host) = self.hosts.for_path(path) else {
            return Vec::new();
        };
        let (language, strings) = (host.language_ids()[0], self.strings(host, path));
        self.check_as(language, text, strings, None)
    }

    /// What the strings of the file at `path`, read by `host`, are: a test file's are test data, a
    /// translation's into a language no enabled pack speaks are foreign, and any other's are text.
    /// Read off the path alone, as which host reads it is.
    fn strings(&self, host: &dyn Host, path: &str) -> Strings {
        if is_test_path(host, path) {
            return Strings::TestData;
        }
        let spoken = |language: &str| {
            let tags = self.packs.iter().map(|pack| pack.tag());
            tags.chain(self.missing.iter().map(String::as_str))
                .any(|tag| locale_path::language(tag).eq_ignore_ascii_case(language))
        };
        match locale_path::named_language(path) {
            Some(language) if !spoken(language) => Strings::Foreign,
            _ => Strings::Text,
        }
    }

    /// The findings for a document an editor has open: read as `language_id` when a host speaks
    /// it, and by its path when none does, so a buffer the user set to Markdown is read as
    /// Markdown. Whether it is a test file is read off the path either way. Which host reads it is
    /// a lookup, made once: nothing is retried with another.
    ///
    /// `parsed` is what the last check of this document kept, told of every edit since: a
    /// keystroke is parsed again where it changed. The findings are the same without it.
    pub fn check_document(
        &mut self,
        path: &str,
        language_id: &str,
        text: &str,
        parsed: &mut Parsed,
    ) -> Vec<Finding> {
        let Some(host) = self
            .hosts
            .for_language(language_id)
            .or_else(|| self.hosts.for_path(path))
        else {
            return Vec::new();
        };
        // An open minified bundle is a tool's output, as it is on disk.
        if is_generated(host, path) {
            return Vec::new();
        }
        let (language, strings) = (host.language_ids()[0], self.strings(host, path));
        self.check_as(language, text, strings, Some(parsed))
    }

    /// How a file of this kind writes a comment, for a directive to be written in: `("//", "")`,
    /// `("<!--", " -->")`. `None` for a format with no comments, such as plain text.
    pub fn comment_syntax(
        &self,
        path: &str,
        language_id: &str,
    ) -> Option<(&'static str, &'static str)> {
        let host = self
            .hosts
            .for_language(language_id)
            .or_else(|| self.hosts.for_path(path))?;
        let conventions = host.conventions();
        conventions
            .line_comment
            .map(|line| (line, ""))
            .or(conventions.block_comment)
    }

    /// An engine with this one's packs and hosts, and nothing of the project: what a project is
    /// learned into off the request path, before it takes this one's place.
    pub fn sibling(&self) -> Engine {
        Engine {
            packs: Arc::clone(&self.packs),
            hosts: Arc::clone(&self.hosts),
            checks: Arc::clone(&self.checks),
            word_internal: self.word_internal.clone(),
            name_labels: self.name_labels.clone(),
            lexicon: Lexicon::default(),
            declared: Arc::default(),
            host_words: Arc::clone(&self.host_words),
            words: self.words.clone(),
            missing: self.missing.clone(),
        }
    }

    /// An engine that knows what this one has learned, to check with off the request path while
    /// this one answers: the same packs, words and declarations, and a verdict cache of its own.
    pub fn fork(&self) -> Engine {
        let mut fork = self.sibling();
        fork.declared = Arc::clone(&self.declared);
        fork
    }

    /// The findings for `text` read as `language_id`, by range. A language no host speaks has none.
    pub fn check(&mut self, language_id: &str, text: &str) -> Vec<Finding> {
        self.check_as(language_id, text, Strings::Text, None)
    }

    /// [`Engine::check`] of a document whose last check kept `parsed`.
    pub fn check_reusing(
        &mut self,
        language_id: &str,
        text: &str,
        parsed: &mut Parsed,
    ) -> Vec<Finding> {
        self.check_as(language_id, text, Strings::Text, Some(parsed))
    }

    fn check_as(
        &mut self,
        language_id: &str,
        text: &str,
        strings: Strings,
        parsed: Option<&mut Parsed>,
    ) -> Vec<Finding> {
        let Some((host, words, places)) = self.words(language_id, text, parsed) else {
            return Vec::new();
        };
        let local = judge::local_declarations(text, &words, &places);
        let directives = Directives::read(text, &self.packs);
        let judge = Judge {
            packs: &self.packs,
            words_files: &self.words,
            directives: &directives,
            host_words: self.host_words.get(host),
            local: &local,
            declared: &self.declared,
        };
        let lexicon = &mut self.lexicon;
        let verdicts: Vec<Verdict> = words
            .iter()
            .zip(&places)
            .map(|(word, place)| judge.verdict(lexicon, text, word, *place).0)
            .collect();
        let packs = &self.packs;

        let facts = DocFacts {
            text,
            words: &words,
            verdicts: &verdicts,
            places: &places,
            strings,
            packs,
        };
        let mut findings = Vec::new();
        for check in self.checks.iter() {
            check.run(&facts, &mut findings);
        }
        // One finding per site: a stable sort keeps registry order within a range, and the first
        // finding at a range is the one the earliest check made.
        findings.sort_by_key(|finding| (finding.range.start(), finding.range.end()));
        findings.dedup_by(|later, earlier| later.range == earlier.range);
        findings
    }

    /// Learns what `text` declares, as part of the project: the names its declarations hold become
    /// words in every other document. Call it for every file of the project before checking, and
    /// again when a file changes.
    pub fn learn(&mut self, language_id: &str, text: &str) {
        let mut declared = Declared(std::mem::take(Arc::make_mut(&mut self.declared)));
        self.declarations(language_id, text, &mut declared);
        *Arc::make_mut(&mut self.declared) = declared.0;
    }

    /// What `text` declares, added to `into`: what [`Engine::learn`] learns from it, read with a
    /// shared engine, so that a project's files can be read on several threads at once and what
    /// each declares handed to [`Engine::declare`] after.
    pub fn declarations(&self, language_id: &str, text: &str, into: &mut Declared) {
        let Some((_, words, places)) = self.words(language_id, text, None) else {
            return;
        };
        for (word, place) in words.iter().zip(&places) {
            if *place != SpanKind::Declaration {
                continue;
            }
            // Lower-cased as a check looks it up. A name is declared over and over, and kept once.
            let written = &text[word.range];
            let lower: Cow<'_, str> = if written.chars().any(char::is_uppercase) {
                written.to_lowercase().into()
            } else {
                Cow::Borrowed(written)
            };
            if !into.0.contains(&*lower) {
                into.0.insert(lower.into_owned().into_boxed_str());
            }
        }
    }

    /// Learns what [`Engine::declarations`] read.
    pub fn declare(&mut self, declared: Declared) {
        let into = Arc::make_mut(&mut self.declared);
        if into.is_empty() {
            *into = declared.0;
        } else {
            into.extend(declared.0);
        }
    }

    /// Declares a name from the project's manifest: its package or a dependency. `tower-lsp-server`
    /// declares `tower`, `lsp` and `server`; `phrost` declares `phrost`.
    pub fn declare_name(&mut self, name: &str) {
        for part in name
            .split(|c: char| !c.is_alphanumeric())
            .filter(|part| part.chars().count() >= 3)
        {
            Arc::make_mut(&mut self.declared).insert(part.to_lowercase().into_boxed_str());
        }
    }

    /// The words of `text` and the kind of span each was found in.
    fn words(
        &self,
        language_id: &str,
        text: &str,
        parsed: Option<&mut Parsed>,
    ) -> Option<(&'static str, Vec<Word>, Vec<SpanKind>)> {
        let host = self.hosts.for_language(language_id)?;
        if tool_output::written_by_a_tool(text) {
            return None;
        }
        let mut spans = Vec::new();
        let rules = Rules {
            word_internal: &self.word_internal,
            backslash_escapes: host.conventions().backslash_escapes,
            doubled_escapes: host.conventions().doubled_escapes,
        };
        if let Some(parsed) = parsed {
            host.spans_reusing(text, &mut parsed.host, &mut spans);
            let (mut words, mut places) = parsed.words(text, &spans, host.id(), &rules);
            self.name_labels
                .drop_named_lines(text, &mut words, &mut places);
            return Some((host.id(), words, places));
        }
        host.spans(text, &mut spans);
        let mut words: Vec<Word> = Vec::new();
        orthra_segment::words(
            text,
            spans.iter().map(|span| span.range),
            &rules,
            &mut words,
        );
        // Spans and words are both in order, so one walk gives each word its span.
        let mut places = Vec::with_capacity(words.len());
        let mut at = 0;
        for word in &words {
            while at + 1 < spans.len() && spans[at].range.end() <= word.range.start() {
                at += 1;
            }
            places.push(spans.get(at).map_or(SpanKind::Prose, |span| span.kind));
        }
        self.name_labels
            .drop_named_lines(text, &mut words, &mut places);
        Some((host.id(), words, places))
    }

    /// Everything the engine decided about the word at `at` in the file at `path`, for `orthra
    /// explain`: which span holds it, how it was segmented, what each pack said, the verdict, the
    /// finding, and suggestions. The finding is [`Engine::check_file`]'s, so a test file reads as
    /// one here too. `None` when no host claims the path.
    pub fn explain(&mut self, path: &str, text: &str, at: TextSize) -> Option<Explanation> {
        let host = self.hosts.for_path(path)?;
        let (language_id, strings) = (host.language_ids()[0], self.strings(host, path));
        let mut spans = Vec::new();
        host.spans(text, &mut spans);
        let span = spans
            .iter()
            .find(|span| span.range.contains_inclusive(at))
            .copied();
        let mut explanation = Explanation {
            host: host.id(),
            span,
            word: None,
        };
        if span.is_none() {
            return Some(explanation);
        }
        // The words as a check reads them, so that the verdict below is the check's.
        let (_, words, places) = self.words(language_id, text, None)?;
        let Some(i) = words
            .iter()
            .position(|word| word.range.contains_inclusive(at))
        else {
            return Some(explanation);
        };
        let (word, place) = (words[i], places[i]);
        let written = &text[word.range];
        let answers = self
            .packs
            .iter()
            .filter(|pack| pack.script() == word.script)
            .map(|pack| PackAnswer {
                tag: pack.tag().to_owned(),
                accepts: pack.accepts(written),
                accepts_rare: pack.accepts_rare(written),
                known_typo: pack
                    .known_typo(written)
                    .map(|fixes| fixes.iter().map(|f| f.to_string()).collect()),
            })
            .collect();
        let local = judge::local_declarations(text, &words, &places);
        let directives = Directives::read(text, &self.packs);
        let (verdict, why) = Judge {
            packs: &self.packs,
            words_files: &self.words,
            directives: &directives,
            host_words: self.host_words.get(host.id()),
            local: &local,
            declared: &self.declared,
        }
        .verdict(&mut self.lexicon, text, &word, place);
        let silenced = match verdict {
            Verdict::Unknown => Silenced::of(text, &word, place, strings),
            // A known typo is reported anywhere but in test data, which may misspell on purpose,
            // and in a translation's strings, which are in another language.
            Verdict::KnownTypo if DocFacts::fixture(strings, place) => {
                Silenced::of_strings(strings)
            }
            Verdict::KnownTypo | Verdict::Correct | Verdict::NotChecked => None,
        };
        let finding = self
            .check_as(language_id, text, strings, None)
            .into_iter()
            .find(|finding| finding.range == word.range);
        let suggestions = if matches!(verdict, Verdict::Unknown | Verdict::KnownTypo) {
            self.suggest(written, 5)
        } else {
            Vec::new()
        };
        explanation.word = Some(WordExplained {
            word,
            text: written.to_owned(),
            answers,
            verdict,
            why,
            silenced,
            finding,
            suggestions,
        });
        Some(explanation)
    }

    /// Reads every enabled pack's dictionaries now. A check reads a pack's on the first word of its
    /// script; a measurement of what loading costs asks for it here.
    pub fn load_packs(&self) {
        self.packs.read_dictionaries();
    }

    /// Forgets every verdict, as a change to config, word files or packs must.
    pub fn bump_epoch(&mut self) {
        self.lexicon.bump_epoch();
    }

    /// Corrections for one word, best first. For a code action, never for diagnostics.
    pub fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(pack) = self.packs.for_script(Script::of(word)).next() {
            pack.suggest(word, limit, &mut out);
        }
        out
    }

    /// The sentence at `at` in a document an editor has open, for a model to rewrite when somebody
    /// asks (`docs/04-context.md` §2 C): in text meant to be read -- prose, a comment, a doc
    /// comment -- and never in code, a string, a name or a value. Read by the host
    /// [`Engine::check_document`] reads it with, through what its last check kept, so that a
    /// document just checked is not parsed again. For a code action, never for diagnostics.
    pub fn sentence_at(
        &self,
        path: &str,
        language_id: &str,
        text: &str,
        at: TextSize,
        parsed: &mut Parsed,
    ) -> Option<orthra_text::TextRange> {
        let host = self
            .hosts
            .for_language(language_id)
            .or_else(|| self.hosts.for_path(path))?;
        if tool_output::written_by_a_tool(text) {
            return None;
        }
        let mut spans = Vec::new();
        host.spans_reusing(text, &mut parsed.host, &mut spans);
        let passage = passage::around(text, &spans, at)?;
        orthra_context::sentence::around(text, passage, at)
    }

    /// The tag of the enabled language `text` is written in, as far as its letters tell: the first
    /// pack for the script most of them are in. `None` when no pack speaks it.
    pub fn language_of(&self, text: &str) -> Option<&str> {
        let mut letters: Vec<(Script, usize)> = Vec::new();
        for script in text.chars().filter_map(Script::of_char) {
            match letters.iter_mut().find(|(seen, _)| *seen == script) {
                Some((_, count)) => *count += 1,
                None => letters.push((script, 1)),
            }
        }
        let (script, _) = letters.into_iter().max_by_key(|&(_, count)| count)?;
        self.packs.for_script(script).next().map(|pack| pack.tag())
    }
}

/// Names a project's files declare, lower-cased, as [`Engine::declarations`] reads them.
#[derive(Debug, Default)]
pub struct Declared(FxHashSet<Box<str>>);

/// What `orthra explain` prints.
#[derive(Debug)]
pub struct Explanation {
    pub host: &'static str,
    /// The span holding the position, if any: outside every span is not prose.
    pub span: Option<Span>,
    /// The word at the position, if segmentation made one there.
    pub word: Option<WordExplained>,
}

#[derive(Debug)]
pub struct WordExplained {
    pub word: Word,
    pub text: String,
    /// What each pack for the word's script said.
    pub answers: Vec<PackAnswer>,
    pub verdict: Verdict,
    /// The rule that gave the verdict.
    pub why: Why,
    /// Why an unknown word or a known typo makes no finding, when it makes none.
    pub silenced: Option<Silenced>,
    pub finding: Option<Finding>,
    pub suggestions: Vec<String>,
}

#[derive(Debug)]
pub struct PackAnswer {
    pub tag: String,
    pub accepts: bool,
    /// Whether its rare words do, which counts only where no pack knows the word as a typo.
    pub accepts_rare: bool,
    pub known_typo: Option<Vec<String>>,
}

impl fmt::Debug for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Engine")
            .field("packs", &self.packs)
            .finish_non_exhaustive()
    }
}
