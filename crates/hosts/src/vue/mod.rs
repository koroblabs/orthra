//! Vue single-file components: the template's text, its comments and the attributes that hold text
//! for a reader.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

// `{{ … }}` is its own node in the grammar (`interpolation`), so it never reaches a `text`.
pub(crate) static HOST: TreeSitterHost = TreeSitterHost::cutting(
    Grammar {
        id: "vue",
        language_ids: &["vue"],
        file_names: &[],
        extensions: &["vue"],
        // An entity is its own node and is cut out of the text. A template has no backslash
        // escapes.
        conventions: Conventions {
            backslash_escapes: false,
            doubled_escapes: false,
            line_comment: None,
            block_comment: Some(("<!--", "-->")),
        },
        language: LanguageSource::Constant(tree_sitter_vue_next::LANGUAGE),
        query: include_str!("spell.scm"),
        // A component's template is HTML, with HTML's words: the classic lorem ipsum.
        vocabulary: include_str!("../html/vocabulary.txt"),
        test_files: &[],
        generated_files: &[],
        doc_tags: DocTags::NONE,
        one_token_strings_are_values: false,
    },
    &[("{{", "}}")],
);
