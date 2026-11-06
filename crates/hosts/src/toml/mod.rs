//! TOML: comments, the text of strings, and keys.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "toml",
    language_ids: &["toml"],
    file_names: &[],
    extensions: &["toml"],
    // A basic string's `\n` is an escape. A literal string has none, and a backslash in it is most
    // often a Windows path, which is not words either way.
    conventions: Conventions {
        backslash_escapes: true,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: tree_sitter_toml_ng::LANGUAGE,
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    // `"derive"`, `"source.ocaml"`, `"1.2"`: TOML's strings are values, and its text has spaces.
    one_token_strings_are_values: true,
});
