use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use orthra_packs::{Packs, installed};

use super::retyped;

/// uk-UA installed with a dictionary of the test's own, where the test can find it: never the
/// person's packs.
fn with_ukrainian() -> (Packs, PathBuf) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "orthra-layout-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let uk = installed::Install::of("uk-UA").unwrap().unwrap();
    let at = uk.dir(&dir);
    fs::create_dir_all(&at).unwrap();
    let (aff, dic) = ("SET UTF-8\n", "3\nпривіт\nдякую\nкеш\n");
    fs::write(at.join("uk_UA.aff"), aff).unwrap();
    fs::write(at.join("uk_UA.dic"), dic).unwrap();
    fs::write(at.join("uk_50k.txt"), "привіт 1\n").unwrap();
    fs::write(at.join("uk_UA.fst"), installed::compile(aff, dic).unwrap()).unwrap();
    let (packs, missing) = Packs::load(&["en-US", "uk-UA"], Some(&dir)).unwrap();
    assert!(missing.is_empty());
    (packs, dir)
}

#[test]
fn the_same_keys_in_the_other_layout_are_offered_when_they_make_a_word() {
    let (english, _) = Packs::load(&["en-US"], None).unwrap();
    let (both, dir) = with_ukrainian();
    let got = [
        ("en-US", "руддщ", &english),
        ("en-US", "Руддщ", &english),
        ("en-US", "цщкдв", &english),
        // Russian's own letters: `ы` is on the same key as Ukrainian `і`.
        ("en-US", "ыщгтв", &english),
        ("en-US", "вщтєе", &english),
        // A Ukrainian word typed right makes no English word: nothing to offer.
        ("en-US", "привіт", &english),
        ("en-US, uk-UA", "ghbdsn", &both),
        ("en-US, uk-UA", "Lzre.", &both),
        // A Latin word is not retyped in Cyrillic with no Cyrillic pack to accept it.
        ("en-US", "ghbdsn", &english),
        // A real typo stays a typo: its keys make no word in the other layout.
        ("en-US, uk-UA", "recieve", &both),
    ]
    .map(|(enabled, word, packs)| {
        format!(
            "{enabled:<12} {word} -> {}",
            retyped(word, packs).unwrap_or_default()
        )
    });
    expect_test::expect![[r#"
        en-US        руддщ -> hello
        en-US        Руддщ -> Hello
        en-US        цщкдв -> world
        en-US        ыщгтв -> sound
        en-US        вщтєе -> don't
        en-US        привіт -> 
        en-US, uk-UA ghbdsn -> привіт
        en-US, uk-UA Lzre. -> Дякую
        en-US        ghbdsn -> 
        en-US, uk-UA recieve -> "#]]
    .assert_eq(&got.join("\n"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_latin_slip_takes_the_keys_around_it_that_type_letters_in_the_other_layout() {
    let (both, dir) = with_ukrainian();
    let text = "Lzre. Ok, ghbdsn; lzre, then.";
    let words: Vec<(std::ops::Range<usize>, &str)> = ["Lzre", "ghbdsn", "lzre"]
        .iter()
        .scan(0, |from, word| {
            let start = *from + text[*from..].find(word).unwrap();
            *from = start + word.len();
            Some((start..*from, *word))
        })
        .collect();
    let got: Vec<String> = words
        .into_iter()
        .map(|(range, word)| {
            let at = |offset: usize| orthra_text::TextSize::try_from(offset).unwrap();
            let range = orthra_text::TextRange::new(at(range.start), at(range.end));
            let around = super::with_keys_around(text, range, orthra_text::script::Script::Latin);
            let around_text = &text[around];
            format!(
                "{word} -> {around_text:?} -> {}",
                super::retyped(around_text, &both).unwrap_or_default()
            )
        })
        .collect();
    expect_test::expect![[r#"
        Lzre -> "Lzre." -> Дякую
        ghbdsn -> "ghbdsn;" -> 
        lzre -> "lzre," -> "#]]
    .assert_eq(&got.join("\n"));
    fs::remove_dir_all(dir).unwrap();
}
