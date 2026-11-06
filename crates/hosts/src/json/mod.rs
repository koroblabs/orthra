//! JSON and JSONC: comments, the text of strings, and keys.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "json",
    language_ids: &["json", "jsonc"],
    file_names: &[],
    extensions: &["json", "jsonc"],
    // A string is read whole, so `\n` is an escape. JSON has no comments, but the grammar reads
    // JSONC's, and a `//` line is what the `shift` dimension prepends.
    conventions: Conventions {
        backslash_escapes: true,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: tree_sitter_json::LANGUAGE,
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &["package-lock.json", "npm-shrinkwrap.json"],
    doc_tags: DocTags::NONE,
    // `"^1.0"`, `"commonjs"`, `"es2022"`: JSON's strings are values, and its text has spaces.
    one_token_strings_are_values: true,
});
