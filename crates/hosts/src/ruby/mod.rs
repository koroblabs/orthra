//! Ruby: comments, YARD tags, the text of strings, and the names a file declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "ruby",
    language_ids: &["ruby"],
    file_names: &[
        "Rakefile",
        "Gemfile",
        "Guardfile",
        "Capfile",
        "Vagrantfile",
        "Podfile",
        "Brewfile",
        ".irbrc",
        ".pryrc",
    ],
    extensions: &["rb", "rake", "gemspec", "ru", "rbi"],
    // The grammar makes every escape of a string a node of its own (`escape_sequence`), so the
    // query cuts them out and a backslash is never left inside a piece.
    conventions: Conventions {
        backslash_escapes: false,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: None,
    },
    language: LanguageSource::Constant(tree_sitter_ruby::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &["*_test.rb", "*_spec.rb", "spec/"],
    generated_files: &["*_pb.rb", "*_services_pb.rb"],
    // YARD. `@param name [Type] text` and `@option opts [Type] :key text` carry a name, which the
    // reader skips as the one token after the tag, and then a type in brackets; `@return [Type]`
    // and `@raise [Error]` carry the type alone.
    doc_tags: DocTags {
        prefixes: &[],
        code: &[],
        typed: &[
            "option",
            "param",
            "raise",
            "return",
            "type",
            "yieldparam",
            "yieldreturn",
        ],
        unread: &["api", "author", "since", "see", "version"],
    },
    // `"Content-Type"`, `"HTTP_HOST"`, `"rack.input"`: a string of one token in Ruby is a key, a
    // header or a name. A message has spaces.
    one_token_strings_are_values: true,
});
