//! Erlang: comments, edoc, the `-doc` attributes, the text of strings, and the names a file
//! declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "erlang",
    language_ids: &["erlang"],
    // Erlang terms, as Zed's Erlang extension reads them: rebar's settings, an application's
    // resource file, `erl -make`'s.
    file_names: &["rebar.config", "*.app.src", "Emakefile"],
    extensions: &["erl", "hrl", "escript"],
    // A string's escapes stay inside its node, so segmentation has to know `"\n"` is one.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("%"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_erlang::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*_SUITE.erl", "*_tests.erl", "test/"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
