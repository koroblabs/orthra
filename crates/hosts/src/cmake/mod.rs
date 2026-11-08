//! CMake: comments, the messages `message()` prints, the documentation of an option or a cache
//! variable, and the names a project declares.
//!
//! Every other argument is a keyword, a target, a path or a flag, and is somebody else's.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "cmake",
    language_ids: &["cmake"],
    file_names: &["CMakeLists.txt"],
    extensions: &["cmake"],
    // A quoted argument reads `\n`, `\t`, `\"` and `\;`.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: Some(("#[[", "]]")),
    },
    language: LanguageSource::Constant(tree_sitter_cmake::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
