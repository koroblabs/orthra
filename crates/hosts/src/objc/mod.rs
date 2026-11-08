//! Objective-C: C's comments, strings and declarations, and the classes, protocols, selectors and
//! properties a file declares.

use crate::{
    Conventions,
    c::DOC_TAGS,
    tree_sitter_host::{Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "objective-c",
    language_ids: &["objective-c", "objc"],
    file_names: &[],
    // `.h` stays with C, as it is both languages' header and a path cannot say which; an open
    // header carries the editor's language id, which decides. `.mm` is Objective-C++, which this
    // grammar does not read.
    extensions: &["m"],
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_objc::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*Tests.m", "*Test.m", "*Spec.m", "Tests/"],
    generated_files: &["*.pbobjc.m", "*.pbobjc.h", "*.pbrpc.m"],
    doc_tags: DOC_TAGS,
    one_token_strings_are_values: true,
});
