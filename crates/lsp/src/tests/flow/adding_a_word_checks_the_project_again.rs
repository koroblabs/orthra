//! "Add to the project's words" writes the project's words file and checks the project again:
//! the word is no finding in the open file, nor in a closed one.

use std::fs;

use expect_test::expect;
use serde_json::json;

use crate::tests::{Fixture, zed_client};

const FILES: &[(&str, &str)] = &[
    ("notes.md", "A qzxv here.\n"),
    ("other.md", "Another qzxv there.\n"),
    (".orthra/words.txt", "# the project's words\nabcq\n"),
];

#[tokio::test]
async fn the_word_is_written_and_every_file_checked_again() {
    let mut server = Fixture::new(FILES);
    server.initialize(zed_client()).await;
    server.initialized().await;
    server.open("notes.md").await;
    let count = |server: &Fixture, path| server.published(path).map(|found| found.len());
    assert_eq!(count(&server, "notes.md"), Some(1));
    assert_eq!(count(&server, "other.md"), Some(1));

    let pass = server.next_pass();
    server
        .request(
            "workspace/executeCommand",
            json!({ "command": "orthra.addWord", "arguments": [{ "scope": "project", "word": "qzxv" }] }),
        )
        .await;
    server.pass_finished(pass).await;
    let words = fs::read_to_string(server.root.join(".orthra/words.txt")).expect("written");
    expect![[r#"
        # the project's words
        abcq
        qzxv
        open file: Some(0)
        closed file: Some(0)
    "#]]
    .assert_eq(&format!(
        "{words}open file: {:?}\nclosed file: {:?}\n",
        count(&server, "notes.md"),
        count(&server, "other.md")
    ));
}

#[tokio::test]
async fn a_word_for_the_user_goes_to_the_users_file() {
    let mut server = Fixture::new(&[("notes.md", "A qzxv here.\n")]);
    server.initialize(zed_client()).await;
    server.initialized().await;
    let pass = server.next_pass();
    server
        .request(
            "workspace/executeCommand",
            json!({ "command": "orthra.addWord", "arguments": [{ "scope": "user", "word": "Qzxv" }] }),
        )
        .await;
    server.pass_finished(pass).await;
    let written = fs::read_to_string(server.user.join("words.txt")).expect("written");
    assert_eq!(written, "Qzxv\n");
    assert!(!server.root.join(".orthra/words.txt").exists());
}
