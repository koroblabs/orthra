//! What a host keeps of a document between two checks of it, so that a keystroke is parsed again
//! where it changed and not from the start.
//!
//! A tree-sitter host keeps its tree. Every edit to the text is told to the tree, in order, before
//! the next check parses with it: tree-sitter then reuses whatever the edits did not touch. A tree
//! is handed back only to the host that made it. One whose edits do not add up to the text it is
//! checked with means an edit was not told: a bug, which fails every test build, and which a
//! release build pays for with one full parse rather than a wrong span.
//!
//! A host that parses whole every time keeps nothing here.

use std::fmt;

use orthra_text::TextRange;
use tree_sitter::{InputEdit, Point, Tree};

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct Parsed {
    pub(crate) kept: Option<Kept>,
}

pub(crate) struct Kept {
    /// The host that made the tree.
    pub(crate) host: &'static str,
    pub(crate) tree: Tree,
    /// The length of the text the tree describes, once the edits told since are applied.
    pub(crate) len: usize,
    /// What the query captured in each of the tree's nodes it was run over whole, by node:
    /// see `tree_sitter_host`'s `Loaded::capture_reusing`.
    pub(crate) captured: rustc_hash::FxHashMap<usize, Box<[crate::tree_sitter_host::Captured]>>,
}

impl Parsed {
    /// `range` of `before`, the text as it is now, is about to become `inserted`.
    pub fn edit(&mut self, before: &str, range: TextRange, inserted: &str) {
        let Some(kept) = &mut self.kept else {
            return;
        };
        let (start, old_end) = (usize::from(range.start()), usize::from(range.end()));
        let Some(replaced) = before.get(start..old_end) else {
            self.kept = None;
            return;
        };
        let start_position = point_at(before, start);
        kept.tree.edit(&InputEdit {
            start_byte: start,
            old_end_byte: old_end,
            new_end_byte: start + inserted.len(),
            start_position,
            old_end_position: advance(start_position, replaced),
            new_end_position: advance(start_position, inserted),
        });
        kept.len = kept.len - replaced.len() + inserted.len();
    }

    /// The text is about to be replaced whole: nothing of the old tree is worth keeping.
    pub fn clear(&mut self) {
        self.kept = None;
    }

    /// What `host` kept, if its tree describes a text of `len` bytes.
    pub(crate) fn take(&mut self, host: &'static str, len: usize) -> Option<Kept> {
        let kept = self.kept.take().filter(|kept| kept.host == host)?;
        debug_assert_eq!(
            kept.len, len,
            "the text changed by an edit its tree was not told of"
        );
        (kept.len == len).then_some(kept)
    }
}

impl fmt::Debug for Parsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parsed")
            .field("host", &self.kept.as_ref().map(|kept| kept.host))
            .finish_non_exhaustive()
    }
}

/// The row and byte column of `at` in `text`, as tree-sitter counts them: rows end at `\n`.
fn point_at(text: &str, at: usize) -> Point {
    let before = &text.as_bytes()[..at];
    let row = before.iter().filter(|&&byte| byte == b'\n').count();
    let column = at
        - before
            .iter()
            .rposition(|&byte| byte == b'\n')
            .map_or(0, |i| i + 1);
    Point { row, column }
}

/// Where `from` ends up after `text` is written at it.
fn advance(from: Point, text: &str) -> Point {
    match text.as_bytes().iter().rposition(|&byte| byte == b'\n') {
        None => Point {
            row: from.row,
            column: from.column + text.len(),
        },
        Some(last) => Point {
            row: from.row + text.bytes().filter(|&byte| byte == b'\n').count(),
            column: text.len() - last - 1,
        },
    }
}
