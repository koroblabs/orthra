//! How often each word is written, to put the likeliest correction first.
//!
//! A pack's `frequency` file is `word count` per line, most frequent first, lower case. A
//! dictionary's corrections come in the order of the edits that made them, and the likeliest word
//! is often third or fourth: `сюда` gets `юда, слюда, сода, сюди`. Ordered by how often each is
//! written, `сюди` comes first. Read on the first correction asked for, since only a code action
//! asks for one.

use std::borrow::Cow;

#[cfg(test)]
mod tests;

pub(crate) struct Frequencies {
    text: Cow<'static, str>,
    /// Each word's place in the text and its rank (0 is the most frequent), ordered by the word.
    words: Vec<(u32, u32, u32)>,
}

impl Frequencies {
    pub(crate) fn parse(text: Cow<'static, str>) -> Frequencies {
        let mut words = Vec::new();
        let mut at = 0usize;
        for (rank, line) in text.split_inclusive('\n').enumerate() {
            let start = at;
            at += line.len();
            let word = line
                .trim_end()
                .rsplit_once(' ')
                .map_or("", |(word, _)| word);
            if word.is_empty() {
                continue;
            }
            let (Ok(start), Ok(len), Ok(rank)) = (
                u32::try_from(start),
                u32::try_from(word.len()),
                u32::try_from(rank),
            ) else {
                break;
            };
            words.push((start, len, rank));
        }
        let mut table = Frequencies { text, words };
        let mut words = std::mem::take(&mut table.words);
        // A word listed twice keeps its better rank: the sort puts it first, and the dedup keeps it.
        words.sort_unstable_by(|a, b| table.word(*a).cmp(table.word(*b)).then(a.2.cmp(&b.2)));
        words.dedup_by(|later, earlier| table.word(*later) == table.word(*earlier));
        table.words = words;
        table
    }

    /// The rank of `word`, lower-cased: 0 for the most frequent, `None` for a word not listed.
    pub(crate) fn rank(&self, word: &str) -> Option<u32> {
        let lower = word.to_lowercase();
        self.words
            .binary_search_by(|entry| self.word(*entry).cmp(&lower))
            .ok()
            .map(|at| self.words[at].2)
    }

    fn word(&self, (start, len, _): (u32, u32, u32)) -> &str {
        &self.text[start as usize..(start + len) as usize]
    }
}

impl std::fmt::Debug for Frequencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frequencies")
            .field("words", &self.words.len())
            .finish_non_exhaustive()
    }
}
