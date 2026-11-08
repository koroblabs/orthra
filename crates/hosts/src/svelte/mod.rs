//! Svelte components: the markup's text, its comments and the attributes that hold text for a
//! reader.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

// `{expr}`, `{#if}`, `{:else}` and `{/if}` are Svelte's code, not the page's words.
pub(crate) static HOST: TreeSitterHost = TreeSitterHost::cutting(
    Grammar {
        id: "svelte",
        language_ids: &["svelte"],
        file_names: &[],
        extensions: &["svelte"],
        conventions: Conventions {
            backslash_escapes: false,
            doubled_escapes: false,
            line_comment: None,
            block_comment: Some(("<!--", "-->")),
        },
        language: LanguageSource::Constant(tree_sitter_svelte_ng::LANGUAGE),
        query: include_str!("spell.scm"),
        // A component's template is HTML, with HTML's words: the classic lorem ipsum.
        vocabulary: include_str!("../html/vocabulary.txt"),
        test_files: &[],
        generated_files: &[],
        doc_tags: DocTags::NONE,
        one_token_strings_are_values: false,
    },
    &[("{", "}")],
);
