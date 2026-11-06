//! Lines, columns and the encodings LSP counts them in: lspkit's, which phrost's server uses too.
//! The byte-column core is `line_col` and `offset`; a column in an encoding is `line_col_in` and
//! `offset_in`, and a line without its terminator is `line_content`.

pub use lspkit_text::{Encoding, LineCol, LineIndex};
