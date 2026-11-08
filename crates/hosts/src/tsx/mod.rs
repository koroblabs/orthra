//! TSX: TypeScript with JSX, whose text between tags is read like a string.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "tsx",
    language_ids: &["typescriptreact", "tsx"],
    file_names: &[],
    extensions: &["tsx"],
    // The query cuts escape sequences out of string spans, and comments have none.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_typescript::LANGUAGE_TSX),
    // TypeScript's query, then what JSX adds to it.
    query: concat!(
        include_str!("../typescript/spell.scm"),
        include_str!("spell.scm")
    ),
    vocabulary: include_str!("../typescript/vocabulary.txt"),
    test_files: crate::typescript::TEST_FILES,
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
