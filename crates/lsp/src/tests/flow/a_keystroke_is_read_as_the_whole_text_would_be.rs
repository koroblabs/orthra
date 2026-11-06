//! A keystroke in an open document is parsed again where it changed, from the tree its last check
//! kept: what is published after it is what a check of the whole new text says.

use expect_test::expect;
use serde_json::json;

use crate::tests::{Fixture, zed_client};

#[tokio::test(start_paused = true)]
async fn an_edit_that_turns_a_call_into_a_comment_is_read_as_a_comment() {
    let mut server = Fixture::new(&[("src/main.rs", "fn main() {\n    go(recieve);\n}\n")]);
    server.initialize(zed_client()).await;
    server.initialized().await;
    server.open("src/main.rs").await;
    assert_eq!(
        server.published("src/main.rs"),
        Some(Vec::new()),
        "a word at a use site is the API's, not the author's to fix"
    );
    let uri = server.uri("src/main.rs");
    // `go(` becomes `// `. The text keeps its length, so only a tree told of the edit can see that
    // the call is a comment now, and `recieve` is in prose instead of at a use site.
    server
        .notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": 2 },
                "contentChanges": [
                    { "range": { "start": { "line": 1, "character": 4 },
                                 "end": { "line": 1, "character": 7 } },
                      "text": "// " },
                ],
            }),
        )
        .await;
    server.after_quiet().await;
    let items = server
        .published("src/main.rs")
        .expect("published when quiet");
    expect![[r#"
        > fn main() {
        >     // recieve);
                 ^^^^^^^ known-typo 2: `recieve` is a misspelling of `receive`
        > }
    "#]]
    .assert_eq(&server.render("fn main() {\n    // recieve);\n}\n", &items));
}
