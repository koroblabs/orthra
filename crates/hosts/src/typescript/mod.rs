//! TypeScript: comments, JSDoc, the text of strings and templates, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

/// Jest, Vitest and Mocha name their tests so; TSX shares them.
pub(crate) const TEST_FILES: &[&str] = &["*.test.*", "*.spec.*", "__tests__/"];

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "typescript",
    language_ids: &["typescript"],
    file_names: &[],
    extensions: &["ts", "mts", "cts"],
    // The query cuts escape sequences out of string spans, and comments have none.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_typescript::LANGUAGE_TYPESCRIPT),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: TEST_FILES,
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
