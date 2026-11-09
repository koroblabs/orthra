//! Incremental sync: each change replaces the range the client names, in its encoding, in order,
//! and what is checked next is the text the client has.

use expect_test::expect;
use serde_json::json;

use crate::tests::{Fixture, zed_client};

#[tokio::test(start_paused = true)]
async fn ranges_after_multibyte_text_land_where_the_client_meant() {
    let mut server = Fixture::new(&[("notes.txt", "Привіт: teh value.\n")]);
    server.initialize(zed_client()).await;
    server.initialized().await;
    server.open("notes.txt").await;
    let uri = server.uri("notes.txt");
    // UTF-8 columns: `Привіт: ` is 14 bytes, so `teh` is 14..17.
    server
        .notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": 2 },
                "contentChanges": [
                    { "range": { "start": { "line": 0, "character": 14 },
                                 "end": { "line": 0, "character": 17 } },
                      "text": "the" },
                    { "range": { "start": { "line": 0, "character": 23 },
                                 "end": { "line": 0, "character": 23 } },
                      "text": " we recieve" },
                ],
            }),
        )
        .await;
    server.after_quiet().await;
    let items = server.published("notes.txt").expect("published when quiet");
    expect![[r#"
        > Привіт: the value we recieve.
                               ^^^^^^^ known-typo 2: `recieve` is a misspelling of `receive`
    "#]]
    .assert_eq(&server.render("Привіт: the value we recieve.\n", &items));
}

/// A change that cannot be applied, at a column inside a character, leaves this server with
/// another text than the client's: what it published is cleared, nothing is offered, and later
/// ranges are refused, since each would land elsewhere in the client's text. A whole new text
/// brings it back.
#[tokio::test(start_paused = true)]
async fn a_change_inside_a_letter_silences_the_file_until_it_is_sent_whole() {
    let mut server = Fixture::new(&[("notes.txt", "Привіт: teh value.\n")]);
    server.initialize(zed_client()).await;
    server.initialized().await;
    server.open("notes.txt").await;
    let uri = server.uri("notes.txt");
    let count = |server: &Fixture| server.published("notes.txt").map(|found| found.len());
    let opened = count(&server);
    let change = |version: i32, change: serde_json::Value| json!({ "textDocument": { "uri": uri, "version": version }, "contentChanges": [change] });
    // UTF-8 column 1 is inside `П`, two bytes.
    let inside = json!({ "range": { "start": { "line": 0, "character": 1 },
                                    "end": { "line": 0, "character": 1 } }, "text": "x" });
    server
        .notify("textDocument/didChange", change(2, inside))
        .await;
    let refused = count(&server);
    let offered = server
        .actions(
            "notes.txt",
            json!({ "start": { "line": 0, "character": 15 }, "end": { "line": 0, "character": 15 } }),
        )
        .await
        .len();
    let range = json!({ "range": { "start": { "line": 0, "character": 0 },
                                   "end": { "line": 0, "character": 0 } }, "text": "A " });
    server
        .notify("textDocument/didChange", change(3, range))
        .await;
    server.after_quiet().await;
    let after_a_range = count(&server);
    server
        .notify(
            "textDocument/didChange",
            change(4, json!({ "text": "We recieve it.\n" })),
        )
        .await;
    server.after_quiet().await;
    let whole = count(&server);
    expect![[r#"
        opened Some(1)
        refused Some(0)
        offered 0
        after a range Some(0)
        sent whole Some(1)
    "#]].assert_eq(&format!(
        "opened {opened:?}\nrefused {refused:?}\noffered {offered}\nafter a range {after_a_range:?}\nsent whole {whole:?}\n"
    ));
}
