//! R: comments, roxygen doc comments, the text of strings, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "r",
    language_ids: &["r"],
    file_names: &[".Rprofile", "Rprofile.site"],
    // Extensions are lower-cased before they are compared: this is `.R` as well.
    extensions: &["r"],
    // tree-sitter-r cuts an escape out of the string's content, as Rust's grammar does.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_r::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["tests/", "test_*.R", "test-*.R", "test_*.r", "test-*.r"],
    generated_files: &[],
    // roxygen2: `#' @param name Description`. `@param` is followed by the name it documents, which
    // a typed tag skips. Every tag below `unread` holds names, topics or code.
    doc_tags: DocTags {
        prefixes: &[],
        code: &["examples", "examplesIf", "example", "usage", "rawRd"],
        typed: &["param", "field", "slot"],
        unread: &[
            "aliases",
            "concept",
            "describeIn",
            "docType",
            "eval",
            "evalRd",
            "evalNamespace",
            "export",
            "exportClass",
            "exportMethod",
            "exportPattern",
            "family",
            "import",
            "importClassesFrom",
            "importFrom",
            "importMethodsFrom",
            "include",
            "inherit",
            "inheritDotParams",
            "inheritParams",
            "inheritSection",
            "keywords",
            "method",
            "name",
            "noRd",
            "order",
            "rawNamespace",
            "rdname",
            "S3method",
            "template",
            "templateVar",
            "useDynLib",
        ],
    },
    // `"mpg"`, `"%Y-%m-%d"`, `"data.frame"`: a string of one token is a column, a format or a class.
    one_token_strings_are_values: true,
});
