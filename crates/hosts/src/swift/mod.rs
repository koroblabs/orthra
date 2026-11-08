//! Swift: comments, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "swift",
    language_ids: &["swift"],
    file_names: &[],
    // `Package.swift` is covered by the extension.
    extensions: &["swift"],
    // The grammar cuts a string's escapes (`\n`, `\u{1F600}`, `\(value)`) out of its text, so the
    // segmenter never sees a backslash.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_swift::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*Tests.swift", "*Test.swift", "Tests/"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
