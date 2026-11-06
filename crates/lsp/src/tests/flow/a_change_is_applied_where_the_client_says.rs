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
