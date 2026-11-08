//! INI: comments, and the text of a value.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "ini",
    language_ids: &["ini"],
    file_names: &[
        ".editorconfig",
        ".gitconfig",
        ".pylintrc",
        ".flake8",
        ".coveragerc",
        ".gitmodules",
        "pylintrc",
        "setup.cfg",
        "tox.ini",
    ],
    extensions: &["ini", "cfg"],
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some(";"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_ini::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: true,
});
