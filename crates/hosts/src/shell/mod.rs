//! Shell: comments, the text of strings, and the names a script declares.
//!
//! The bash grammar serves `sh`, `bash`, `zsh` and `ksh`, as far as it parses them.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "shell",
    language_ids: &["shellscript"],
    file_names: &[
        ".bashrc",
        ".bash_profile",
        ".bash_logout",
        ".bash_aliases",
        ".profile",
        ".zshrc",
        ".zprofile",
        ".zshenv",
        ".zlogin",
        ".zlogout",
        ".envrc",
        ".kshrc",
        "PKGBUILD",
    ],
    extensions: &["sh", "bash", "zsh", "ksh"],
    // `printf 'a\nb'` and `echo -e "a\tb"` read their escapes, and so does `$'...'`. A double-quoted
    // string keeps its backslash in the content, so segmentation has to know `\n` is not a letter.
    conventions: Conventions {
        backslash_escapes: true,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: tree_sitter_bash::LANGUAGE,
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*.bats", "test_*.sh", "*_test.sh"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    // `"frtc-probe"`, `"uptime-kuma"`, `"--release"`, the `"Hello "` before a `$name`: a string of one
    // token is a name, a path or a flag, and a message has spaces.
    one_token_strings_are_values: true,
});
