//! HTML: the text of a page, its comments, and the attributes that hold text for a reader.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

// A template in a page: Angular's, Vue's, Jinja's, Django's, Handlebars' and Go's `{{ … }}`,
// Jinja's, Django's, Twig's and Liquid's `{% … %}` and `{# … #}`, ERB's and EJS's `<% … %>`. What
// is between them is the template language's code, not the page's words.
pub(crate) static HOST: TreeSitterHost = TreeSitterHost::cutting(
    Grammar {
        id: "html",
        language_ids: &["html"],
        file_names: &[],
        // Zed's HTML also claims `shtml`.
        extensions: &["html", "htm", "xhtml", "shtml"],
        // An entity (`&amp;`, `&#8212;`) is its own node in the grammar and is cut out of the text,
        // so no letter glues onto a word. HTML has no backslash escapes.
        conventions: Conventions {
            backslash_escapes: false,
            doubled_escapes: false,
            line_comment: None,
            block_comment: Some(("<!--", "-->")),
        },
        language: LanguageSource::Constant(tree_sitter_html::LANGUAGE),
        query: include_str!("spell.scm"),
        vocabulary: include_str!("vocabulary.txt"),
        test_files: &[],
        generated_files: &[],
        doc_tags: DocTags::NONE,
        one_token_strings_are_values: false,
    },
    &[("{{", "}}"), ("{%", "%}"), ("{#", "#}"), ("<%", "%>")],
);
