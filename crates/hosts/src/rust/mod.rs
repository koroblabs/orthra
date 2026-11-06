//! Rust: comments, doc comments, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "rust",
    language_ids: &["rust"],
    file_names: &[],
    extensions: &["rs"],
    // The query cuts escape sequences out of string spans, and raw strings and comments have none:
    // `r"C:\Users"` is not `\U` followed by `sers`.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: tree_sitter_rust::LANGUAGE,
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["tests.rs", "*_test.rs", "*_tests.rs"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
