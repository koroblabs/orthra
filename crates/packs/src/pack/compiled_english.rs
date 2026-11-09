//! English's dictionaries are compiled when orthra is (`build.rs`), and what is embedded accepts
//! what spellbook accepts from the same files.

use orthra_dict::{
    Dictionary,
    compiled::{Bytes, CompiledDict},
    hunspell::HunspellDict,
};

const US_AFF: &str = include_str!("../../../../packs/en-US/en_US.aff");
const GB_AFF: &str = include_str!("../../../../packs/en-GB/en_GB.aff");
const TYPOS: &str = include_str!("../../../../packs/en-US/typos.tsv");

/// The embedded dictionary `name` of pack `tag`, with its marks.
fn embedded(tag: &str, name: &str, aff: &str) -> CompiledDict {
    let file = |file: String| {
        crate::builtin::COMPILED
            .iter()
            .find(|(path, _)| *path == format!("{tag}/{file}"))
            .map(|(_, bytes)| Bytes::Embedded(bytes))
    };
    CompiledDict::new(
        aff,
        file(format!("{name}.fst")).expect("compiled when orthra was"),
        file(format!("{name}.marks.fst")),
    )
    .unwrap()
}

/// Every third stem of `dic`, as written, in lower case, in capitals and title-cased, and every
/// third known typo, as written and in capitals: the words on which the two must agree. Every stem
/// and every typo, 330 000 to 390 000 words a dictionary, agreed on 2026-11-09; a third keeps the
/// test to seconds in a debug build.
fn words(dic: &str) -> Vec<String> {
    let mut words = Vec::new();
    for line in dic.lines().skip(1).step_by(3) {
        let stem = line.split('/').next().unwrap_or_default();
        let mut chars = stem.chars();
        let title: String = chars
            .next()
            .map(|first| first.to_uppercase().chain(chars).collect())
            .unwrap_or_default();
        words.extend([
            stem.to_owned(),
            stem.to_lowercase(),
            stem.to_uppercase(),
            title,
        ]);
    }
    for line in TYPOS.lines().step_by(3) {
        let typo = line.split('\t').next().unwrap_or_default();
        words.extend([typo.to_owned(), typo.to_uppercase()]);
    }
    words
}

#[test]
fn each_compiled_english_dictionary_accepts_what_spellbook_accepts() {
    let dictionaries = [
        (
            "en-US",
            "en_US",
            US_AFF,
            include_str!("../../../../packs/en-US/en_US.dic"),
        ),
        (
            "en-US",
            "en_US-rare",
            US_AFF,
            include_str!("../../../../packs/en-US/en_US-rare.dic"),
        ),
        (
            "en-GB",
            "en_GB",
            GB_AFF,
            include_str!("../../../../packs/en-GB/en_GB.dic"),
        ),
    ];
    let mut disagree = Vec::new();
    for (tag, name, aff, dic) in dictionaries {
        let compiled = embedded(tag, name, aff);
        let hunspell = HunspellDict::new(aff, dic).unwrap();
        disagree.extend(
            words(dic)
                .into_iter()
                .filter(|word| compiled.check(word) != hunspell.check(word))
                .map(|word| format!("{name} {word:?}: spellbook {}", hunspell.check(&word))),
        );
    }
    assert!(disagree.is_empty(), "{}", disagree.join("\n"));
}
