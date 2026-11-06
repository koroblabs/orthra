use expect_test::expect;
use orthra_text::{TextRange, TextSize};

use super::Annotation;

/// The annotation over the first occurrence of `word` in `text`.
fn at(text: &str, word: &str, label: &str) -> Annotation {
    let start = text
        .find(word)
        .unwrap_or_else(|| panic!("{word:?} is not in the text"));
    let start = TextSize::try_from(start).unwrap();
    Annotation::new(TextRange::at(start, TextSize::of(word)), label)
}

#[test]
fn carets_under_words_in_order_whatever_the_input_order() {
    let text = "/// Retruns the frist element.\nfn main() {}\n";
    let rendered = Annotation::render(
        text,
        &[
            at(text, "frist", "known-typo -> first"),
            at(text, "Retruns", "unknown-word"),
        ],
    );
    expect![[r#"
        > /// Retruns the frist element.
              ^^^^^^^ unknown-word
                          ^^^^^ known-typo -> first
        > fn main() {}
    "#]]
    .assert_eq(&rendered);
}

#[test]
fn a_clean_result_is_the_document_not_an_empty_string() {
    expect![[r#"
        > nothing to see
        >
        > here
    "#]]
    .assert_eq(&Annotation::render("nothing to see\n\nhere\n", &[]));
    assert_eq!(
        Annotation::render("", &[]),
        "",
        "only an empty document renders as nothing"
    );
}

#[test]
fn cyrillic_and_tabs_keep_the_carets_aligned() {
    let text = "\t// Привіт, свiт\n";
    expect![[r#"
        > 	// Привіт, свiт
          	           ^^^^ mixed-script
    "#]]
    .assert_eq(&Annotation::render(
        text,
        &[at(text, "свiт", "mixed-script")],
    ));
}

#[test]
fn an_empty_range_and_a_range_across_lines_say_so() {
    let text = "first line\nsecond line\n";
    let across = TextRange::new(TextSize::new(6), TextSize::new(17));
    let empty = TextRange::empty(TextSize::new(5));
    expect![[r#"
        > first line
               ^ missing-word (empty)
                ^^^^ sentence (+1 lines)
        > second line
    "#]]
    .assert_eq(&Annotation::render(
        text,
        &[
            Annotation::new(across, "sentence"),
            Annotation::new(empty, "missing-word"),
        ],
    ));
}

#[test]
#[should_panic(expected = "starts inside a character")]
fn an_annotation_inside_a_character_is_a_bug_in_its_producer() {
    let text = "б\n";
    Annotation::render(
        text,
        &[Annotation::new(TextRange::empty(TextSize::new(1)), "x")],
    );
}

#[test]
fn parse_reads_back_what_render_wrote() {
    let text = "\t// Привіт, свiт\nfn recieve_msg() {}\n\nlast line\n";
    let annotations = vec![
        at(text, "свiт", "mixed-script"),
        at(text, "recieve", "known-typo -> receive"),
        at(text, "msg", ""),
        Annotation::new(TextRange::empty(TextSize::new(1)), "missing-word"),
    ];
    let rendered = Annotation::render(text, &annotations);
    let (parsed_text, parsed) = Annotation::parse(&rendered);
    assert_eq!(parsed_text, text);
    assert_eq!(Annotation::render(&parsed_text, &parsed), rendered);
    let mut expected = annotations;
    expected.sort_by_key(|a| (a.range.start(), a.range.end()));
    assert_eq!(parsed, expected);
}

#[test]
#[should_panic(expected = "a range across lines cannot be read back")]
fn a_range_across_lines_cannot_be_an_expectation() {
    Annotation::parse("> first line\n        ^^^^ sentence (+1 lines)\n> second\n");
}

#[test]
#[should_panic(expected = "an annotated line is `> text`, `>`, or carets indented by two")]
fn a_line_in_no_known_shape_is_refused() {
    Annotation::parse("> fine\nnot a rendering\n");
}
