//! Lua: comments, LuaLS doc comments, the text of strings, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "lua",
    language_ids: &["lua"],
    file_names: &[".luacheckrc", ".luarc.lua"],
    extensions: &["lua"],
    // tree-sitter-lua keeps an escape inside the string's content, so segmentation has to know
    // `"\n"` is one.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("--"),
        block_comment: Some(("--[[", "]]")),
    },
    language: LanguageSource::Constant(tree_sitter_lua::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*_spec.lua", "spec/", "test/"],
    generated_files: &[],
    // LuaLS annotations: `---@param name type description`, `---@return type name description`.
    doc_tags: DocTags {
        prefixes: &[],
        code: &[],
        typed: &[
            "param", "return", "type", "field", "alias", "class", "generic", "cast", "overload",
            "operator", "vararg",
        ],
        unread: &[
            "diagnostic",
            "module",
            "meta",
            "source",
            "version",
            "see",
            "async",
            "nodiscard",
            "private",
            "protected",
            "package",
            "public",
            "enum",
        ],
    },
    one_token_strings_are_values: true,
});
