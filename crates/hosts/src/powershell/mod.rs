//! PowerShell: comments, the text of strings, and the names a script declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "powershell",
    language_ids: &["powershell"],
    file_names: &[],
    extensions: &["ps1", "psm1", "psd1"],
    // PowerShell escapes with a backtick, which `Conventions` cannot say, and a backslash is a path
    // separator: `"C:\network"` has no newline in it. A string with a backtick is a value in the
    // query, so no `n` of a `` `n `` is read as a word.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: Some(("<#", "#>")),
    },
    language: LanguageSource::Constant(tree_sitter_powershell::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*.Tests.ps1", "tests/", "test/"],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    // `'Stop'`, `"*.ps1"`, `'HKLM:\Software'`: a string of one token is a value, a name or a path.
    one_token_strings_are_values: true,
});
