//! Word interning and the verdict cache.
//!
//! Words repeat: a file has thousands of tokens and a few hundred distinct words, so each distinct
//! word is judged once per epoch. An epoch ends whenever what judges words changes (config, word
//! files, packs), and the whole cache goes with it, so a stale verdict cannot outlive the edit
//! that should have changed it: the bug `cold == warm` exists to catch.

use rustc_hash::FxHashMap;

/// A word, interned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// An enabled pack accepts it.
    Correct,
    /// No pack accepts it, and a pack lists it as a known typo.
    KnownTypo,
    /// No pack accepts it, and none knows it as a typo.
    Unknown,
    /// No enabled pack speaks its script, or it is in two scripts: not a dictionary's question.
    NotChecked,
}

#[derive(Debug, Default)]
pub struct Lexicon {
    ids: FxHashMap<Box<str>, WordId>,
    words: Vec<Box<str>>,
    verdicts: FxHashMap<WordId, Verdict>,
}

impl Lexicon {
    pub fn intern(&mut self, word: &str) -> WordId {
        if let Some(&id) = self.ids.get(word) {
            return id;
        }
        let id =
            WordId(u32::try_from(self.words.len()).expect("fewer than 4 billion distinct words"));
        self.words.push(word.into());
        self.ids.insert(word.into(), id);
        id
    }

    pub fn word(&self, id: WordId) -> &str {
        &self.words[id.0 as usize]
    }

    /// The cached verdict for `id`, or the one `judge` gives, cached.
    pub fn verdict(&mut self, id: WordId, judge: impl FnOnce(&str) -> Verdict) -> Verdict {
        if let Some(&verdict) = self.verdicts.get(&id) {
            return verdict;
        }
        let verdict = judge(&self.words[id.0 as usize]);
        self.verdicts.insert(id, verdict);
        verdict
    }

    /// Forgets every verdict. Interned words stay: a word is the same word in any epoch.
    pub fn bump_epoch(&mut self) {
        self.verdicts.clear();
    }
}
