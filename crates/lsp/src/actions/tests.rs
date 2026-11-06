use expect_test::expect;
use orthra_text::line_index::{Encoding, LineIndex};

use super::ignore_edit;

/// The text after the edit that ignores `word`, written with `open` and `close`.
fn ignored(text: &str, word: &str, open: &str, close: &str) -> String {
    let index = LineIndex::new(text);
    let edit = ignore_edit(text, &index, Encoding::Utf8, word, open, close).expect("an edit");
    let at = lspkit_text::lsp::offset(&index, text, edit.range.start, Encoding::Utf8)
        .expect("in the text");
    let mut after = text.to_owned();
    after.insert_str(usize::from(at), &edit.new_text);
    after
}

#[test]
fn a_word_joins_the_directive_or_starts_one() {
    let rendered = [
        ignored("fn main() {}", "qzxv", "//", ""),
        ignored("fn main() {}\n// orthra: ignore abcq\n", "qzxv", "//", ""),
        ignored(
            "Text.\n<!-- orthra: ignore abcq -->\nMore.\n",
            "qzxv",
            "<!--",
            "-->",
        ),
        ignored("# A comment.\n", "qzxv", "#", ""),
    ]
    .join("--\n");
    expect![[r#"
        fn main() {}
        // orthra: ignore qzxv
        --
        fn main() {}
        // orthra: ignore abcq qzxv
        --
        Text.
        <!-- orthra: ignore abcq qzxv -->
        More.
        --
        # A comment.
        # orthra: ignore qzxv
    "#]]
    .assert_eq(&rendered);
}
