//! A rare word, from the larger dictionary, is a word unless the known typos name it: SCOWL's
//! uncommon words are often misspellings of common ones. The main dictionary's words are words
//! whatever the typo list says.

use expect_test::expect;
use orthra_testkit::annotate::Annotation;

use super::subject::packs;
use crate::Engine;

#[test]
fn a_rare_word_is_a_word_unless_it_is_a_known_typo() {
    let text = "Integrations vanish vanishingly, decrypting; incudes and algin do not.\n";
    let mut engine = Engine::with_packs(packs());
    let annotations: Vec<Annotation> = engine
        .check("plaintext", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > Integrations vanish vanishingly, decrypting; incudes and algin do not.
                                                       ^^^^^^^ known-typo
                                                                   ^^^^^ known-typo
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}
