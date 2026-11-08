//! JavaScript and JSX: comments, JSDoc, the text of strings, templates and JSX, and the names a
//! file declares. The grammar parses JSX, so one host serves `.js` and `.jsx` alike.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "javascript",
    language_ids: &["javascript", "javascriptreact"],
    file_names: &[],
    extensions: &["js", "mjs", "cjs", "jsx"],
    // The query cuts escape sequences out of string spans, and comments have none.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_javascript::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    // Jest, Vitest and Mocha name their tests as TypeScript's do.
    test_files: crate::typescript::TEST_FILES,
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
