//! Scala: comments and Scaladoc, the text of strings, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::cutting(
    Grammar {
        id: "scala",
        language_ids: &["scala"],
        file_names: &[],
        // `build.sbt` and `.sc` scripts are Scala too.
        extensions: &["scala", "sc", "sbt", "mill"],
        // A string is captured whole, as the grammar gives no node for the text between its
        // escapes, so the segmenter must know `\n` and `é` are not letters. A comment has them
        // in the examples a doc gives of a string.
        conventions: Conventions {
            backslash_escapes: true,
            doubled_escapes: false,
            line_comment: Some("//"),
            block_comment: Some(("/*", "*/")),
        },
        language: LanguageSource::Constant(tree_sitter_scala::LANGUAGE),
        query: include_str!("spell.scm"),
        vocabulary: include_str!("vocabulary.txt"),
        // sbt, Maven and Gradle keep tests in `src/test/`; ScalaTest and specs2 name a suite
        // `*Spec`, `*Suite` or `*Test`, and munit `*Suite`.
        test_files: &[
            "*Spec.scala",
            "*Suite.scala",
            "*Test.scala",
            "*Tests.scala",
            "src/test/",
        ],
        generated_files: &[],
        // Scaladoc: `@param name text`, `@tparam T text`, `@throws Type text`. A name or a type is
        // code. `@example` holds code, and the rest hold references and versions.
        doc_tags: DocTags {
            typed: &["param", "tparam", "throws", "constructor"],
            unread: &[
                "author",
                "define",
                "example",
                "group",
                "groupdesc",
                "groupname",
                "groupprio",
                "inheritdoc",
                "see",
                "since",
                "usecase",
                "version",
            ],
            ..DocTags::NONE
        },
        // `"org.typelevel"`, `"-Xfatal-warnings"`, `"2.13.12"`: a string of one token is a
        // coordinate, a flag or a key far more often than a word. A known typo is still reported.
        one_token_strings_are_values: true,
    },
    // `${expr}` in `s"..."`, `f"..."` and `raw"..."` is code. A query cannot cut inside a string
    // node, and the grammar gives the text between interpolations no node of its own.
    &[("${", "}")],
);
