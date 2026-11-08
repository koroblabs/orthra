//! Go: comments, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "go",
    language_ids: &["go"],
    file_names: &[],
    extensions: &["go"],
    // The query cuts escape sequences out of interpreted strings, and raw strings have none.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_go::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*_test.go", "testdata/"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
