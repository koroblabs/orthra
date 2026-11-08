use expect_test::{Expect, expect};
use orthra_testkit::annotate::Annotation;

use super::{DocTags, Grammar, LanguageSource, TreeSitterHost};
use crate::{Conventions, Host, Hosts};

/// A host over Rust's grammar, with a query of the test's own.
const fn rust_with(query: &'static str) -> TreeSitterHost {
    TreeSitterHost::new(Grammar {
        id: "test",
        language_ids: &[],
        file_names: &[],
        extensions: &[],
        conventions: Conventions {
            backslash_escapes: false,
            doubled_escapes: false,
            line_comment: Some("//"),
            block_comment: None,
        },
        language: LanguageSource::Constant(tree_sitter_rust::LANGUAGE),
        query,
        vocabulary: "",
        test_files: &[],
        generated_files: &[],
        doc_tags: DocTags::NONE,
        one_token_strings_are_values: false,
    })
}

/// The host's health, then the spans it finds in `text`.
fn check(host: &'static TreeSitterHost, text: &str, expect: Expect) {
    let host: &dyn Host = &host;
    let mut spans = Vec::new();
    host.spans(text, &mut spans);
    let annotations: Vec<Annotation> = spans
        .iter()
        .map(|span| Annotation::new(span.range, format!("{:?}", span.kind).to_lowercase()))
        .collect();
    expect.assert_eq(&format!(
        "health {:?}\n{}",
        host.health(),
        Annotation::render(text, &annotations)
    ));
}

static NO_SUCH_NODE: TreeSitterHost =
    rust_with("(line_comment) @comment\n(no_such_node) @string\n");

#[test]
fn a_query_the_grammar_rejects_is_the_hosts_error_and_yields_no_spans() {
    check(
        &NO_SUCH_NODE,
        "// A comment the first pattern would find.\nfn main() {}\n",
        expect![[r#"
            health Err("spell.scm: Query error at 2:2. Invalid node type \"no_such_node\"")
            > // A comment the first pattern would find.
            > fn main() {}
        "#]],
    );
}

static MISSPELT_CAPTURE: TreeSitterHost =
    rust_with("(line_comment) @comment\n(string_content) @strnig\n");

#[test]
fn a_capture_with_no_meaning_is_the_hosts_error_and_yields_no_spans() {
    check(
        &MISSPELT_CAPTURE,
        "// A comment the first pattern would find.\nfn main() {}\n",
        expect![[r#"
            health Err("spell.scm: unknown capture @strnig")
            > // A comment the first pattern would find.
            > fn main() {}
        "#]],
    );
}

static NESTED: TreeSitterHost = rust_with(
    "(field_identifier) @declaration\n(field_declaration) @comment\n(struct_item name: (type_identifier) @string)\n",
);

#[test]
fn nested_captures_keep_the_outer_one() {
    check(
        &NESTED,
        "struct Point { across: i32, down: i32 }\n",
        expect![[r#"
            health Ok(())
            > struct Point { across: i32, down: i32 }
                     ^^^^^ string
                             ^^^^^^^^^^^ comment
                                          ^^^^^^^^^ comment
        "#]],
    );
}

static SKIPPED: TreeSitterHost = rust_with(
    "(field_declaration) @skip\n(field_identifier) @declaration\n(struct_item name: (type_identifier) @declaration)\n",
);

#[test]
fn a_skip_claims_its_range_for_nothing() {
    check(
        &SKIPPED,
        "struct Point { across: i32, down: i32 }\n",
        expect![[r#"
            health Ok(())
            > struct Point { across: i32, down: i32 }
                     ^^^^^ declaration
        "#]],
    );
}

/// What catches a grammar built for an ABI the core does not read: it fails when the language is
/// set, at runtime, and never at compile time.
#[test]
fn every_builtin_host_loads() {
    let hosts = Hosts::builtin();
    let failures: Vec<String> = hosts
        .hosts
        .iter()
        .filter_map(|host| Some(format!("{}: {}", host.id(), host.health().err()?)))
        .collect();
    assert_eq!(failures, Vec::<String>::new());
    for language in [
        "rust",
        "typescript",
        "typescriptreact",
        "javascript",
        "javascriptreact",
        "python",
        "css",
        "html",
        "go",
        "shellscript",
        "php",
        "toml",
        "yaml",
        "sql",
        "json",
        "jsonc",
        "lua",
        "scss",
        "less",
        "kotlin",
        "java",
        "swift",
        "c",
        "cpp",
        "ruby",
    ] {
        assert!(hosts.for_language(language).is_some(), "no {language} host");
    }
}

/// `cargo install` reads the manifest and not the lockfile, so a range lets a grammar move to a
/// release whose node types the query no longer names (`CLAUDE.md` §8.1).
#[test]
fn every_tree_sitter_dependency_is_pinned_exactly() {
    let root: toml::Table = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml"))
        .parse()
        .expect("the root manifest parses");
    let pins: Vec<(&str, Option<&str>)> = root["workspace"]["dependencies"]
        .as_table()
        .expect("[workspace.dependencies]")
        .iter()
        .filter(|(name, _)| name.starts_with("tree-sitter"))
        .map(|(name, spec)| {
            let version = spec.as_str().or_else(|| spec.get("version")?.as_str());
            (name.as_str(), version)
        })
        .collect();
    assert!(
        pins.iter().any(|(name, _)| *name == "tree-sitter"),
        "{pins:?}"
    );
    let loose: Vec<String> = pins
        .iter()
        .filter(|(_, version)| !version.is_some_and(|v| v.starts_with('=') && !v.contains(',')))
        .map(|(name, version)| format!("{name} = {version:?}"))
        .collect();
    assert_eq!(loose, Vec::<String>::new());

    // A crate that names its own version would step around the pins.
    let hosts: toml::Table = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .parse()
        .expect("the hosts manifest parses");
    let local: Vec<&str> = hosts["dependencies"]
        .as_table()
        .expect("[dependencies]")
        .iter()
        .filter(|(name, spec)| {
            name.starts_with("tree-sitter")
                && spec.get("workspace").and_then(toml::Value::as_bool) != Some(true)
        })
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(local, Vec::<&str>::new());
}

#[test]
fn an_edited_document_parsed_with_its_old_tree_has_a_fresh_parses_spans() {
    let hosts = Hosts::builtin();
    let host = hosts.for_language("rust").expect("the Rust host");
    let mut text = "// a coment\nfn main() {\n    let s = \"hello wrld\";\n}\n".to_owned();
    let mut parsed = crate::parsed::Parsed::default();
    let mut reused = Vec::new();
    host.spans_reusing(&text, &mut parsed, &mut reused);
    // A word fixed, a line inserted, a comment opened over code, and the line removed again.
    for (old, new) in [
        ("wrld", "world"),
        ("{\n", "{\n    // one more line\n"),
        ("let s", "// let s"),
        ("    // one more line\n", ""),
    ] {
        let start = text.find(old).expect("the text to replace");
        let range = orthra_text::TextRange::at(
            orthra_text::TextSize::try_from(start).unwrap(),
            orthra_text::TextSize::of(old),
        );
        parsed.edit(&text, range, new);
        text.replace_range(start..start + old.len(), new);
        reused.clear();
        host.spans_reusing(&text, &mut parsed, &mut reused);
        let mut fresh = Vec::new();
        host.spans(&text, &mut fresh);
        assert_eq!(reused, fresh, "after {old:?} -> {new:?}, in:\n{text}");
    }
}
