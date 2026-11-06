//! Known misspellings, each with its fixes: a pack's `typos.tsv`.
//!
//! The signal with the fewest false positives there is, after crate-ci/typos: a word is reported
//! only because somebody has seen it written wrong before, and its fix is exact.
//!
//! The list is read where it is embedded, never copied: an entry is where its line starts, and a
//! lookup is a binary search over those. As a hash map of owned strings, en-US's 96 000 entries
//! were 9 MB of heap, three allocations each, most of the server's memory before its first file
//! (measured 2026-11-05); as offsets they are 4 bytes each.

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub struct KnownTypos {
    /// The list as written: `typo<TAB>fix[<TAB>fix…]` per line.
    text: &'static str,
    /// Where each entry's line starts, in the order of its typo. A typo is lower case, so the
    /// order is its bytes'.
    entries: Box<[u32]>,
}

/// A known typo's fixes, best first, in the case the list writes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fixes<'t>(&'t str);

impl<'t> Fixes<'t> {
    pub fn iter(self) -> impl Iterator<Item = &'t str> {
        self.0.split('\t')
    }
}

impl KnownTypos {
    /// `typo<TAB>fix[<TAB>fix…]` per line; `#` starts a comment line, and a line with no fix is
    /// not an entry. A typo must be written in lower case, and listed once: an entry that breaks
    /// either would never be found, and saying so beats a list that is quietly shorter than it
    /// looks. Lines in any order are sorted here; the lists orthra ships are sorted already.
    pub fn parse(tsv: &'static str) -> Result<KnownTypos, String> {
        let mut entries = Vec::new();
        let mut start = 0;
        for line in tsv.split_inclusive('\n') {
            let at = start;
            start += line.len();
            let line = line.trim_end_matches(['\n', '\r']);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((typo, fixes)) = line.split_once('\t') else {
                continue;
            };
            if fixes.is_empty() {
                continue;
            }
            if typo.chars().any(char::is_uppercase) {
                return Err(format!("{typo:?} is not written in lower case"));
            }
            entries.push(u32::try_from(at).map_err(|_| "the list is over 4 GB".to_owned())?);
        }
        let mut typos = KnownTypos {
            text: tsv,
            entries: entries.into(),
        };
        if !typos.entries.is_sorted_by_key(|&at| typos.typo(at)) {
            let mut entries = std::mem::take(&mut typos.entries);
            entries.sort_unstable_by_key(|&at| typos.typo(at));
            typos.entries = entries;
        }
        if let Some(twice) = typos
            .entries
            .windows(2)
            .find(|pair| typos.typo(pair[0]) == typos.typo(pair[1]))
        {
            return Err(format!("{:?} is listed twice", typos.typo(twice[0])));
        }
        Ok(typos)
    }

    /// The fixes for `word`, whatever its case.
    pub fn get(&self, word: &str) -> Option<Fixes<'static>> {
        let lower;
        let word = if word.chars().any(char::is_uppercase) {
            lower = word.to_lowercase();
            &lower
        } else {
            word
        };
        let found = self
            .entries
            .binary_search_by(|&at| self.typo(at).cmp(word))
            .ok()?;
        let line = self.line(self.entries[found]);
        line.split_once('\t').map(|(_, fixes)| Fixes(fixes))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entry whose line starts at `at`, without its line break.
    fn line(&self, at: u32) -> &'static str {
        let rest = &self.text[at as usize..];
        let end = rest.find('\n').unwrap_or(rest.len());
        rest[..end].trim_end_matches('\r')
    }

    /// The typo of the entry at `at`. Every entry's line holds a tab, so the first one ends it: a
    /// binary search reads a few bytes of each line it passes, not the line.
    fn typo(&self, at: u32) -> &'static str {
        let rest = &self.text[at as usize..];
        &rest[..rest.find('\t').unwrap_or(rest.len())]
    }
}
