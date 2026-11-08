//! Elm: comments, doc comments, the text of strings, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "elm",
    language_ids: &["elm"],
    file_names: &[],
    extensions: &["elm"],
    // The grammar cuts a string's escapes (`\n`, `\u{1F600}`) out of its parts.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("--"),
        block_comment: Some(("{-", "-}")),
    },
    language: LanguageSource::Constant(tree_sitter_elm::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["tests/", "*Test.elm", "*Tests.elm"],
    generated_files: &[],
    // `@docs view, update` lists names, and is the one tag a module's doc comment has.
    doc_tags: DocTags {
        prefixes: &[],
        code: &[],
        typed: &[],
        unread: &["docs"],
    },
    one_token_strings_are_values: false,
});
