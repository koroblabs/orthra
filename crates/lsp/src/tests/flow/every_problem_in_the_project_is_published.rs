//! Once the project is learned, every file in it is checked and the closed files' findings are
//! published, so the editor can list every problem in the project and not only the open files'.
//! A file that opens is published as an open file, and one that closes as a closed one.

use expect_test::expect;
use serde_json::{Value, json};

use crate::tests::{Fixture, zed_client};

const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[package]\nname = \"qzxvlib\"\nversion = \"0.1.0\"\n",
    ),
    (
        "src/lib.rs",
        "// qzxvlib recieves teh value.\npub fn greet() {}\n",
    ),
    ("src/other.rs", "// Occured once.\npub fn other() {}\n"),
    ("README.md", "Definately a readme.\n"),
    ("docs/clean.md", "Nothing wrong here.\n"),
];

fn words(diagnostics: &Option<Vec<Value>>) -> String {
    match diagnostics {
        None => "never published".to_owned(),
        Some(found) if found.is_empty() => "cleared".to_owned(),
        Some(found) => found
            .iter()
            .map(|d| d["message"].as_str().unwrap_or("?").to_owned())
            .collect::<Vec<_>>()
            .join("; "),
    }
}

#[tokio::test]
async fn open_and_closed_files_are_published() {
    let mut server = Fixture::new(FILES);
    server.initialize(zed_client()).await;
    server.open("src/lib.rs").await;
    server.initialized().await;
    let mut seen = String::new();
    for path in ["src/lib.rs", "src/other.rs", "README.md", "docs/clean.md"] {
        seen.push_str(&format!("{path:<14} {}\n", words(&server.published(path))));
    }

    server.open("src/other.rs").await;
    seen.push_str(&format!(
        "opened src/other.rs: {}\n",
        words(&server.published("src/other.rs"))
    ));
    let uri = server.uri("src/lib.rs");
    server
        .notify(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": uri } }),
        )
        .await;
    seen.push_str(&format!(
        "closed src/lib.rs: {}\n",
        words(&server.published("src/lib.rs"))
    ));
    expect![[r#"
        src/lib.rs     `recieves` is a misspelling of `receives`; `teh` is a misspelling of `the`
        src/other.rs   `Occured` is a misspelling of `Occurred`
        README.md      `Definately` is a misspelling of `Definitely`
        docs/clean.md  never published
        opened src/other.rs: `Occured` is a misspelling of `Occurred`
        closed src/lib.rs: `recieves` is a misspelling of `receives`; `teh` is a misspelling of `the`
    "#]].assert_eq(&seen);
}
