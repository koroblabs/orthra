//! How often words are written, as each word's rank: an FST map from a word to its place among the
//! counts, 0 for the most written. A pack puts its corrections in that order among the ones as
//! near.
//!
//! A pack's frequency file is `word count` per line, most written first, in lower case, and words
//! of one count are equals: English's counts are SCOWL's sizes, a class each. A built-in pack's is
//! compiled when orthra is and read where it is embedded, which costs nothing until a correction
//! is asked for and then only the pages read. Parsed into a sorted table on the first correction
//! instead, English's 93 000 words took 20 ms of that correction's 24 on the slow VM, and 1.1 MB of
//! heap (2026-11-09). An installed pack's is compiled on the first correction asked for.

use std::fmt;

use fst::Map;

use crate::{DictError, compiled::Bytes};

#[cfg(test)]
mod tests;

/// The FST type a rank table is written with, so that neither a dictionary's forms nor its marks
/// read as one.
const FORMAT: u64 = 0x5241_4e4b;

pub struct Ranks {
    map: Map<Bytes>,
}

impl Ranks {
    /// The table for `text`, `word count` per line, most written first. The rank goes up when the
    /// count changes, and a word listed twice keeps its better rank.
    pub fn compile(text: &str) -> Result<Vec<u8>, DictError> {
        let mut words: Vec<(&str, u64)> = Vec::new();
        let (mut rank, mut count) = (0, None);
        for line in text.lines() {
            let Some((word, written)) = line.trim_end().rsplit_once(' ') else {
                continue;
            };
            if word.is_empty() {
                continue;
            }
            if count.is_some_and(|count| count != written) {
                rank += 1;
            }
            count = Some(written);
            words.push((word, rank));
        }
        // By word, and a word's better rank first, which the dedup keeps.
        words.sort_unstable();
        words.dedup_by(|later, earlier| later.0 == earlier.0);
        let failed = |err: fst::Error| DictError(format!("the frequencies do not compile: {err}"));
        let mut builder = fst::raw::Builder::new_type(Vec::new(), FORMAT).map_err(failed)?;
        for (word, rank) in words {
            builder.insert(word, rank).map_err(failed)?;
        }
        builder.into_inner().map_err(failed)
    }

    /// The table [`Ranks::compile`] wrote.
    pub fn new(bytes: Bytes) -> Result<Ranks, DictError> {
        let fst = fst::raw::Fst::new(bytes)
            .map_err(|err| DictError(format!("a frequency table that does not read: {err}")))?;
        if fst.fst_type() != FORMAT {
            return Err(DictError(format!(
                "a frequency table of format {}, and this orthra reads {FORMAT}",
                fst.fst_type()
            )));
        }
        Ok(Ranks {
            map: Map::from(fst),
        })
    }

    /// The rank of `word`, lower-cased: 0 for the most written, `None` for a word not listed.
    pub fn rank(&self, word: &str) -> Option<u32> {
        self.map
            .get(word.to_lowercase())
            .and_then(|rank| u32::try_from(rank).ok())
    }
}

impl fmt::Debug for Ranks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ranks")
            .field("words", &self.map.len())
            .finish()
    }
}
