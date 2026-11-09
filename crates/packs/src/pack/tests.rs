use crate::Packs;

#[test]
fn corrections_come_from_the_dictionary_and_the_vocabulary_nearest_first() {
    let packs = Packs::builtin(&["en-US"]).unwrap();
    let en = packs.iter().next().unwrap();
    let got: Vec<String> = [
        // In the vocabulary alone: no dictionary has it.
        "clodebase",
        // Its casing follows the word written.
        "Clodebase",
        // The dictionary's, its order kept among those as near.
        "leter",
        // A tool's name a letter short.
        "kubctl",
    ]
    .iter()
    .map(|word| {
        let mut out = Vec::new();
        en.suggest(word, 5, &mut out);
        format!("{word} -> {}", out.join(", "))
    })
    .collect();
    expect_test::expect![[r#"
        clodebase -> codebase
        Clodebase -> Codebase
        leter -> later, letter, liter, deter, meter
        kubctl -> kubectl"#]]
    .assert_eq(&got.join("\n"));
}

#[test]
fn a_plain_word_from_the_vocabulary_waits_for_nothing_nearer_and_a_name_is_offered_regardless() {
    let packs = Packs::builtin(&["en-US"]).unwrap();
    let en = packs.iter().next().unwrap();
    let got: Vec<String> = [
        // The dictionary's `Scheduler` is one edit away, so the vocabulary's `Schedule`, two, waits.
        "Schedulor",
        // The dictionary has `JavaScript` and does not suggest it.
        "javascrpt",
        // The lists of names before the languages' words: `GitHub`, then the package `gitdb`.
        "Githb",
    ]
    .iter()
    .map(|word| {
        let mut out = Vec::new();
        en.suggest(word, 5, &mut out);
        format!("{word} -> {}", out.join(", "))
    })
    .collect();
    expect_test::expect![[r#"
        Schedulor -> Scheduler
        javascrpt -> JavaScript
        Githb -> GitHub, Gitdb"#]]
    .assert_eq(&got.join("\n"));
}
