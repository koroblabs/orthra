use expect_test::expect;
use orthra_text::line_index::{Encoding, LineIndex};

use orthra_text::TextSize;

use super::{ignore_edit, next_line_edit};

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

/// The text after the edit that ignores the line holding `at`, written with `open` and `close`.
fn line_ignored(text: &str, at: usize, open: &str, close: &str) -> String {
    let index = LineIndex::new(text);
    let at = TextSize::try_from(at).unwrap();
    let edit = next_line_edit(text, &index, Encoding::Utf8, at, open, close).expect("an edit");
    let at = lspkit_text::lsp::offset(&index, text, edit.range.start, Encoding::Utf8)
        .expect("in the text");
    let mut after = text.to_owned();
    after.insert_str(usize::from(at), &edit.new_text);
    after
}

/// A line is ignored by a directive above it, indented as it is; a word ignored in the file joins a
/// directive that lists words, never one that ignores a line.
#[test]
fn a_line_is_ignored_from_the_line_above_it() {
    let rendered = [
        line_ignored("fn f() {\n    // Teh qzxv.\n}\n", 20, "//", ""),
        line_ignored("Intro.\n\nTeh qzxv.\n", 9, "<!--", "-->"),
        ignored("// orthra: ignore-next-line\n// Teh.\n", "qzxv", "//", ""),
    ]
    .join("--\n");
    expect![[r#"
        fn f() {
            // orthra: ignore-next-line
            // Teh qzxv.
        }
        --
        Intro.

        <!-- orthra: ignore-next-line -->
        Teh qzxv.
        --
        // orthra: ignore-next-line
        // Teh.
        // orthra: ignore qzxv
    "#]]
    .assert_eq(&rendered);
}
