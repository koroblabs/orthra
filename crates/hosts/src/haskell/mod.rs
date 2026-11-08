//! Haskell: comments, Haddock, the text of strings, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "haskell",
    language_ids: &["haskell"],
    file_names: &[],
    // `.lhs` is left out: a literate file is prose with code in `>` lines, and the grammar reads it
    // as code.
    extensions: &["hs"],
    // The grammar keeps a string's escapes (`\n`, `\1234`, `\&`) in its text, so the segmenter has
    // to know them.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("--"),
        block_comment: Some(("{-", "-}")),
    },
    language: LanguageSource::Constant(tree_sitter_haskell::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["test/", "tests/", "*Spec.hs", "*Test.hs", "*Tests.hs"],
    generated_files: &[],
    // Haddock has `@since` and nothing else at the start of a line.
    doc_tags: DocTags {
        prefixes: &[],
        code: &[],
        typed: &[],
        unread: &["since"],
    },
    one_token_strings_are_values: true,
});
