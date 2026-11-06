//! A client that does not pull is sent diagnostics: nothing while the project is being learned,
//! then everything open at once, then each change once the buffer is quiet.

use expect_test::expect;
use serde_json::json;

use crate::tests::{Fixture, pushed_client};

const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[package]\nname = \"qzxvlib\"\nversion = \"0.1.0\"\n",
    ),
    (
        "src/lib.rs",
        "// qzxvlib recieves teh value.\npub fn greet() {}\n",
    ),
];

#[tokio::test(start_paused = true)]
async fn nothing_before_the_project_is_learned_then_everything_open() {
    let mut server = Fixture::new(FILES);
    server.initialize(pushed_client()).await;
    server.open("src/lib.rs").await;
    // Opened before the project is learned: `qzxvlib`, which the manifest declares, would read as
    // an unknown word, so nothing is said yet.
    assert_eq!(server.published("src/lib.rs"), None);

    server.initialized().await;
    let published = server
        .published("src/lib.rs")
        .expect("published once learned");
    expect![[r#"
        > // qzxvlib recieves teh value.
                     ^^^^^^^^ known-typo 2: `recieves` is a misspelling of `receives`
                              ^^^ known-typo 2: `teh` is a misspelling of `the`
        > pub fn greet() {}
    "#]]
    .assert_eq(&server.render(&server.text("src/lib.rs"), &published));
}

#[tokio::test(start_paused = true)]
async fn a_change_is_published_once_the_buffer_is_quiet() {
    let mut server = Fixture::new(FILES);
    server.initialize(pushed_client()).await;
    server.initialized().await;
    server.open("src/lib.rs").await;
    let uri = server.uri("src/lib.rs");
    server
        .notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": 2 },
                "contentChanges": [{ "text": "// Occured once.\n" }],
            }),
        )
        .await;
    let before = server.published("src/lib.rs").expect("published at open");
    assert_eq!(before.len(), 2, "still the open's findings: {before:#?}");

    server.after_quiet().await;
    let after = server
        .published("src/lib.rs")
        .expect("published when quiet");
    expect![[r#"
        > // Occured once.
             ^^^^^^^ known-typo 2: `Occured` is a misspelling of `Occurred`
    "#]]
    .assert_eq(&server.render("// Occured once.\n", &after));
}
