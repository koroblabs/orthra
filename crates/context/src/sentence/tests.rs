use expect_test::expect;
use orthra_text::{TextRange, TextSize};

use super::{SENTENCE_BYTES, around};

/// The sentence around the cursor, `$0`, in `text` taken as one passage: the text with the
/// sentence between `«` and `»`, or `none`.
fn marked(text: &str) -> String {
    let at = text.find("$0").expect("a cursor");
    let text = text.replace("$0", "");
    let passage = TextRange::up_to(TextSize::of(&*text));
    match around(&text, passage, TextSize::try_from(at).unwrap()) {
        Some(range) => format!(
            "{}«{}»{}",
            &text[..range.start().into()],
            &text[range],
            &text[range.end().into()..]
        ),
        None => format!("none: {text:?}"),
    }
}

fn each(texts: &[&str]) -> String {
    texts.iter().map(|text| marked(text) + "\n").collect()
}

#[test]
fn a_sentence_ends_at_its_punctuation_or_where_the_passage_does() {
    expect![[r#"
        One. «Two!» Three?
        «One.» Two.
        One. «Two.»
        «just a fragment of text»
        «Version 1.5 is out.» Next.
        Wait… «then go.»
        Really?! «Yes.»
        Це м'ята. «Друге речення!»
        «Ask e.g.» this.
    "#]]
    .assert_eq(&each(&[
        "One. Tw$0o! Three?",
        "One.$0 Two.",
        "One. $0Two.",
        "just a fragment$0 of text",
        "Version 1.5 is $0out. Next.",
        "Wait… th$0en go.",
        "Really?! Ye$0s.",
        "Це м'ята. Друг$0е речення!",
        "Ask e.$0g. this.",
    ]));
}

#[test]
fn a_line_break_is_inside_a_sentence_and_a_blank_line_ends_one() {
    expect![[r#"
        «A sentence that
        wraps lines.» Next.
        No end here

        «Next paragraph»
        none: "No end here\n\n\nNext paragraph"
    "#]]
    .assert_eq(&each(&[
        "A sentence that\nwr$0aps lines. Next.",
        "No end here\n\nNext pa$0ragraph",
        "No end here\n\n$0\nNext paragraph",
    ]));
}

#[test]
fn quotes_brackets_and_emphasis_go_with_the_words_they_open_and_close() {
    expect![[r#"
        «He said "stop."» Then he left.
        «(See this.)» Next.
        First. «**Bold** uses `foo` here.» Last.
        «Read [the guide](https://example.com/a.b).»
        «'Tis said.»
    "#]]
    .assert_eq(&each(&[
        "He said \"st$0op.\" Then he left.",
        "(See th$0is.) Next.",
        "First. **Bold** uses `fo$0o` here. Last.",
        "Read [the gu$0ide](https://example.com/a.b).",
        "'Tis sa$0id.",
    ]));
}

#[test]
fn comment_markers_stay_out_of_the_sentence() {
    expect![[r#"
        // «A comment that»
        /* «in a block» */
        """«Docstring here.»"""
        /// Doc. «More»
        # «A heading»
        /*«tight»*/
        /* «Ends here.»*/
    "#]]
    .assert_eq(&each(&[
        "// A comm$0ent that",
        "/* in a bl$0ock */",
        "\"\"\"Docstr$0ing here.\"\"\"",
        "/// Doc. M$0ore",
        "# A head$0ing",
        "/*tig$0ht*/",
        "/* Ends $0here.*/",
    ]));
}

#[test]
fn no_sentence_where_there_is_no_word_under_the_cursor() {
    expect![[r#"
        none: "One.  Two."
        none: "--- ---"
        none: ""
    "#]]
    .assert_eq(&each(&["One. $0 Two.", "--- $0---", "$0"]));
}

#[test]
fn a_sentence_is_found_only_inside_its_passage() {
    let text = "call(); // A comment. Another one\nnext(); // Not this.";
    let passage = TextRange::new(8.into(), 33.into());
    let at = TextSize::try_from(text.find("Another").unwrap()).unwrap();
    let sentence = around(text, passage, at).map(|range| &text[range]);
    assert_eq!(sentence, Some("Another one"));
    let outside = TextSize::try_from(text.find("call").unwrap()).unwrap();
    assert_eq!(around(text, passage, outside), None);
}

#[test]
fn a_sentence_longer_than_the_limit_is_none_and_never_a_whole_file() {
    let long = format!("Start {}end.", "word ".repeat(SENTENCE_BYTES / 5));
    assert_eq!(
        marked(&format!("{long}$0 Next.")),
        format!("none: {:?}", format!("{long} Next."))
    );
    // A long text of short sentences is read only around the cursor.
    let many = "Short one. ".repeat(SENTENCE_BYTES);
    let at = many.len() / 2 + 3;
    let range = around(
        &many,
        TextRange::up_to(TextSize::of(&*many)),
        TextSize::try_from(at).unwrap(),
    )
    .expect("a sentence");
    assert_eq!(&many[range], "Short one.");
}
