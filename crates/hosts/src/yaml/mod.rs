//! YAML: comments, the text of strings, and keys.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "yaml",
    language_ids: &["yaml"],
    file_names: &[],
    extensions: &["yaml", "yml"],
    // A double-quoted string's `\n` is an escape. Elsewhere a backslash is most often a Windows
    // path, which is not words either way.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_yaml::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    // `ubuntu-latest`, `"3.12"`, `stable`: YAML's strings are values, and its text has spaces.
    one_token_strings_are_values: true,
});
