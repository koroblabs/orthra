//! The same finding, after a line of Cyrillic and an emoji, in the column each client counts in:
//! bytes for one that offers UTF-8, UTF-16 code units for one that does not.

use expect_test::expect;
use serde_json::Value;

use crate::tests::{Fixture, pushed_client, zed_client};

const FILES: &[(&str, &str)] = &[("notes.txt", "Привіт 👋 recieve\n")];

/// `line:character` of each finding, as the client was told it.
fn positions(diagnostics: &[Value]) -> String {
    diagnostics
        .iter()
        .map(|d| {
            format!(
                "{}:{}-{}:{}",
                d["range"]["start"]["line"],
                d["range"]["start"]["character"],
                d["range"]["end"]["line"],
                d["range"]["end"]["character"]
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[tokio::test]
async fn utf8_when_offered_and_utf16_otherwise() {
    let mut utf8 = Fixture::new(FILES);
    let answer = utf8.initialize(zed_client()).await;
    assert_eq!(answer["capabilities"]["positionEncoding"], "utf-8");
    utf8.initialized().await;
    utf8.open("notes.txt").await;
    let utf8_items = utf8.published("notes.txt").expect("published at open");

    let mut utf16 = Fixture::new(FILES);
    let answer = utf16.initialize(pushed_client()).await;
    assert_eq!(answer["capabilities"]["positionEncoding"], "utf-16");
    utf16.initialized().await;
    utf16.open("notes.txt").await;
    let utf16_items = utf16.published("notes.txt").expect("published at open");

    expect![[r#"
        utf-8  0:18-0:25
        utf-16 0:10-0:17
    "#]]
    .assert_eq(&format!(
        "utf-8  {}\nutf-16 {}\n",
        positions(&utf8_items),
        positions(&utf16_items)
    ));
}
