//! Dart: comments, `///` doc comments, the text of string literals, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "dart",
    language_ids: &["dart"],
    file_names: &[],
    extensions: &["dart"],
    // The grammar cuts a string's escapes (`\n`, `\u{1F600}`) and its `$name` and `${...}` out of
    // the text, so the segmenter never sees a backslash. A raw string has none and is a value.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_dart::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    // `test/` is every language's; Dart's own are `foo_test.dart` beside the code.
    test_files: &["*_test.dart", "integration_test/"],
    // build_runner's output (json_serializable, freezed, mockito, auto_route, injectable) and
    // protoc's.
    generated_files: &[
        "*.g.dart",
        "*.freezed.dart",
        "*.pb.dart",
        "*.pbenum.dart",
        "*.pbjson.dart",
        "*.pbgrpc.dart",
        "*.mocks.dart",
        "*.gr.dart",
        "*.config.dart",
        "*.gen.dart",
        "*.chopper.dart",
        "*.mapper.dart",
        "*_bindings_generated.dart",
        "generated_plugin_registrant.dart",
    ],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
