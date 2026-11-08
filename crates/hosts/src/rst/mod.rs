//! reStructuredText: paragraphs, titles and the text of an admonition, never a literal block, a
//! directive's arguments or a role's target.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

// A paragraph is one node that holds its inline markup, and a capture cannot have a hole in it.
// Its `interpreted text`, a role's target and a link's `text <target>`_ are read by the
// segmenter as code between backticks, which it holds to known typos as it does in Markdown. It
// does not know a ``literal``, whose double backticks pair up as empty code and leave `flaskr` in
// ``flaskr`` a word (Flask's docs, 2026-11-09: 27 findings became 56), nor a |substitution|
// reference, so those are cut out by their delimiters.
pub(crate) static HOST: TreeSitterHost = TreeSitterHost::cutting(
    Grammar {
        id: "restructuredtext",
        language_ids: &["restructuredtext"],
        file_names: &[],
        extensions: &["rst"],
        // A backslash in prose is LaTeX (`\tableofcontents`) or a Windows path far more often than
        // an escape that hides a letter, so none is read as one: `\t` would eat the `t` (Sphinx's
        // `latex.rst` reported `ableofcontents`, `lpha`, `athbb`). reST's own `\*` is punctuation.
        conventions: Conventions {
            backslash_escapes: false,
            doubled_escapes: false,
            line_comment: Some(".."),
            block_comment: None,
        },
        language: LanguageSource::Constant(tree_sitter_rst::LANGUAGE),
        query: include_str!("spell.scm"),
        vocabulary: include_str!("vocabulary.txt"),
        test_files: &[],
        generated_files: &[],
        doc_tags: DocTags::NONE,
        one_token_strings_are_values: false,
    },
    &[("``", "``"), ("|", "|")],
)
// tree-sitter-rst parses an edited tree differently from the text alone.
.parsed_whole();
