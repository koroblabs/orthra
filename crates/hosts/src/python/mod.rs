//! Python: comments, docstrings, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "python",
    language_ids: &["python"],
    file_names: &[],
    extensions: &["py", "pyi"],
    // tree-sitter-python keeps an escape *inside* the string's content, so the query cannot cut it
    // out, as Rust's does: segmentation has to know `"\n"` is an escape.
    conventions: Conventions {
        backslash_escapes: true,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: tree_sitter_python::LANGUAGE,
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["test_*.py", "*_test.py", "conftest.py"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
