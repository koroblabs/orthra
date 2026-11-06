//! Zed declares that it can pull diagnostics, and orthra pushes them anyway: open files and closed
//! ones alike, each publication replacing the last. Zed's list of a project's problems opens a
//! buffer for each file it shows; when an open cleared a closed file's findings so a pull could
//! report them, the list emptied, refilled and emptied again (reported 2026-11-05).

use expect_test::expect;
use serde_json::json;

use crate::tests::{Fixture, zed_client};

const FILES: &[(&str, &str)] = &[
    ("notes.md", "Definately a note.\n"),
    ("other.md", "Teh other one.\n"),
];

#[tokio::test(start_paused = true)]
async fn opening_and_closing_never_clears_a_finding() {
    let mut server = Fixture::new(FILES);
    let initialized = server.initialize(zed_client()).await;
    assert_eq!(
        initialized["capabilities"]["diagnosticProvider"],
        json!(null),
        "pushed, so that one channel says everything"
    );
    server.initialized().await;
    // What a list of problems does: open a file to show it, then let it go.
    server.open("other.md").await;
    let uri = server.uri("other.md");
    server
        .notify(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": uri } }),
        )
        .await;
    let seen: Vec<String> = server
        .publications("other.md")
        .iter()
        .map(|found| {
            found
                .iter()
                .map(|d| d["message"].as_str().unwrap_or("?").to_owned())
                .collect::<Vec<_>>()
                .join("; ")
        })
        .collect();
    expect![[r#"
        `Teh` is a misspelling of `The`
        `Teh` is a misspelling of `The`
        `Teh` is a misspelling of `The`"#]]
    .assert_eq(&seen.join("\n"));
}
