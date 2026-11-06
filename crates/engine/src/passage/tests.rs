//! The sentence a rewrite is offered for, through the hosts that read each kind of file.

use expect_test::expect;
use orthra_text::TextSize;

use crate::{Engine, Parsed};

/// The sentence at the cursor, `$0`, in the file at `path`: the text with the sentence between
/// `«` and `»`, or `none`.
fn marked(path: &str, text: &str) -> String {
    let engine = Engine::new(&["en-US"]).unwrap();
    let at = text.find("$0").expect("a cursor");
    let text = text.replace("$0", "");
    let at = TextSize::try_from(at).unwrap();
    match engine.sentence_at(path, "", &text, at, &mut Parsed::default()) {
        Some(range) => format!(
            "{}«{}»{}",
            &text[..range.start().into()],
            &text[range],
            &text[range.end().into()..]
        ),
        None => format!("none: {text:?}\n"),
    }
}

fn each(path: &str, texts: &[&str]) -> String {
    texts
        .iter()
        .map(|text| marked(path, text))
        .collect::<Vec<_>>()
        .join("--\n")
}

#[test]
fn in_markdown_a_sentence_runs_through_inline_markup_and_a_wrapped_line() {
    expect![[r#"
        Fixx the leter. «It are
        wrapped `here` today.»
        --
        - «A list item that
          wraps.» Next.
        - Another item.
        --
        «Read [the guide](https://x.y/z) now.»
        --
        «**Bold** start.»
        --
        «Use `foo` now.»
        --
        # Title

        «Paragraph one.»
        --
        > quoted line
        > «second line.»
    "#]]
    .assert_eq(&each(
        "notes.md",
        &[
            "Fixx the leter. It a$0re\nwrapped `here` today.\n",
            "- A list item that\n  wr$0aps. Next.\n- Another item.\n",
            "Read [the gu$0ide](https://x.y/z) now.\n",
            "**Bold** st$0art.\n",
            "Use `fo$0o` now.\n",
            "# Title\n\nPara$0graph one.\n",
            "> quoted line\n> sec$0ond line.\n",
        ],
    ));
}

#[test]
fn in_markdown_code_and_markup_alone_are_no_sentence() {
    expect![[r#"
        none: "Text.\n\n```\nlet x = code;\n```\n"
        --
        none: "Text.\n\n<div>html block</div>\n"
    "#]]
    .assert_eq(&each(
        "notes.md",
        &[
            "Text.\n\n```\nlet x = co$0de;\n```\n",
            "Text.\n\n<div>html bl$0ock</div>\n",
        ],
    ));
}

#[test]
fn in_code_a_sentence_is_in_a_comment_on_its_line() {
    expect![[r#"
        // «A comment that»
        // spans two lines.
        fn main() {}
        --
        /// Doc. «More here.»
        fn main() {}
        --
        /* Block
         * «comment here.» */
        --
        none: "fn main() {}\n"
        --
        none: "let s = \"a string here.\";\n"
        --
        let n = 1; // «Trailing comment.»
    "#]]
    .assert_eq(&each(
        "lib.rs",
        &[
            "// A comm$0ent that\n// spans two lines.\nfn main() {}\n",
            "/// Doc. M$0ore here.\nfn main() {}\n",
            "/* Block\n * comm$0ent here. */\n",
            "fn ma$0in() {}\n",
            "let s = \"a str$0ing here.\";\n",
            "let n = 1; // Trailing com$0ment.\n",
        ],
    ));
}

#[test]
fn a_python_docstring_is_a_passage() {
    expect![[r#"
        def f():
            """«Docstring here.»"""
    "#]]
    .assert_eq(&marked(
        "app.py",
        "def f():\n    \"\"\"Docstr$0ing here.\"\"\"\n",
    ));
}

#[test]
fn plain_text_is_one_passage_cut_into_sentences() {
    expect![[r#"
        One. «Two.»

        Three.
    "#]]
    .assert_eq(&marked("notes.txt", "One. Tw$0o.\n\nThree.\n"));
}

#[test]
fn the_language_is_the_first_pack_for_the_script_most_letters_are_in() {
    let english = Engine::new(&["en-GB", "en-US"]).unwrap();
    let got = [
        "The cat sat.",
        "The кіт sat on the mat.",
        "Кіт сидить.",
        "123 + 456",
    ]
    .map(|text| format!("{text}: {:?}", english.language_of(text)));
    expect![[r#"
        The cat sat.: Some("en-GB")
        The кіт sat on the mat.: Some("en-GB")
        Кіт сидить.: None
        123 + 456: None"#]]
    .assert_eq(&got.join("\n"));
}
