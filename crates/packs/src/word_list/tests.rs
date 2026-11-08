use expect_test::expect;

use super::WordList;

/// Every entry is found, at either end and between, and nothing else is: a word before the first,
/// after the last, between two, or an entry in capitals asked in lower case.
#[test]
fn a_word_list_finds_its_entries_by_halving() {
    let list = WordList::new("# a comment\n# another\nAAA\nbéta\nChipset\nlossy\nNASA\nzymogen\n");
    let asked = [
        "aaa", "AAA", "béta", "BÉTA", "chipset", "Chipset", "lossy", "nasa", "NASA", "zymogen",
        "a", "chips", "lossyy", "zz", "",
    ];
    let found: Vec<String> = asked
        .iter()
        .map(|word| format!("{word:<8}{}", list.accepts(word)))
        .collect();
    expect![[r#"
        aaa     false
        AAA     true
        béta    true
        BÉTA    true
        chipset true
        Chipset true
        lossy   true
        nasa    false
        NASA    true
        zymogen true
        a       false
        chips   false
        lossyy  false
        zz      false
                false
    "#]]
    .assert_eq(&(found.join("\n") + "\n"));
}
