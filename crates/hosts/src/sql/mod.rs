//! SQL: comments, and the names a script declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "sql",
    language_ids: &["sql"],
    file_names: &[],
    extensions: &["sql"],
    // A backslash is an escape in MySQL's strings and in Postgres's `E'...'`; in standard SQL it
    // is a letter, and one followed by letters is rare there.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("--"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_sequel::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: true,
});
