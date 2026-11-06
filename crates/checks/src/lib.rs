//! The check registry and every built-in check, one module each.
//!
//! A check reads facts the engine has already computed for the document and never walks the
//! document itself; the cost of a pass is the number of distinct projections, not the number of
//! checks (`docs/01-architecture.md` §5).
//!
//! **One finding per site.** When two checks claim the same range, the one earlier in [`builtin`]
//! wins, and that order is the registry's, not a `match` anybody has to remember to extend.

use std::fmt;

use orthra_hosts::SpanKind;
use orthra_lexicon::Verdict;
use orthra_packs::Packs;
use orthra_segment::Word;
use orthra_text::TextRange;

pub mod known_typo;
pub mod mixed_script;
pub mod unknown_word;
pub mod wrong_layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CheckCode(pub &'static str);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub range: TextRange,
    pub code: CheckCode,
    /// The exact fixes, best first, when the finding knows them. Suggestions are not this: they are
    /// computed for one word, on request (`docs/01-architecture.md` §8).
    pub fixes: Vec<String>,
}

/// What a check may read about one document.
#[derive(Debug, Clone, Copy)]
pub struct DocFacts<'a> {
    pub text: &'a str,
    pub words: &'a [Word],
    /// One per word, in the same order.
    pub verdicts: &'a [Verdict],
    /// The kind of span each word was found in, in the same order.
    pub places: &'a [SpanKind],
    /// Whether the document is a test file, whose string literals are fixtures rather than text
    /// anybody reads: `$ti@scalar@tle` in a phrost test is a marker, not a typo.
    pub test_file: bool,
    pub packs: &'a Packs,
}

impl DocFacts<'_> {
    /// Whether word `i` is code rather than prose: a declared name, an identifier part, code in
    /// backticks, or a value. Code is held to known typos only (`docs/03-roadmap.md` open
    /// question 7).
    pub fn is_code(&self, i: usize) -> bool {
        Self::code(&self.words[i], self.places[i])
    }

    /// [`DocFacts::is_code`], for a word found in a span of kind `place`.
    pub fn code(word: &Word, place: SpanKind) -> bool {
        word.code || matches!(place, SpanKind::Declaration | SpanKind::Value)
    }

    /// Whether word `i` is test data: a string literal in a test file. Not checked at all, not even
    /// for known typos, because a test's data is often a typo on purpose; orthra's own fixtures
    /// spell `recieve`. A comment in a test file is still prose.
    pub fn is_fixture(&self, i: usize) -> bool {
        Self::fixture(self.test_file, self.places[i])
    }

    /// [`DocFacts::is_fixture`], for a word found in a span of kind `place`.
    pub fn fixture(test_file: bool, place: SpanKind) -> bool {
        test_file && matches!(place, SpanKind::String | SpanKind::Value)
    }
}

pub trait Check: Send + Sync + fmt::Debug {
    fn code(&self) -> CheckCode;
    fn run(&self, facts: &DocFacts<'_>, out: &mut Vec<Finding>);
}

/// Every built-in check, in precedence order.
pub fn builtin() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(known_typo::KnownTypo),
        Box::new(mixed_script::MixedScript),
        // Before unknown-word: a word that is another layout's word is that finding at its site.
        Box::new(wrong_layout::WrongLayout),
        Box::new(unknown_word::UnknownWord),
    ]
}
