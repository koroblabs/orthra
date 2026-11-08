//! C: comments, Doxygen, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

/// Doxygen's tags, shared with C++: `@param name`, `@tparam T` and `@throws Error` are followed by
/// the name of what they describe, which the reader treats as a type and skips. `@see`, `@ref` and
/// the file-level tags hold a reference, a name or a date, not prose.
pub(crate) const DOC_TAGS: DocTags = DocTags {
    prefixes: &[],
    typed: &["param", "tparam", "throws", "throw", "exception", "retval"],
    unread: &[
        "author",
        "copydoc",
        "date",
        "file",
        "headerfile",
        "link",
        "ref",
        "sa",
        "see",
        "since",
        "version",
    ],
};

/// The files a C or C++ project keeps its tests in: `foo_test.c`, `foo_unittest.cc`, `test/`.
pub(crate) const TEST_FILES: &[&str] = &[
    "*_test.c",
    "*_test.cc",
    "*_test.cpp",
    "*_test.cxx",
    "*_unittest.cc",
    "*_unittest.cpp",
    "test/",
    "tests/",
];

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "c",
    language_ids: &["c"],
    file_names: &[],
    // `.h` is both languages' header, and a path cannot say which. It goes to C, whose headers
    // most `.h` files are; an open file carries the editor's language id, which decides.
    extensions: &["c", "h"],
    // The query cuts escape sequences out of a string, and a comment has none.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_c::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: TEST_FILES,
    generated_files: &[],
    doc_tags: DOC_TAGS,
    one_token_strings_are_values: true,
});
