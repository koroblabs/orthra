//! An escape a host's comments and raw strings write out is an escape there too, where the host
//! says its text has them: its letters are not the word's after it.

use expect_test::expect;
use orthra_testkit::annotate::Annotation;

use super::subject::packs;
use crate::Engine;

#[test]
fn go_reads_an_escape_written_out_as_an_escape() {
    let text =
        "package p\n\n// Writes `\\ufffd`, then \"\\nqzxvword\".\nvar re = `\\d+\\nqzxvraw`\n";
    let mut engine = Engine::with_packs(packs());
    let annotations: Vec<Annotation> = engine
        .check("go", text)
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect![[r#"
        > package p
        >
        > // Writes `\ufffd`, then "\nqzxvword".
                                      ^^^^^^^^ unknown-word
        > var re = `\d+\nqzxvraw`
                         ^^^^^^^ unknown-word
    "#]]
    .assert_eq(&Annotation::render(text, &annotations));
}
