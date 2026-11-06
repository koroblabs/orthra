//! `explain` says which rule decided a word, by the same function a check asks: one word for each
//! rule, and each reason an unknown word goes unreported.

use expect_test::expect;
use orthra_text::TextSize;

use super::subject::packs;
use crate::Engine;

#[test]
fn every_rule_that_decides_a_word_is_named() {
    let text = "\
// Listed qzxlisted, declared blorpify and qzxdep, PostgreSQL, async, and grumblefest.
// orthra: ignore qzxignored
// Then qzxignored, and the rest is code.
fn blorpify(rdr: u8) -> &'static str { \"teh qzxdata\" }
";
    let mut engine = Engine::with_packs(packs());
    engine.set_words(["qzxlisted"]);
    engine.declare_name("qzxdep");
    let explained: Vec<String> = [
        "qzxlisted",
        "blorpify and",
        "qzxdep",
        "Postgre",
        "async",
        "grumblefest",
        "qzxignored,",
        "fn blorpify",
        "rdr",
        "teh",
        "qzxdata",
    ]
    .iter()
    .map(|marker| {
        // `fn blorpify` is the declaration, three bytes into its marker.
        let skip = if marker.starts_with("fn ") { 3 } else { 0 };
        let at = text.find(marker).expect("the marker is in the text") + skip;
        let explanation = engine
            .explain(
                "/src/tests/lib_test.rs",
                text,
                TextSize::try_from(at).unwrap(),
            )
            .expect("a host claims a .rs file");
        let word = explanation.word.expect("a word at the marker");
        format!(
            "{:<12} {:?} by {:?}, unreported {:?}, finding {:?}",
            word.text,
            word.verdict,
            word.why,
            word.silenced,
            word.finding.map(|finding| finding.code.0)
        )
    })
    .collect();
    expect![[r#"
        qzxlisted    Correct by WordsFile, unreported None, finding None
        blorpify     Correct by DeclaredHere, unreported None, finding None
        qzxdep       Correct by DeclaredByProject, unreported None, finding None
        Postgre      Correct by WholeRun, unreported None, finding None
        async        Correct by HostVocabulary, unreported None, finding None
        grumblefest  Unknown by Packs, unreported None, finding Some("unknown-word")
        qzxignored   Correct by Directive, unreported None, finding None
        blorpify     Unknown by Packs, unreported Some(Code), finding None
        rdr          Unknown by Packs, unreported Some(Code), finding None
        teh          KnownTypo by Packs, unreported Some(TestData), finding None
        qzxdata      Unknown by Packs, unreported Some(TestData), finding None"#]]
    .assert_eq(&explained.join("\n"));
}
