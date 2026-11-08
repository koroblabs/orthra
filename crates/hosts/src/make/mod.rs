//! Makefiles: comments, the messages `$(info)`, `$(warning)` and `$(error)` print, and the
//! variables a makefile defines.
//!
//! Recipes are shell and belong to its programs, so they are held to known typos only. Targets are
//! not declarations: they are file names (`vmlinux`, `foo.o`) and the names of other tools' phases.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "makefile",
    language_ids: &["makefile"],
    file_names: &["Makefile", "makefile", "GNUmakefile", "Makefile.*"],
    extensions: &["mk", "mak"],
    // make reads no escapes of its own: a backslash continues a line, and what follows it in a
    // recipe is the shell's.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_make::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    one_token_strings_are_values: false,
});
