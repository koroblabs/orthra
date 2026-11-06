//! PHP: comments and docblocks, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "php",
    language_ids: &["php"],
    file_names: &[],
    extensions: &["php", "phtml"],
    // The query cuts escape sequences out of strings. A backslash anywhere else separates the
    // parts of a namespaced name, `App\Models\User`, and is not an escape.
    conventions: Conventions {
        backslash_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    // The grammar that reads a whole file: the HTML around `<?php` as well as the PHP in it.
    language: tree_sitter_php::LANGUAGE_PHP,
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    // PHPUnit's convention; Pest and Codeception keep theirs under `tests/`.
    test_files: &["*Test.php"],
    generated_files: &[],
    doc_tags: DocTags {
        prefixes: &["phpstan-", "psalm-", "phan-"],
        typed: &[
            "assert",
            "assert-if-false",
            "assert-if-true",
            "extends",
            "implements",
            "mixin",
            "param",
            "param-out",
            "property",
            "property-read",
            "property-write",
            "require-extends",
            "require-implements",
            "return",
            "self-out",
            "template",
            "template-contravariant",
            "template-covariant",
            "this-out",
            "throws",
            "type",
            "use",
            "var",
        ],
        // Names, versions and references, and `@method`, which is a signature. PHPUnit's
        // annotations name tests, groups and classes.
        unread: &[
            "author",
            "category",
            "copyright",
            "covers",
            "coversDefaultClass",
            "dataProvider",
            "depends",
            "example",
            "filesource",
            "group",
            "import-type",
            "license",
            "link",
            "method",
            "package",
            "requires",
            "see",
            "since",
            "source",
            "subpackage",
            "ticket",
            "used-by",
            "uses",
            "version",
        ],
    },
    one_token_strings_are_values: false,
});
