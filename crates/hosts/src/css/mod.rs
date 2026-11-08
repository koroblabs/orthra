//! CSS: comments, the text of `content:`, and the names a stylesheet declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "css",
    language_ids: &["css"],
    file_names: &[],
    extensions: &["css"],
    // The grammar cuts a string's escapes (`\201C`, `\f101`, `\a`) out of its content, and a class
    // name's (`.sm\:flex`) out of its pieces, so the segmenter never sees a backslash.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: None,
        block_comment: Some(("/*", "*/")),
    },
    language: tree_sitter_css::LANGUAGE,
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
