use expect_test::expect;
use serde_json::json;

use super::{cleaned, read};

/// Each `(sentence, answer)` as the rewrite it is read as, or why it is none.
fn read_as(cases: &[(&str, &str)]) -> String {
    cases
        .iter()
        .map(|(sentence, answer)| match cleaned(sentence, answer) {
            Ok(rewrite) => format!("{answer:?} -> {rewrite:?}\n"),
            Err(err) => format!("{answer:?} -> error: {err}\n"),
        })
        .collect()
}

#[test]
fn what_wraps_the_whole_answer_is_taken_off() {
    expect![[r#"
        "The cat sat." -> "The cat sat."
        "  The cat sat.\n" -> "The cat sat."
        "\"The cat sat.\"" -> "The cat sat."
        "“The cat sat.”" -> "The cat sat."
        "'The cat sat.'" -> "The cat sat."
        "```\nThe cat sat.\n```" -> "The cat sat."
        "```text\nThe cat sat.\n```" -> "The cat sat."
        "<think>\nTeh is the.\n</think>\n\nThe cat sat." -> "The cat sat."
        "<think>Teh is the.</think>\"The cat sat.\"" -> "The cat sat."
    "#]]
    .assert_eq(&read_as(&[
        ("Teh cat sat.", "The cat sat."),
        ("Teh cat sat.", "  The cat sat.\n"),
        ("Teh cat sat.", "\"The cat sat.\""),
        ("Teh cat sat.", "“The cat sat.”"),
        ("Teh cat sat.", "'The cat sat.'"),
        ("Teh cat sat.", "```\nThe cat sat.\n```"),
        ("Teh cat sat.", "```text\nThe cat sat.\n```"),
        (
            "Teh cat sat.",
            "<think>\nTeh is the.\n</think>\n\nThe cat sat.",
        ),
        ("Teh cat sat.", "<think>Teh is the.</think>\"The cat sat.\""),
    ]));
}

#[test]
fn what_belongs_to_the_sentence_is_kept() {
    expect![[r#"
        "\"Stop,\" he said." -> "\"Stop,\" he said."
        "He said \"stop.\"" -> "He said \"stop.\""
        "\"He said \"stop.\"\"" -> "He said \"stop.\""
        "`foo` is `bar`." -> "`foo` is `bar`."
        "`foo` returns." -> "`foo` returns."
        "```rust``` is a fence." -> "```rust``` is a fence."
        "A thought <think>here</think>." -> "A thought <think>here</think>."
    "#]]
    .assert_eq(&read_as(&[
        ("\"Stop,\" he sayd.", "\"Stop,\" he said."),
        ("He sayd \"stop.\"", "He said \"stop.\""),
        ("He sayd \"stop.\"", "\"He said \"stop.\"\""),
        ("foo is bar.", "`foo` is `bar`."),
        ("`foo` retruns.", "`foo` returns."),
        ("```rust``` is a fense.", "```rust``` is a fence."),
        (
            "A thougt <think>here</think>.",
            "A thought <think>here</think>.",
        ),
    ]));
}

#[test]
fn an_answer_that_is_not_a_rewrite_is_refused() {
    let long = "word ".repeat(30);
    expect![[r#"
        "" -> error: the model answered nothing
        "\"\"" -> error: the model answered nothing
        "<think>Nothing to fix.</think>" -> error: the model answered nothing
        "Here is the corrected sentence:\n\nThe cat sat." -> error: the model answered more lines than the sentence has, which is not a rewrite of it
        "word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word " -> error: the model answered 149 characters for a sentence of 12, which is not a rewrite of it
    "#]]
    .assert_eq(&read_as(&[
        ("Teh cat sat.", ""),
        ("Teh cat sat.", "\"\""),
        ("Teh cat sat.", "<think>Nothing to fix.</think>"),
        (
            "Teh cat sat.",
            "Here is the corrected sentence:\n\nThe cat sat.",
        ),
        ("Teh cat sat.", long.as_str()),
    ]));
}

#[test]
fn a_rewrite_keeps_or_joins_the_sentences_lines() {
    expect![[r#"
        "A sentence that\nwraps." -> "A sentence that\nwraps."
        "A sentence that wraps." -> "A sentence that wraps."
    "#]]
    .assert_eq(&read_as(&[
        ("A sentense that\nwraps.", "A sentence that\nwraps."),
        ("A sentense that\nwraps.", "A sentence that wraps."),
    ]));
}

#[test]
fn the_answer_is_the_first_choices_message() {
    let answer = |content: serde_json::Value, finish: &str| {
        json!({
            "id": "chatcmpl-1",
            "object": "chat.completion",
            "choices": [{
                "index": 0,
                "message": { "role": "assistant", "content": content },
                "finish_reason": finish,
            }],
        })
        .to_string()
    };
    let got = [
        read("Teh cat.", &answer(json!("The cat."), "stop")),
        read("Teh cat.", &answer(json!("The c"), "length")),
        read("Teh cat.", &answer(json!(null), "stop")),
        read("Teh cat.", r#"{"choices": []}"#),
        read("Teh cat.", "<html>Bad gateway</html>"),
    ]
    .map(|got| match got {
        Ok(rewrite) => format!("{rewrite:?}"),
        Err(err) => format!("error: {err}"),
    });
    expect![[r#"
        "The cat."
        error: the model's answer was cut short
        error: the answer has no message in it
        error: the answer has no message in it
        error: the answer is not JSON (expected value at line 1 column 1)"#]]
    .assert_eq(&got.join("\n"));
}
