//! Julia: comments, docstrings, the text of strings, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "julia",
    language_ids: &["julia"],
    file_names: &[],
    extensions: &["jl"],
    // tree-sitter-julia cuts an escape and a `$interpolation` out of the string's content.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: Some(("#=", "=#")),
    },
    language: LanguageSource::Constant(tree_sitter_julia::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["test/", "runtests.jl"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    // `"key"`, `"%d"`, `"data.csv"`: a string of one token is a name, a format or a path.
    one_token_strings_are_values: true,
});
