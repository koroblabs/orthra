//! Rust: comments, doc comments, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "rust",
    language_ids: &["rust"],
    file_names: &[],
    extensions: &["rs"],
    // The query cuts escape sequences out of string spans, and raw strings and comments are read
    // as written: ripgrep's man page writes roff's `\fBauto\fP` in raw strings, where `\f` read
    // as an escape made `Bauto` a word, 113 of them (2026-11-09).
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_rust::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["tests.rs", "*_test.rs", "*_tests.rs"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
