//! A pack's vocabulary: words accepted beside its dictionary, `kubectl`, `PostgreSQL`, `rustup`.
//!
//! Read where it is embedded, never copied. Each file is sorted by the word in lower case, one
//! spelling of each (`tools/packs.py` writes it so); a word is looked up by halving over the
//! entries' offsets. en-US's 26 838 words cost 107 KB of offsets this way. A hash set of owned
//! words would cost about 1.5 MB of heap (2026-11-06).
//!
//! Case is ignored -- `Kubectl` at the start of a sentence is still `kubectl` -- except for an
//! entry in capitals, which is matched only by a word in capitals: `INOUT` is SQL, and `inout` in
//! prose is `input` mistyped.

use std::sync::OnceLock;

#[cfg(test)]
mod tests;

/// The vocabulary files of a pack, each sorted.
#[derive(Debug, Default)]
pub(crate) struct Vocabulary {
    lists: Vec<Sorted>,
}

/// One file, and where each of its entries starts.
#[derive(Debug)]
struct Sorted {
    text: &'static str,
    starts: Vec<u32>,
    /// Each entry's [`Sketch`], in file order, made on the first correction asked for.
    sketches: OnceLock<Vec<Sketch>>,
}

/// What an entry's distance from a word can be bounded by without reading the entry: which
/// letters it has, one bit each, and its length. An edit changes the length by one at most and
/// the letters by two bits at most (a substitution takes one away and brings one), so an entry
/// whose sketch is further than that from the word's is further in edits too.
///
/// Going through en-US's 38 895 entries by edits alone cost 3.1 ms a word at the 95th percentile;
/// an FST of them searched with the dictionary's automaton, 0.97 ms, and 1.6 to 2.4 MB of RSS
/// from what building it freed. Skipping by sketch costs 5 bytes an entry (2026-11-06).
#[derive(Clone, Copy, Debug)]
struct Sketch {
    letters: u32,
    length: u8,
}

impl Vocabulary {
    /// The vocabulary of `texts`, each a file of one entry per line, `#` lines being comments.
    pub(crate) fn new(texts: &[&'static str]) -> Vocabulary {
        Vocabulary {
            lists: texts.iter().map(|text| Sorted::new(text)).collect(),
        }
    }

    /// The entries within `edits` of `lower`, a word in lower case, each in its own spelling and
    /// with its distance as [`orthra_dict::edits::between`] counts it. An entry in more than one
    /// file comes once from each.
    pub(crate) fn within(&self, lower: &str, edits: u32, out: &mut Vec<(&'static str, u32)>) {
        let word: Vec<char> = lower.chars().collect();
        let wanted = Sketch::of(word.iter().copied());
        let (mut entry, mut rows) = (Vec::new(), Vec::new());
        for list in &self.lists {
            for (at, sketch) in list.sketches().iter().enumerate() {
                if !sketch.may_be_within(wanted, edits) {
                    continue;
                }
                let found = list.entry(at);
                entry.clear();
                entry.extend(key(found));
                if let Some(distance) = orthra_dict::edits::within(&word, &entry, edits, &mut rows)
                {
                    out.push((found, distance));
                }
            }
        }
    }

    /// Whether `word` is in it: in any case, or in capitals for an entry written in capitals.
    pub(crate) fn accepts(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.lists.iter().any(|list| {
            list.find(&lower)
                .is_some_and(|entry| !capitals(entry) || entry == word)
        })
    }
}

impl Sorted {
    fn new(text: &'static str) -> Sorted {
        let mut starts = Vec::new();
        let mut at = 0usize;
        for line in text.split_inclusive('\n') {
            let entry = line.trim_end();
            if !entry.is_empty() && !entry.starts_with('#') {
                starts.push(u32::try_from(at).unwrap_or(u32::MAX));
            }
            at += line.len();
        }
        let sorted = Sorted {
            text,
            starts,
            sketches: OnceLock::new(),
        };
        debug_assert!(
            (1..sorted.starts.len()).all(|at| key(sorted.entry(at - 1)).lt(key(sorted.entry(at)))),
            "a vocabulary file sorted by the word in lower case, one spelling of each"
        );
        sorted
    }

    fn sketches(&self) -> &[Sketch] {
        self.sketches.get_or_init(|| {
            (0..self.starts.len())
                .map(|at| Sketch::of(key(self.entry(at))))
                .collect()
        })
    }

    fn entry(&self, at: usize) -> &'static str {
        let start = self.starts[at] as usize;
        let rest = &self.text[start..];
        rest[..rest.find('\n').unwrap_or(rest.len())].trim_end()
    }

    /// The entry whose lower case is `lower`.
    fn find(&self, lower: &str) -> Option<&'static str> {
        self.starts
            .binary_search_by(|start| {
                let rest = &self.text[*start as usize..];
                let entry = rest[..rest.find('\n').unwrap_or(rest.len())].trim_end();
                key(entry).cmp(lower.chars())
            })
            .ok()
            .map(|at| self.entry(at))
    }
}

impl Sketch {
    /// The sketch of a word in lower case. `a` to `z` have a bit each; any other letter shares one
    /// of six, which only lets more entries through to be measured.
    fn of(word: impl Iterator<Item = char>) -> Sketch {
        let (mut letters, mut length) = (0u32, 0u8);
        for c in word {
            letters |= match c {
                'a'..='z' => 1 << (c as u32 - 'a' as u32),
                _ => 1 << (26 + c as u32 % 6),
            };
            length = length.saturating_add(1);
        }
        Sketch { letters, length }
    }

    /// Whether an entry of this sketch can be within `edits` of a word of the `wanted` one.
    fn may_be_within(self, wanted: Sketch, edits: u32) -> bool {
        u32::from(self.length.abs_diff(wanted.length)) <= edits
            && (self.letters ^ wanted.letters).count_ones() <= 2 * edits
    }
}

/// An entry as it is sorted: in lower case.
fn key(entry: &str) -> impl Iterator<Item = char> + '_ {
    entry.chars().flat_map(char::to_lowercase)
}

/// Whether `entry` is written in capitals: two letters or more, none of them lower case.
fn capitals(entry: &str) -> bool {
    entry.chars().filter(|c| c.is_alphabetic()).count() > 1
        && !entry.chars().any(char::is_lowercase)
}
