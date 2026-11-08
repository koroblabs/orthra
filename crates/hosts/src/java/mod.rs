//! Java: comments and Javadoc, the text of strings and text blocks, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "java",
    language_ids: &["java"],
    file_names: &[],
    extensions: &["java"],
    // The query cuts escape sequences out of strings and text blocks. A comment has them too, in
    // the examples a doc gives of a string: `"jump\nto the website"` is not `nto`.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_java::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    // Surefire runs `*Test`, `*Tests` and `*TestCase`, failsafe `*IT`; Maven keeps them in
    // `src/test/`.
    test_files: &["*Test.java", "*Tests.java", "*IT.java", "src/test/"],
    generated_files: &[],
    doc_tags: DocTags {
        prefixes: &[],
        // `@param name`, `@throws Type`: the first token is a name or a type, then prose.
        typed: &["exception", "param", "throws"],
        // Names, references and versions.
        unread: &[
            "author",
            "hidden",
            "see",
            "serial",
            "serialField",
            "since",
            "version",
        ],
    },
    // `"jdk.httpclient.allowRestrictedHeaders"`, `"dd'T'HH"`: a string of one token is a key, a
    // pattern or a table entry far more often than a word. A known typo in it is still reported.
    one_token_strings_are_values: true,
});
