//! Groovy and Gradle's build scripts: comments and Groovydoc, the text of strings, and the names
//! a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::cutting(
    Grammar {
        id: "groovy",
        language_ids: &["groovy"],
        // Jenkins reads `Jenkinsfile` as Groovy, and Gradle's Groovy build scripts end in `.gradle`.
        file_names: &["Jenkinsfile"],
        extensions: &["groovy", "gradle"],
        // The grammar is Java's: a double-quoted string has escape nodes between its pieces, but
        // a single-quoted one is a leaf, so the segmenter must know `\n` is not a letter.
        conventions: Conventions {
            backslash_escapes: true,
            doubled_escapes: false,
            line_comment: Some("//"),
            block_comment: Some(("/*", "*/")),
        },
        language: LanguageSource::Constant(tree_sitter_groovy::LANGUAGE),
        query: include_str!("spell.scm"),
        vocabulary: include_str!("vocabulary.txt"),
        // Spock specs end in `Spec`; Gradle and Maven keep tests in `src/test/`.
        test_files: &[
            "*Spec.groovy",
            "*Test.groovy",
            "*Tests.groovy",
            "src/test/",
            "src/integTest/",
            "src/functionalTest/",
        ],
        generated_files: &[],
        // Groovydoc is Javadoc: `@param name text`, `@throws Type text`.
        doc_tags: DocTags {
            prefixes: &[],
            code: &[],
            typed: &["exception", "param", "throws"],
            unread: &["author", "see", "since", "version"],
        },
        // A Gradle build script is coordinates and plugin ids: `'com.squareup.okio:okio:3.0.0'`,
        // `'java-library'`. A known typo is still reported.
        one_token_strings_are_values: true,
    },
    // `${expr}` in a double-quoted GString is code, and the grammar does not parse it.
    &[("${", "}")],
);
