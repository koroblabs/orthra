//! A settings file the client watches changes, and the project is checked with what it says now:
//! a language named, a severity lowered.

use std::fs;

use serde_json::json;

use crate::tests::{Fixture, zed_client};

/// The severity and message of every finding last published for `notes.md`.
fn published(server: &Fixture) -> Vec<String> {
    server
        .published("notes.md")
        .unwrap_or_default()
        .iter()
        .map(|d| format!("{} {}", d["severity"], d["message"].as_str().unwrap_or("?")))
        .collect()
}

#[tokio::test]
async fn a_language_and_a_severity_take_effect() {
    let mut server = Fixture::new(&[("notes.md", "The colour of qzxv.\n")]);
    let initialized = server.initialize(zed_client()).await;
    assert!(
        initialized["capabilities"]["executeCommandProvider"]["commands"]
            .as_array()
            .is_some_and(|commands| commands.contains(&json!("orthra.addWord")))
    );
    server.initialized().await;
    assert!(
        server.sent("client/registerCapability"),
        "the settings files are watched"
    );
    server.open("notes.md").await;
    // Built in, English is either spelling.
    assert_eq!(published(&server), ["2 `qzxv` is not a known word"]);

    fs::write(
        server.root.join("orthra.toml"),
        "languages = [\"en-US\"]\nseverity = \"information\"\n",
    )
    .unwrap();
    let toml = server.uri("orthra.toml");
    let pass = server.next_pass();
    server
        .notify(
            "workspace/didChangeWatchedFiles",
            json!({ "changes": [{ "uri": toml, "type": 2 }] }),
        )
        .await;
    server.pass_finished(pass).await;
    assert_eq!(
        published(&server),
        [
            "3 `colour` is not a known word",
            "3 `qzxv` is not a known word"
        ]
    );
}
