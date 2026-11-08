//! Nix: comments, the text of strings, and the names an expression declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "nix",
    language_ids: &["nix"],
    file_names: &[],
    extensions: &["nix"],
    // A string's `\n` is an escape, and the grammar cuts it out of the fragments, as it does an
    // indented string's `''\n` and `''$`.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_nix::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    // What bundix, node2nix and cabal2nix write.
    generated_files: &[
        "gemset.nix",
        "generated.nix",
        "hackage-packages.nix",
        "node-env.nix",
        "node-packages.nix",
    ],
    doc_tags: DocTags::NONE,
    // `"https://..."`, `"sha256-..."`, `"x86_64-linux"`, the `"-v"` after a `${version}`: a string
    // of one token is a name, a URL or a hash, and a message has spaces.
    one_token_strings_are_values: true,
});
