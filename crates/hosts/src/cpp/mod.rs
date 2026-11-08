//! C++: comments, Doxygen, the text of strings and raw strings, and the names a file declares.

use crate::{
    Conventions,
    c::{DOC_TAGS, TEST_FILES},
    tree_sitter_host::{Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "cpp",
    language_ids: &["cpp"],
    file_names: &[],
    // `.h` is C's (`c/mod.rs`): a C++ header opened in the editor arrives with the id `cpp`.
    extensions: &[
        "cpp", "cc", "cxx", "c++", "hpp", "hh", "hxx", "h++", "ipp", "inl", "tpp",
    ],
    // The query cuts escape sequences out of a string, and a raw string has none.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_cpp::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: TEST_FILES,
    generated_files: &[],
    doc_tags: DOC_TAGS,
    one_token_strings_are_values: true,
});
