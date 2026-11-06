//! Every dimension catches the defect it exists for, and only that one. A dimension that cannot
//! fail is a comment, so each gets a planted bug before anything real depends on it.

use expect_test::expect;
use orthra_testkit::fixture::Fixture;

use super::toy::{Bug, FIXTURE, Toy};
use crate::{
    dimensions::{check_all, cold_warm, context_subset, incremental, shift},
    subject::Edit,
};

fn fixture() -> Fixture {
    Fixture::parse(FIXTURE)
}

/// One typo fixed, the edit a stale cache gets wrong.
fn fix_recieve(fixture: &Fixture) -> [Edit; 1] {
    [Edit::replace(&fixture.subject().text, "recieve", "receive")]
}

#[test]
fn a_memo_an_edit_forgets_to_drop_fails_incremental() {
    let (toy, fixture) = (Toy(Bug::StaleAfterEdit), fixture());
    assert!(cold_warm(&toy, &fixture).is_ok());
    assert!(shift(&toy, &fixture).is_ok());
    assert!(context_subset(&toy, &fixture).is_ok());

    let violation = incremental(&toy, &fixture, &fix_recieve(&fixture)).unwrap_err();
    expect![[r#"
        `incremental == from-scratch` does not hold
        from scratch:
        > The cat will receive their food.
        > Teh end: teh.
          ^^^ known-typo
                   ^^^ known-typo
        incremental:
        > The cat will receive their food.
                       ^^^^^^^ known-typo
        > Teh end: teh.
          ^^^ known-typo
                   ^^^ known-typo
    "#]]
    .assert_eq(&violation.to_string());
}

#[test]
fn character_offsets_pass_every_ascii_fixture_and_fail_shift() {
    let (toy, fixture) = (Toy(Bug::CharOffsets), fixture());
    assert!(incremental(&toy, &fixture, &fix_recieve(&fixture)).is_ok());
    assert!(cold_warm(&toy, &fixture).is_ok());
    assert!(context_subset(&toy, &fixture).is_ok());

    let violation = shift(&toy, &fixture).unwrap_err();
    expect![[r#"
        `shift` does not hold
        the original, one line down:
        > Привіт 👋
        > The cat will recieve their food.
                       ^^^^^^^ known-typo
        > Teh end: teh.
          ^^^ known-typo
                   ^^^ known-typo
        shifted:
        > Привіт 👋
        > The cat will recieve their food.
              ^^^^^^^ known-typo
                                  ^^^ known-typo
        > Teh end: teh.
          ^^^ known-typo
    "#]]
    .assert_eq(&violation.to_string());
}

#[test]
fn a_cache_keyed_by_the_word_alone_fails_cold_warm() {
    let (toy, fixture) = (Toy(Bug::CachedByWord), fixture());
    // `shift` opens a fresh session per side, so the cache is never warm there. Every dimension
    // that asks one session twice sees this bug, not only `cold == warm`; `context-off ⊆
    // context-on` does too, and that is correct, since a warm cache is warm wherever it is asked.
    assert!(shift(&toy, &fixture).is_ok());
    assert!(context_subset(&toy, &fixture).is_err());

    let violation = cold_warm(&toy, &fixture).unwrap_err();
    expect![[r#"
        `cold == warm` does not hold
        cold:
        > The cat will recieve their food.
                       ^^^^^^^ known-typo
        > Teh end: teh.
          ^^^ known-typo
                   ^^^ known-typo
        warm:
        > The cat will recieve their food.
                       ^^^^^^^ known-typo
        > Teh end: teh.
          ^^^ known-typo
          ^^^ known-typo
    "#]]
    .assert_eq(&violation.to_string());
}

#[test]
fn a_context_tier_that_rewords_a_lexical_finding_fails_the_subset() {
    let (toy, fixture) = (Toy(Bug::ContextRewords), fixture());
    assert!(incremental(&toy, &fixture, &fix_recieve(&fixture)).is_ok());
    assert!(cold_warm(&toy, &fixture).is_ok());
    assert!(shift(&toy, &fixture).is_ok());

    let violation = context_subset(&toy, &fixture).unwrap_err();
    expect![[r#"
        `context-off ⊆ context-on` does not hold
        context off:
        > The cat will recieve their food.
                       ^^^^^^^ known-typo
        > Teh end: teh.
          ^^^ known-typo
                   ^^^ known-typo
        turned on mid-session, lexical only:
        > The cat will recieve their food.
                       ^^^^^^^ known-typo (context agrees)
        > Teh end: teh.
          ^^^ known-typo (context agrees)
                   ^^^ known-typo (context agrees)
    "#]]
    .assert_eq(&violation.to_string());
}

#[test]
#[should_panic(expected = "`shift` does not hold")]
fn check_all_fails_on_any_violation() {
    check_all(&Toy(Bug::CharOffsets), &fixture(), &[]);
}

#[test]
#[should_panic(expected = "the live state and a fresh build disagree")]
fn the_fuzzed_edits_find_a_memo_an_edit_forgets_to_drop() {
    check_all(&Toy(Bug::StaleAfterEdit), &fixture(), &[]);
}
