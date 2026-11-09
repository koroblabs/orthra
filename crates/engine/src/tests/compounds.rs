//! Two common words joined are a word where code joins words, a string or a name, and a mistake
//! where prose is written: a comment's `everytime` is still reported.

use expect_test::expect;
use orthra_testkit::annotate::Annotation;

use super::subject::packs;
use crate::Engine;

#[test]
fn two_words_joined_are_a_word_only_where_code_joins_words() {
    let text = "// Run this everytime the statuscode changes.\nfn f() {\n    let a = (\"feedbacks\", \"sectionmissing\");\n}\n";
    let mut engine = Engine::with_packs(packs());
    let annotations: Vec<Annotation> = engine
        .check("rust", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > // Run this everytime the statuscode changes.
                      ^^^^^^^^^ unknown-word
                                    ^^^^^^^^^^ unknown-word
        > fn f() {
        >     let a = ("feedbacks", "sectionmissing");
                        ^^^^^^^^^ unknown-word
        > }
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}
