//! The smoke test every pack gets (`CLAUDE.md` §8.1): a dictionary that loads can still accept
//! nothing, so ten words it must accept and ten it must not.

use std::fmt::Write as _;

use expect_test::expect;

use orthra_text::script::Script;

use crate::Packs;

#[test]
fn en_us_accepts_the_words_and_refuses_the_typos() {
    let packs = Packs::builtin(&["en-US"]).unwrap();
    let en = packs.iter().next().unwrap();
    let mut out = String::new();
    for word in [
        "receive",
        "definitely",
        "separate",
        "the",
        "occurred",
        "doesn't",
        "doesn’t",
        "colors",
        "Ukraine",
        "iterator",
        "toolchain",
        "PostgreSQL",
        "Kubectl",
        "recieve",
        "definately",
        "seperate",
        "teh",
        "occured",
        "wierd",
        "acheive",
        "untill",
        "begining",
        "accomodate",
        "vanishingly",
        "integrations",
        "incudes",
    ] {
        let fixes = en
            .known_typo(word)
            .map(|fixes| fixes.iter().collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        writeln!(
            out,
            "{word:<12} accepted={:<5} rare={:<5} known-typo=[{fixes}]",
            en.accepts(word),
            en.accepts_rare(word)
        )
        .unwrap();
    }
    expect![[r#"
        receive      accepted=true  rare=true  known-typo=[]
        definitely   accepted=true  rare=true  known-typo=[]
        separate     accepted=true  rare=false known-typo=[]
        the          accepted=true  rare=false known-typo=[]
        occurred     accepted=true  rare=false known-typo=[]
        doesn't      accepted=true  rare=false known-typo=[]
        doesn’t      accepted=true  rare=false known-typo=[]
        colors       accepted=true  rare=false known-typo=[]
        Ukraine      accepted=true  rare=false known-typo=[]
        iterator     accepted=true  rare=false known-typo=[]
        toolchain    accepted=true  rare=true  known-typo=[]
        PostgreSQL   accepted=true  rare=false known-typo=[]
        Kubectl      accepted=true  rare=true  known-typo=[]
        recieve      accepted=false rare=false known-typo=[receive]
        definately   accepted=false rare=false known-typo=[definitely]
        seperate     accepted=false rare=false known-typo=[separate]
        teh          accepted=false rare=false known-typo=[the]
        occured      accepted=false rare=false known-typo=[occurred]
        wierd        accepted=false rare=false known-typo=[weird]
        acheive      accepted=false rare=false known-typo=[achieve]
        untill       accepted=false rare=true  known-typo=[until]
        begining     accepted=false rare=false known-typo=[beginning]
        accomodate   accepted=false rare=false known-typo=[accommodate]
        vanishingly  accepted=false rare=true  known-typo=[]
        integrations accepted=false rare=true  known-typo=[]
        incudes      accepted=false rare=true  known-typo=[includes]
    "#]]
    .assert_eq(&out);
}

#[test]
fn en_gb_accepts_british_spelling_and_shares_english_typos_and_vocabulary() {
    let packs = Packs::builtin(&["en-GB"]).unwrap();
    let gb = packs.iter().next().unwrap();
    let mut out = String::new();
    for word in [
        "colour",
        "behaviour",
        "judgement",
        "analysed",
        "neighbour",
        "toolchain",
        "recieve",
        "teh",
    ] {
        let fixes = gb
            .known_typo(word)
            .map(|fixes| fixes.iter().collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        writeln!(
            out,
            "{word:<10} accepted={:<5} known-typo=[{fixes}]",
            gb.accepts(word)
        )
        .unwrap();
    }
    expect![[r#"
        colour     accepted=true  known-typo=[]
        behaviour  accepted=true  known-typo=[]
        judgement  accepted=true  known-typo=[]
        analysed   accepted=true  known-typo=[]
        neighbour  accepted=true  known-typo=[]
        toolchain  accepted=true  known-typo=[]
        recieve    accepted=false known-typo=[receive]
        teh        accepted=false known-typo=[the]
    "#]]
    .assert_eq(&out);
}

#[test]
fn a_language_without_a_built_in_pack_is_an_error_not_a_silence() {
    let err = Packs::builtin(&["en-US", "xx-XX"]).unwrap_err();
    assert_eq!(
        err.0,
        r#"no pack for "xx-XX"; the packs are ["en-GB", "en-US", "uk-UA"]"#
    );
    let err = Packs::builtin(&["uk-UA"]).unwrap_err();
    assert_eq!(
        err.0,
        "uk-UA is installed, not built in: `orthra pack install uk-UA`"
    );
}

/// What en-US accepts as it accepts a rare word, after the known typos have had their say: what
/// SCOWL size 80 adds, the names of commands, a prefix on a word it accepts, and a suffix on a
/// vocabulary word. A base under four letters, and a suffix on a dictionary word, make nothing.
#[test]
fn en_us_accepts_derived_words_as_rare_words() {
    let packs = Packs::builtin(&["en-US"]).unwrap();
    let en = packs.iter().next().unwrap();
    let mut out = String::new();
    for word in [
        "chipset",
        "journalctl",
        "unconfigured",
        "Unconfigured",
        "REFETCHED",
        "subpool",
        "multihost",
        "datagrams",
        "demuxes",
        "recod",
        "reson",
        "informations",
        "advices",
        "feedbacks",
        "busctl",
        "unqzxv",
    ] {
        let fixes = en
            .known_typo(word)
            .map(|fixes| fixes.iter().collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        writeln!(
            out,
            "{word:<12} accepted={:<5} rare={:<5} known-typo=[{fixes}]",
            en.accepts(word),
            en.accepts_rare(word)
        )
        .unwrap();
    }
    expect![[r#"
        chipset      accepted=false rare=true  known-typo=[]
        journalctl   accepted=false rare=true  known-typo=[]
        unconfigured accepted=false rare=true  known-typo=[]
        Unconfigured accepted=false rare=true  known-typo=[]
        REFETCHED    accepted=false rare=true  known-typo=[]
        subpool      accepted=false rare=true  known-typo=[]
        multihost    accepted=false rare=true  known-typo=[]
        datagrams    accepted=false rare=true  known-typo=[]
        demuxes      accepted=false rare=true  known-typo=[]
        recod        accepted=false rare=false known-typo=[record]
        reson        accepted=false rare=false known-typo=[reason]
        informations accepted=false rare=false known-typo=[information]
        advices      accepted=false rare=true  known-typo=[advice, advises]
        feedbacks    accepted=false rare=false known-typo=[]
        busctl       accepted=false rare=true  known-typo=[]
        unqzxv       accepted=false rare=false known-typo=[]
    "#]]
    .assert_eq(&out);
}

/// A word in the form its script's packs check it in: English writes its apostrophe two ways, and
/// a word no English pack checks is left as it is.
#[test]
fn a_word_is_normalized_by_the_packs_for_its_script() {
    let packs = Packs::builtin(&["en-US"]).unwrap();
    let shown: Vec<String> = [
        (Script::Latin, "o’qzxv"),
        (Script::Latin, "o'qzxv"),
        (Script::Latin, "oʼqzxv"),
        (Script::Cyrillic, "м’ята"),
    ]
    .iter()
    .map(|(script, word)| format!("{word} -> {}", packs.normalized(*script, word)))
    .collect();
    expect![[r#"
        o’qzxv -> o'qzxv
        o'qzxv -> o'qzxv
        oʼqzxv -> oʼqzxv
        м’ята -> м’ята"#]]
    .assert_eq(&shown.join("\n"));
}
