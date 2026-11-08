//! C#.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "csharp",
    language_ids: &["csharp"],
    file_names: &[],
    extensions: &["cs", "csx"],
    // The query cuts escape sequences out of regular strings, and a verbatim string has none: a
    // backslash in C# is a path or a regex more often than an escape in what is left.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_c_sharp::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*Test.cs", "*Tests.cs", "Tests/"],
    generated_files: &["*.g.cs", "*.g.i.cs", "*.generated.cs", "*.Designer.cs"],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: true,
});
