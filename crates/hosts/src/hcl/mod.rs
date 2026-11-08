//! HCL and Terraform: comments, the text of strings, and the names a configuration declares.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "hcl",
    language_ids: &["hcl", "terraform", "terraform-vars"],
    file_names: &[],
    extensions: &["hcl", "tf", "tfvars", "nomad"],
    // A quoted template's `\n` is an escape. The grammar keeps it in the literal.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: Some("#"),
        block_comment: Some(("/*", "*/")),
    },
    language: LanguageSource::Constant(tree_sitter_hcl::LANGUAGE),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[".terraform.lock.hcl"],
    doc_tags: DocTags::NONE,
    // `"aws_s3_bucket"`, `"t3.micro"`, `"10.0.0.0/16"`: a string of one token is a name or a
    // value, and a description has spaces.
    one_token_strings_are_values: true,
});
