//! What a finding offers: its fixes, here or wherever the file has the same finding, adding the word
//! to the project's words or the user's, and ignoring it in the file or on its line. Ignoring writes
//! a directive the file keeps, after which the word is no finding there.

use expect_test::expect;
use serde_json::{Value, json};

use crate::tests::{Fixture, zed_client};

const FILES: &[(&str, &str)] = &[("notes.md", "We recieve recognizze.\n")];

/// Each action's title, then its edit or its command.
fn render(actions: &[Value]) -> String {
    actions
        .iter()
        .map(|action| {
            let does = match (&action["edit"]["changes"], &action["command"]) {
                (Value::Object(changes), _) => changes
                    .values()
                    .flat_map(|edits| edits.as_array().cloned().unwrap_or_default())
                    .map(|edit| {
                        format!(
                            "{}:{} {:?}",
                            edit["range"]["start"]["line"],
                            edit["range"]["start"]["character"],
                            edit["newText"].as_str().unwrap_or("?")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                (_, Value::Object(command)) => {
                    format!(
                        "{} {}",
                        command["command"].as_str().unwrap_or("?"),
                        command["arguments"]
                    )
                }
                _ => "nothing".to_owned(),
            };
            format!("{} -> {does}\n", action["title"].as_str().unwrap_or("?"))
        })
        .collect()
}

#[tokio::test(start_paused = true)]
async fn fixes_words_and_a_directive() {
    let mut server = Fixture::new(FILES);
    server.initialize(zed_client()).await;
    server.initialized().await;
    server.open("notes.md").await;
    let known = server
        .actions(
            "notes.md",
            json!({ "start": { "line": 0, "character": 4 }, "end": { "line": 0, "character": 4 } }),
        )
        .await;
    let unknown = server
        .actions("notes.md", json!({ "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 13 } }))
        .await;
    expect![[r#"
        Change to `receive` -> 0:3 "receive"
        Add `recieve` to the project's words -> orthra.addWord [{"scope":"project","word":"recieve"}]
        Add `recieve` to your words -> orthra.addWord [{"scope":"user","word":"recieve"}]
        Ignore `recieve` in this file -> 1:0 "<!-- orthra: ignore recieve -->\n"
        Ignore this line -> 0:0 "<!-- orthra: ignore-next-line -->\n"
        --
        Change to `recognize` -> 0:11 "recognize"
        Add `recognizze` to the project's words -> orthra.addWord [{"scope":"project","word":"recognizze"}]
        Add `recognizze` to your words -> orthra.addWord [{"scope":"user","word":"recognizze"}]
        Ignore `recognizze` in this file -> 1:0 "<!-- orthra: ignore recognizze -->\n"
        Ignore this line -> 0:0 "<!-- orthra: ignore-next-line -->\n"
    "#]].assert_eq(&format!("{}--\n{}", render(&known), render(&unknown)));

    // The directive, written where the action says, and the file checked again.
    let uri = server.uri("notes.md");
    server
        .notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": 2 },
                "contentChanges": [{ "text": "We recieve recognizze.\n<!-- orthra: ignore recieve -->\n" }],
            }),
        )
        .await;
    server.after_quiet().await;
    let published = server.published("notes.md").expect("published when quiet");
    let left: Vec<&str> = published
        .iter()
        .filter_map(|d| d["message"].as_str())
        .collect();
    assert_eq!(left, ["`recognizze` is not a known word"]);
}

/// A finding the file has more than once offers to change every one of them: the same word under
/// the same check, so `Teh` at a sentence's start is its own finding with its own fix.
#[tokio::test(start_paused = true)]
async fn the_same_finding_twice_is_changed_everywhere_at_once() {
    let mut server = Fixture::new(&[("notes.md", "Teh cat and teh dog.\n\nThen teh end.\n")]);
    server.initialize(zed_client()).await;
    server.initialized().await;
    server.open("notes.md").await;
    let twice = server
        .actions(
            "notes.md",
            json!({ "start": { "line": 0, "character": 13 }, "end": { "line": 0, "character": 13 } }),
        )
        .await;
    let once = server
        .actions(
            "notes.md",
            json!({ "start": { "line": 0, "character": 1 }, "end": { "line": 0, "character": 1 } }),
        )
        .await;
    expect![[r#"
        Change to `the` -> 0:12 "the"
        Change every `teh` in this file to `the` -> 0:12 "the", 2:5 "the"
        Add `teh` to the project's words -> orthra.addWord [{"scope":"project","word":"teh"}]
        Add `teh` to your words -> orthra.addWord [{"scope":"user","word":"teh"}]
        Ignore `teh` in this file -> 3:0 "<!-- orthra: ignore teh -->\n"
        Ignore this line -> 0:0 "<!-- orthra: ignore-next-line -->\n"
        --
        Change to `The` -> 0:0 "The"
        Add `Teh` to the project's words -> orthra.addWord [{"scope":"project","word":"Teh"}]
        Add `Teh` to your words -> orthra.addWord [{"scope":"user","word":"Teh"}]
        Ignore `Teh` in this file -> 3:0 "<!-- orthra: ignore Teh -->\n"
        Ignore this line -> 0:0 "<!-- orthra: ignore-next-line -->\n"
    "#]]
    .assert_eq(&format!("{}--\n{}", render(&twice), render(&once)));
}
