//! Zig: comments, `///` and `//!` doc comments, the text of string literals, and the names a file
//! declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "zig",
    language_ids: &["zig"],
    file_names: &[],
    // `.zon` is Zig Object Notation, the build manifest's format: the same grammar reads it.
    extensions: &["zig", "zon"],
    // The grammar cuts a string's escapes (`\n`, `\x41`, `\u{1F600}`) out of its text.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_zig::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
