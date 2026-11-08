//! SCSS: comments, the text of `content:`, and the names a stylesheet declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "scss",
    language_ids: &["scss"],
    file_names: &[],
    extensions: &["scss"],
    // As in CSS, a backslash in a string or a class name is a hex code point (`\f101`) or an
    // escaped punctuation mark (`.sm\:flex`), never a letter escape like `\n`: treating `\a` as one
    // would cut the first letter off the word after it.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    // This grammar was last released against tree-sitter 0.21 and has no `LANGUAGE` constant.
    language: LanguageSource::Function(tree_sitter_scss::language),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
