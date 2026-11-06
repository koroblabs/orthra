use super::WordPiece;
use crate::tests::VOCABULARY;

fn tokens(text: &str) -> String {
    let words = WordPiece::new(VOCABULARY.iter().map(|token| (*token).to_owned())).unwrap();
    let mut ids = Vec::new();
    words.ids(text, &mut ids);
    ids.iter()
        .map(|id| VOCABULARY[*id as usize])
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn text_is_cut_as_bert_cuts_it() {
    let got = [
        // Lower-cased, punctuation apart, the longest piece first and the rest as `##`.
        "The cat's mats, received.",
        // Accents stripped: `Café` is `cafe`.
        "Café",
        // A word with a piece the vocabulary lacks is one unknown token, not a partial one.
        "recxeive",
        // Whitespace of every kind splits; control characters vanish.
        "the\tcat\u{7}\u{a0}sat",
        // A CJK ideograph is a word of its own; typographic punctuation splits.
        "cat猫mat “fix”",
    ]
    .map(|text| format!("{text:?} -> {}", tokens(text)));
    expect_test::expect![[r#"
        "The cat's mats, received." -> the cat ' [UNK] mat ##s , [UNK] .
        "Café" -> cafe
        "recxeive" -> [UNK]
        "the\tcat\u{7}\u{a0}sat" -> the cat sat
        "cat猫mat “fix”" -> cat [UNK] mat [UNK] fix [UNK]"#]]
    .assert_eq(&got.join("\n"));
}

#[test]
fn a_vocabulary_without_the_special_tokens_is_refused() {
    let err = WordPiece::new(["the".to_owned(), "[UNK]".to_owned()]).unwrap_err();
    expect_test::expect!["the vocabulary has no [MASK]"].assert_eq(&err);
}
