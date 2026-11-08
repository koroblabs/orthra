//! Protocol Buffers: comments, and the names a schema declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "proto",
    // `proto` is what Zed's Proto extension and `buf` send; `protobuf` and `proto3` are what other
    // editors name it.
    language_ids: &["proto", "protobuf", "proto3"],
    file_names: &[],
    extensions: &["proto"],
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_proto::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[
        "*.pb.go",
        "*_pb2.py",
        "*_pb2.pyi",
        "*.pb.cc",
        "*.pb.h",
        "*_pb.js",
    ],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
