//! `unknown-word`: no enabled pack for the word's script accepts it, and none knows it as a typo.
//! The weakest claim orthra makes, and reported as the weakest: in prose only. In code an unknown
//! word is usually a name somebody chose (`rdr`, `fnid`); on the zero corpus those were most of
//! codebook's noise in identifiers (`docs/02-testing.md` level 9).

use orthra_hosts::SpanKind;
use orthra_lexicon::Verdict;
use orthra_segment::Word;

use crate::{Check, CheckCode, DocFacts, Finding, Strings};

/// Why an unknown word makes no finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Silenced {
    /// It is code, held to known typos only.
    Code,
    /// It is a string in a test file: test data, not checked at all.
    TestData,
    /// It is a string in a translation into a language no pack speaks.
    Foreign,
    /// It has fewer than four letters.
    Short,
}

impl Silenced {
    /// Why the unknown word `word`, in a span of kind `place`, is not reported, or `None` when it
    /// is.
    pub fn of(text: &str, word: &Word, place: SpanKind, strings: Strings) -> Option<Silenced> {
        // Three-letter unknowns are abbreviations far more often than typos (`dep`, `yml`, `uid`),
        // as cspell's default minimum of four letters agrees; a three-letter known typo (`teh`) is
        // still reported, by known-typo.
        let letters = text[word.range]
            .chars()
            .filter(|c| c.is_alphabetic())
            .count();
        if DocFacts::code(word, place) {
            Some(Silenced::Code)
        } else if DocFacts::fixture(strings, place) {
            Silenced::of_strings(strings)
        } else if letters < 4 {
            Some(Silenced::Short)
        } else {
            None
        }
    }

    /// Why a word in a string that is not text is not reported: `None` for text.
    pub fn of_strings(strings: Strings) -> Option<Silenced> {
        match strings {
            Strings::Text => None,
            Strings::TestData => Some(Silenced::TestData),
            Strings::Foreign => Some(Silenced::Foreign),
        }
    }
}

#[derive(Debug)]
pub struct UnknownWord;

impl Check for UnknownWord {
    fn code(&self) -> CheckCode {
        CheckCode("unknown-word")
    }

    fn run(&self, facts: &DocFacts<'_>, out: &mut Vec<Finding>) {
        for (i, (word, verdict)) in facts.words.iter().zip(facts.verdicts).enumerate() {
            if *verdict == Verdict::Unknown
                && Silenced::of(facts.text, word, facts.places[i], facts.strings).is_none()
            {
                out.push(Finding {
                    range: word.range,
                    run: word.run,
                    code: self.code(),
                    fixes: Vec::new(),
                });
            }
        }
    }
}
