//! A test double of the engine. It flags words from a fixed typo list and, with the context tier
//! on, flags `their` as a confusable.
//!
//! Each [`Bug`] is one real defect class, planted on purpose, so that each dimension can be shown
//! to catch the defect it exists for and to stay quiet about the others.

use std::collections::HashMap;

use orthra_testkit::fixture::Fixture;
use orthra_text::{TextRange, TextSize};

use crate::subject::{Edit, Observed, Session, Subject};

/// The fixture every harness test shares. The first typo sits past byte 9 on purpose: under a
/// character-offset bug, an earlier one would land inside the neutral line's Cyrillic, and the
/// violation would be about a range no rendering can show.
pub(super) const FIXTURE: &str = r#"
    //- /orthra.toml
    languages = ["en-US"]

    //- /notes.txt
    The cat will recieve their food.
    Teh end: teh.
"#;

const TYPOS: &[&str] = &["teh", "recieve"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Bug {
    None,
    /// Memoizes findings and forgets to drop the memo on an edit.
    StaleAfterEdit,
    /// Reports ranges in characters where bytes are meant. Invisible in ASCII.
    CharOffsets,
    /// Caches a word's range by the word alone, so a second occurrence inherits the first one's.
    CachedByWord,
    /// Rewords a lexical finding when the context tier is on.
    ContextRewords,
}

#[derive(Debug)]
pub(super) struct Toy(pub(super) Bug);

impl Subject for Toy {
    type Session = ToySession;

    fn open(&self, _: &Fixture, text: &str) -> ToySession {
        ToySession {
            bug: self.0,
            text: text.to_owned(),
            context: false,
            memo: None,
            by_word: HashMap::new(),
        }
    }

    fn neutral_line(&self, _: &Fixture) -> Option<String> {
        Some("Привіт 👋\n".to_owned())
    }
}

#[derive(Debug)]
pub(super) struct ToySession {
    bug: Bug,
    text: String,
    context: bool,
    memo: Option<Vec<Observed>>,
    by_word: HashMap<String, TextRange>,
}

impl ToySession {
    fn words(&self) -> Vec<(usize, &str)> {
        let mut words = Vec::new();
        let mut start = None;
        for (i, c) in self.text.char_indices().chain([(self.text.len(), ' ')]) {
            match (start, c.is_alphabetic()) {
                (None, true) => start = Some(i),
                (Some(s), false) => {
                    words.push((s, &self.text[s..i]));
                    start = None;
                }
                _ => {}
            }
        }
        words
    }
}

impl Session for ToySession {
    fn text(&self) -> &str {
        &self.text
    }

    fn edit(&mut self, edit: &Edit) {
        edit.apply(&mut self.text);
        if self.bug != Bug::StaleAfterEdit {
            self.memo = None;
        }
        self.by_word.clear();
    }

    fn findings(&mut self) -> Vec<Observed> {
        if let Some(memo) = &self.memo {
            return memo.clone();
        }
        let mut out = Vec::new();
        let mut first_seen = Vec::new();
        for (start, word) in self.words() {
            let lower = word.to_lowercase();
            let start = match self.bug {
                Bug::CharOffsets => self.text[..start].chars().count(),
                _ => start,
            };
            let mut range = TextRange::at(TextSize::try_from(start).unwrap(), TextSize::of(word));
            if self.bug == Bug::CachedByWord {
                match self.by_word.get(&lower) {
                    Some(&cached) => range = cached,
                    None => first_seen.push((lower.clone(), range)),
                }
            }
            if TYPOS.contains(&lower.as_str()) {
                let label = if self.bug == Bug::ContextRewords && self.context {
                    "known-typo (context agrees)"
                } else {
                    "known-typo"
                };
                out.push(Observed::lexical(range, label));
            }
            if self.context && lower == "their" {
                out.push(Observed::context(range, "confusable"));
            }
        }
        for (word, range) in first_seen {
            self.by_word.entry(word).or_insert(range);
        }
        if self.bug == Bug::StaleAfterEdit {
            self.memo = Some(out.clone());
        }
        out
    }

    fn bump_epoch(&mut self) {
        self.memo = None;
        self.by_word.clear();
    }

    fn set_context(&mut self, on: bool) {
        self.context = on;
        self.memo = None;
    }
}
