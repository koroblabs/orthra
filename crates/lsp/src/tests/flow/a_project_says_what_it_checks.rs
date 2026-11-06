//! The project's `orthra.toml` and `.orthra/words.txt` are read at `initialize`, as `orthra check`
//! reads them: its languages are added to the editor's, its words are words, and a file it ignores
//! is open and silent.

use expect_test::expect;

use crate::tests::{Fixture, zed_client};

const FILES: &[(&str, &str)] = &[
    (
        "orthra.toml",
        "languages = [\"en-GB\"]\nignore = [\"fixtures/\"]\n",
    ),
    (".orthra/words.txt", "qzxv\n"),
    ("notes.md", "The colour of qzxv, and teh rest.\n"),
    ("fixtures/sample.md", "Teh recieved qzxva.\n"),
];

#[tokio::test]
async fn languages_words_and_ignored_files() {
    let mut server = Fixture::new(FILES);
    server.initialize(zed_client()).await;
    server.initialized().await;
    let mut rendered = String::new();
    for path in ["notes.md", "fixtures/sample.md"] {
        server.open(path).await;
        let items = server.published(path).unwrap_or_default();
        rendered.push_str(&format!(
            "{path}\n{}",
            server.render(&server.text(path), &items)
        ));
    }
    expect![[r#"
        notes.md
        > The colour of qzxv, and teh rest.
                                  ^^^ known-typo 2: `teh` is a misspelling of `the`
        fixtures/sample.md
        > Teh recieved qzxva.
    "#]]
    .assert_eq(&rendered);
}
