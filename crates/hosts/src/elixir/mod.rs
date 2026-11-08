//! Elixir: comments, the Markdown of `@doc`, `@moduledoc` and `@typedoc`, the text of strings, and
//! the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "elixir",
    language_ids: &["elixir"],
    // `mix.lock` is a tool's file: no host claims it.
    file_names: &[],
    extensions: &["ex", "exs"],
    // The grammar makes every escape and every interpolation of a string a node of its own, so the
    // query cuts them out and a backslash is never left inside a piece.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_elixir::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*_test.exs", "test_helper.exs", "test/"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
