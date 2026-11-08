//! A pack's `rare-words`: sorted lists of words that are accepted and never offered, searched by
//! halving the text itself where it is embedded. No index and no heap: a lookup reads the lines it
//! halves to, seventeen or so of en-US's 91 000. An offset per entry, as the vocabulary keeps for
//! its corrections, cost about 2 MB of RSS between the offsets and the text read to make them
//! (2026-11-08).
//!
//! Each file is sorted by the word in lower case, one spelling of each, as the vocabulary is
//! (`tools/packs.py` writes both), and case is read the same way: see [`crate::vocabulary`].

use std::cmp::Ordering;

use crate::vocabulary::{capitals, key};

#[cfg(test)]
mod tests;

/// One sorted file, without its comment lines.
#[derive(Debug)]
pub(crate) struct WordList {
    text: &'static str,
}

impl WordList {
    /// The list in `text`: one entry per line, after any `#` lines at the top.
    pub(crate) fn new(text: &'static str) -> WordList {
        let header: usize = text
            .split_inclusive('\n')
            .take_while(|line| line.starts_with('#') || line.trim().is_empty())
            .map(str::len)
            .sum();
        let list = WordList {
            text: &text[header..],
        };
        debug_assert!(
            list.text
                .lines()
                .zip(list.text.lines().skip(1))
                .all(|(a, b)| key(a.trim_end()).lt(key(b.trim_end()))),
            "a word list sorted by the word in lower case, one spelling of each"
        );
        list
    }

    /// Whether `word` is in the list: in any case, or in capitals for an entry in capitals.
    pub(crate) fn accepts(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.find(&lower)
            .is_some_and(|entry| !capitals(entry) || entry == word)
    }

    /// The entry whose lower case is `lower`. Every bound is the start of a line, so the line
    /// around the middle starts inside them.
    fn find(&self, lower: &str) -> Option<&'static str> {
        let (text, bytes) = (self.text, self.text.as_bytes());
        let (mut low, mut high) = (0, text.len());
        while low < high {
            let middle = low + (high - low) / 2;
            let start = bytes[..middle]
                .iter()
                .rposition(|&b| b == b'\n')
                .map_or(0, |at| at + 1);
            let end = bytes[middle..]
                .iter()
                .position(|&b| b == b'\n')
                .map_or(text.len(), |at| middle + at);
            let entry = text[start..end].trim_end();
            match key(entry).cmp(lower.chars()) {
                Ordering::Equal => return Some(entry),
                Ordering::Less => low = end + 1,
                Ordering::Greater => high = start,
            }
        }
        None
    }
}
