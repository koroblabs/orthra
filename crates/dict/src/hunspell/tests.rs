use super::HunspellDict;
use crate::Dictionary;

/// A dictionary small enough to read: one suffix rule, three stems.
const AFF: &str = "SET UTF-8\nTRY esianrtolcdugmphbyfvkwz\n\nSFX S Y 1\nSFX S 0 s .\n";
const DIC: &str = "3\nreceive/S\ncache/S\nUkraine\n";

#[test]
fn it_accepts_stems_and_their_affixed_forms_and_nothing_else() {
    let dict = HunspellDict::new(AFF, DIC).unwrap();
    for good in ["receive", "receives", "cache", "caches", "Ukraine"] {
        assert!(dict.check(good), "{good} should be accepted");
    }
    for bad in ["recieve", "ukraine", "cachez", "receiving"] {
        assert!(!dict.check(bad), "{bad} should be refused");
    }
}

#[test]
fn it_suggests_the_stem_a_typo_was_meant_to_be_with_its_distance() {
    let dict = HunspellDict::new(AFF, DIC).unwrap();
    let mut out = Vec::new();
    dict.suggest("recieve", &mut out);
    let shown: Vec<String> = out
        .iter()
        .map(|candidate| format!("{} {}", candidate.word, candidate.edits))
        .collect();
    expect_test::expect!["receive 1"].assert_eq(&shown.join("\n"));
}

#[test]
fn a_dictionary_that_does_not_parse_says_so() {
    let err = HunspellDict::new("FLAG long\nSFX AB Y 1\nSFX AB 0 s .\n", "nonsense").unwrap_err();
    assert!(
        err.0
            .starts_with("a Hunspell dictionary that does not parse"),
        "{err}"
    );
}
