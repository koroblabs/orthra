use orthra_packs::Packs;

use super::unmixed;

#[test]
fn a_mixed_word_is_offered_in_the_scripts_a_pack_accepts_it_in_or_else_the_one_it_is_mostly_in() {
    let (english, _) = Packs::load(&["en-US"], None).unwrap();
    let got = [
        // A Cyrillic `с`, `е`, `о` in English words.
        "сount",
        "Thе",
        "dосument",
        // A Latin `c` in a Ukrainian word: no pack for Cyrillic is enabled, so the script most of
        // it is in comes first, and Latin cannot write `л`.
        "cлово",
        // Capitals: a Cyrillic `Т`, `Е`, `Х`; and Greek omicron in `hello`.
        "ТEХ",
        "hellο",
        // Letters with no look-alike: nothing to offer.
        "cλ",
    ]
    .map(|word| format!("{word} -> {}", unmixed(word, &english).join(", ")));
    expect_test::expect![[r#"
        сount -> count
        Thе -> The
        dосument -> document
        cлово -> слово
        ТEХ -> TEX
        hellο -> hello
        cλ -> "#]]
    .assert_eq(&got.join("\n"));
}
