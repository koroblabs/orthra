//! Kotlin: comments, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "kotlin",
    language_ids: &["kotlin"],
    file_names: &[],
    // `.kts` is Gradle's build scripts and Kotlin scripts: the same grammar.
    extensions: &["kt", "kts"],
    // The query cuts escape sequences out of strings (`\n`, `é`, `\$`), and a raw string has
    // none. `$name` and `${...}` are nodes of their own, so no word is glued to them.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_kotlin_ng::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[
        "*Test.kt",
        "*Tests.kt",
        "src/test/",
        "src/androidTest/",
        "src/commonTest/",
        "src/jvmTest/",
    ],
    generated_files: &[],
    // KDoc: `@param name text`. The name is declared where the parameter is, so it is skipped
    // the way a type is in the other languages' doc tags; `@see` and `@author` hold a reference or a
    // name.
    doc_tags: DocTags {
        prefixes: &[],
        typed: &["param", "property", "throws", "exception"],
        unread: &["author", "sample", "see", "since", "suppress", "version"],
    },
    one_token_strings_are_values: true,
});
