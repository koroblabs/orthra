//! XML: its comments, the text between tags, and the attributes that hold text for a reader.

use crate::{
    Conventions,
    tree_sitter_host::{DocTags, Grammar, LanguageSource, TreeSitterHost},
};

#[cfg(test)]
mod tests;

pub(crate) static HOST: TreeSitterHost = TreeSitterHost::new(Grammar {
    id: "xml",
    language_ids: &["xml", "xsl"],
    file_names: &[],
    // Every one is XML whose text is worth reading: schemas and stylesheets (`xsd`, `xsl`,
    // `xslt`), project files whose `<Description>` and `<PackageReleaseNotes>` are published
    // (`csproj`, `fsproj`, `vbproj`, `props`, `targets`), .NET and Android resources (`resx`,
    // `xaml`), service descriptions and feeds (`wsdl`, `rss`, `atom`), icons (`svg`: its `<title>`
    // and `<desc>`) and property lists (`plist`).
    extensions: &[
        "xml", "xsd", "xsl", "xslt", "svg", "plist", "csproj", "fsproj", "vbproj", "props",
        "targets", "resx", "xaml", "wsdl", "rss", "atom",
    ],
    // An entity (`&amp;`) is its own node and is cut out of the text. Android's `strings.xml` is
    // the one XML that has backslash escapes (`\n`, `\'`, `\"`), and a stray backslash elsewhere is
    // a Windows path, which is not words either way.
    conventions: Conventions {
        backslash_escapes: true,
        doubled_escapes: false,
        line_comment: None,
        block_comment: Some(("<!--", "-->")),
    },
    language: LanguageSource::Constant(tree_sitter_xml::LANGUAGE_XML),
    query: include_str!("spell.scm"),
    vocabulary: include_str!("vocabulary.txt"),
    test_files: &[],
    generated_files: &[],
    doc_tags: DocTags::NONE,
    // `title="Name"`, `android:text="OK"`: an attribute of one token is more often a key or a
    // name than a sentence.
    one_token_strings_are_values: true,
});
