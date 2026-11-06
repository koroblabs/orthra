//! Positions in a document, and the writing system of a word — and nothing else.
//!
//! Every range in orthra is a byte range into UTF-8 text. LSP counts columns in UTF-16 unless the
//! client agrees otherwise, and that conversion happens in [`line_index`] and nowhere else, so a
//! column bug has exactly one place to live.

pub mod line_index;
pub mod script;

pub use text_size::{TextRange, TextSize};
