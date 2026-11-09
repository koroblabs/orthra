//! A words file's words are words everywhere: in prose, in code, at a declaration, and when a
//! pack lists one as a known typo. Somebody wrote each one down on purpose.

use expect_test::expect;
use orthra_testkit::annotate::Annotation;

use super::subject::packs;
use crate::Engine;

#[test]
fn a_words_file_word_is_a_word_everywhere() {
    let text = "// Recieve the qzxv, then teh rest.\nfn recieve_qzxv(qzxvlater: u8) {}\n";
    let mut engine = Engine::with_packs(packs());
    engine.set_words(["recieve", "QZXV"]);
    let annotations: Vec<Annotation> = engine
        .check("rust", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > // Recieve the qzxv, then teh rest.
                                    ^^^ known-typo
        > fn recieve_qzxv(qzxvlater: u8) {}
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}

#[test]
fn a_file_says_what_it_ignores() {
    let text = "Teh recieved qzxv and qzxva; the directive is no finding.\n<!-- orthra: ignore qzxv, Recieved -->\n";
    let mut engine = Engine::with_packs(packs());
    let annotations: Vec<Annotation> = engine
        .check("markdown", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > Teh recieved qzxv and qzxva; the directive is no finding.
          ^^^ known-typo
                                ^^^^^ unknown-word
        > <!-- orthra: ignore qzxv, Recieved -->
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}

/// A project that declares a known typo, as a test's `fn fix_recieve` does, has it reported where
/// it is declared and everywhere else; an unknown word it declares is a word everywhere else.
#[test]
fn a_declared_known_typo_is_never_accepted() {
    let mut engine = Engine::with_packs(packs());
    engine.learn("rust", "fn fix_recieve(qzxvfixture: u8) {}\n");
    let text = "// `RECIEVE`, recieve and qzxvfixture.\nfn fix_recieve() {}\n";
    let annotations: Vec<Annotation> = engine
        .check("rust", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > // `RECIEVE`, recieve and qzxvfixture.
              ^^^^^^^ known-typo
                        ^^^^^^^ known-typo
        > fn fix_recieve() {}
                 ^^^^^^^ known-typo
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}

/// `orthra: ignore-next-line` ignores every word of the one line after it, known typos included;
/// the line after that is checked again, and the directive's own line is not prose.
#[test]
fn a_directive_ignores_the_next_line() {
    let text =
        "// orthra: ignore-next-line\n// Teh qzxv recieved it.\n// Teh qzxv again.\nfn f() {}\n";
    let mut engine = Engine::with_packs(packs());
    let annotations: Vec<Annotation> = engine
        .check("rust", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > // orthra: ignore-next-line
        > // Teh qzxv recieved it.
        > // Teh qzxv again.
             ^^^ known-typo
                 ^^^^ unknown-word
        > fn f() {}
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}

/// A listed word is a word in every part of a run it names whole: `TruStage` is cut into `Stage`
/// and `Tru`, and the list can only name it as written. `Tru` alone is still a typo.
#[test]
fn a_words_file_word_is_a_word_in_every_part() {
    let text = "Tru alone is a typo.\nTruStage's rates, from TruStage.\n";
    let mut engine = Engine::with_packs(packs());
    engine.set_words(["TruStage"]);
    let annotations: Vec<Annotation> = engine
        .check("markdown", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > Tru alone is a typo.
          ^^^ known-typo
        > TruStage's rates, from TruStage.
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}

/// So is a word a file says to ignore.
#[test]
fn a_directive_word_is_a_word_in_every_part() {
    let text = "<!-- orthra: ignore TruStage -->\nTru alone is a typo.\nTruStage's rates, from TruStage.\n";
    let mut engine = Engine::with_packs(packs());
    let annotations: Vec<Annotation> = engine
        .check("markdown", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > <!-- orthra: ignore TruStage -->
        > Tru alone is a typo.
          ^^^ known-typo
        > TruStage's rates, from TruStage.
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}
