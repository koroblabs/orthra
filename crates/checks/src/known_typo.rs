//! `known-typo`: a word a pack lists as a known misspelling, with its exact fix, in the case the
//! word was written in: `Seperate` → `Separate`, `RECIEVE` → `RECEIVE`.

use orthra_lexicon::Verdict;
use orthra_segment::Shape;

use crate::{Check, CheckCode, DocFacts, Finding};

#[derive(Debug)]
pub struct KnownTypo;

impl Check for KnownTypo {
    fn code(&self) -> CheckCode {
        CheckCode("known-typo")
    }

    fn run(&self, facts: &DocFacts<'_>, out: &mut Vec<Finding>) {
        for (i, (word, verdict)) in facts.words.iter().zip(facts.verdicts).enumerate() {
            if *verdict != Verdict::KnownTypo || facts.is_fixture(i) {
                continue;
            }
            let text = &facts.text[word.range];
            let Some(fixes) = facts
                .packs
                .for_script(word.script)
                .find_map(|pack| pack.known_typo(text))
            else {
                continue;
            };
            out.push(Finding {
                range: word.range,
                code: self.code(),
                fixes: fixes
                    .iter()
                    .map(|fix| in_case_of(fix, word.shape))
                    .collect(),
            });
        }
    }
}

fn in_case_of(fix: &str, shape: Shape) -> String {
    match shape {
        Shape::Upper => fix.to_uppercase(),
        Shape::Title => {
            let mut chars = fix.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect())
                .unwrap_or_default()
        }
        Shape::Lower | Shape::Mixed => fix.to_owned(),
    }
}
