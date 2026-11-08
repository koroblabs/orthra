//! Dockerfiles: comments, the prose a `LABEL` carries, and the names a build declares.
//!
//! `RUN`, `CMD` and `ENTRYPOINT` are shell and are not checked. The parser directives on the first
//! lines (`# syntax=`, `# escape=`) are settings, not comments.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "dockerfile",
    language_ids: &["dockerfile"],
    file_names: &[
        "Dockerfile",
        "Containerfile",
        "Dockerfile.*",
        "Containerfile.*",
        "*.Dockerfile",
        "*.dockerfile",
    ],
    extensions: &["dockerfile"],
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: None,
    },
    // This grammar publishes a `language()` function and no `LANGUAGE` constant.
    language: LanguageSource::Function(tree_sitter_dockerfile_updated::language),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
