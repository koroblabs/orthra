//! GraphQL: comments, descriptions, and the names a schema or an operation declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "graphql",
    language_ids: &["graphql", "gql"],
    file_names: &[],
    extensions: &["graphql", "gql", "graphqls"],
    // A description's `\n` and `\"` are escapes; a block string's body has none but `\"""`.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_graphql::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
