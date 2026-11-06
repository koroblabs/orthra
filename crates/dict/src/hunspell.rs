//! A Hunspell dictionary, read as it is: an `.aff` and a `.dic`, through spellbook.

use std::fmt;

use crate::{Candidate, DictError, Dictionary, edits};

#[cfg(test)]
mod tests;

pub struct HunspellDict {
    inner: spellbook::Dictionary,
}

impl HunspellDict {
    pub fn new(aff: &str, dic: &str) -> Result<HunspellDict, DictError> {
        spellbook::Dictionary::new(aff, dic)
            .map(|inner| HunspellDict { inner })
            .map_err(|err| {
                DictError(format!(
                    "a Hunspell dictionary that does not parse: {err:?}"
                ))
            })
    }
}

impl Dictionary for HunspellDict {
    fn check(&self, word: &str) -> bool {
        self.inner.check(word)
    }

    /// Edit-based suggestions only, without spellbook's n-gram pass.
    ///
    /// The n-gram pass walks the whole word list. Measured on 2026-11-05 (`orthra perf` on the zero
    /// corpus's Markdown, `just typo-orthra` on 2 000 real typos):
    ///
    /// | | suggest p95 | top-1 code / prose | top-5 code / prose |
    /// | --- | ---: | ---: | ---: |
    /// | with n-grams | 25.2 ms | 73.8 / 68.6 % | 80.9 / 73.8 % |
    /// | edits only | 0.55 ms | 72.8 / 67.6 % | 79.0 / 72.7 % |
    ///
    /// Forty-five times faster, and inside the 15 ms budget with room to spare, for about a point of
    /// accuracy. It is still ahead of codebook's 63.3 / 59.1 % top-1. The n-gram pass only runs
    /// when edits find nothing, so trying it second costs the slow case and not the common one; it
    /// was measured that way too: 19.6 ms p95, still over budget.
    fn suggest(&self, word: &str, out: &mut Vec<Candidate>) {
        let mut words = Vec::new();
        self.inner
            .suggester()
            .with_ngram_suggestions(false)
            .suggest(word, &mut words);
        out.extend(words.into_iter().map(|correction| Candidate {
            edits: edits::between(word, &correction),
            word: correction,
        }));
    }
}

impl fmt::Debug for HunspellDict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HunspellDict").finish_non_exhaustive()
    }
}
