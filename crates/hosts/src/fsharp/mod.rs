//! F#.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "fsharp",
    language_ids: &["fsharp"],
    file_names: &[],
    extensions: &["fs", "fsi", "fsx", "fsscript"],
    // A string has its escapes in its text, as the grammar gives it no content node.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("(*", "*)")),
    },
    language: LanguageSource::Constant(tree_sitter_fsharp::LANGUAGE_FSHARP),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*Test.fs", "*Tests.fs", "Tests/"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
