//! OCaml: comments, ocamldoc, the text of strings, and the names a file declares. Implementations
//! (`.ml`) and interfaces (`.mli`) are two grammars of one language, read by one query.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

/// `(* … *)` comments nest, and OCaml has no line comment.
const CONVENTIONS: Conventions = Conventions {
    backslash_escapes: true,
    doubled_escapes: false,
    line_comment: None,
    block_comment: Some(("(*", "*)")),
};

/// ocamldoc's tags: `@param name text`, `@raise Exn text`; a name, a version or a reference is not
/// prose.
const DOC_TAGS: DocTags = DocTags {
    typed: &["param", "raise"],
    unread: &["author", "before", "see", "since", "version"],
    ..DocTags::NONE
};

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "ocaml",
    language_ids: &["ocaml"],
    file_names: &[],
    extensions: &["ml"],
    conventions: CONVENTIONS,
    language: LanguageSource::Constant(tree_sitter_ocaml::LANGUAGE_OCAML),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["test/", "tests/", "*_test.ml", "*_tests.ml"],
    generated_files: &[],
    doc_tags: DOC_TAGS,
    one_token_strings_are_values: true,
});

pub(crate) static INTERFACE: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "ocaml_interface",
    language_ids: &["ocaml_interface", "ocaml.interface"],
    file_names: &[],
    extensions: &["mli"],
    conventions: CONVENTIONS,
    language: LanguageSource::Constant(tree_sitter_ocaml::LANGUAGE_OCAML_INTERFACE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["test/", "tests/", "*_test.mli", "*_tests.mli"],
    generated_files: &[],
    doc_tags: DOC_TAGS,
    one_token_strings_are_values: true,
});
