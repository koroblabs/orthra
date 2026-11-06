use std::path::Path;

use super::Case;
use crate::annotate::Annotation;

const INCUMBENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../corpora/incumbent");

#[test]
fn a_case_reads_its_metadata_and_its_expectation() {
    let case = Case::parse(
        "sample",
        "# a comment\nhypothesis: H1, H2\nsource: a probe\nincumbent: not measured yet\nwhy: because\n\n\
         //- /orthra.toml\nlanguages = [\"en-US\"]\n\n//- /a.rs  expect\n> // teh end\n     ^^^ known-typo -> the\n",
    );
    assert_eq!(case.meta["hypothesis"], "H1, H2");
    assert_eq!(case.fixture.subject().text, "// teh end\n");
    assert_eq!(case.expected().len(), 1);
    assert_eq!(case.expected()[0].label, "known-typo -> the");
}

#[test]
#[should_panic(expected = "sample: missing `why`")]
fn a_case_without_its_reason_is_refused() {
    Case::parse(
        "sample",
        "hypothesis: H1\nsource: x\nincumbent: y\n\n//- /a.rs  expect\n> fine\n",
    );
}

/// The incumbent corpus parses, and every case is written in the canonical rendering: exactly what
/// a snapshot of the right answer would print, so a failing case reads like a failing test.
#[test]
fn the_incumbent_corpus_is_well_formed() {
    let cases = Case::load_dir(Path::new(INCUMBENT));
    assert!(
        cases.len() >= 10,
        "Phase 0 asks for at least ten cases; there are {}",
        cases.len()
    );
    for case in &cases {
        let subject = case.fixture.subject();
        let rendered = Annotation::render(&subject.text, case.expected());
        let (text, annotations) = Annotation::parse(&rendered);
        assert_eq!(
            (text.as_str(), annotations.as_slice()),
            (subject.text.as_str(), case.expected()),
            "{}",
            case.name
        );
        let written =
            std::fs::read_to_string(Path::new(INCUMBENT).join(format!("{}.case", case.name)))
                .unwrap();
        assert!(
            written.ends_with(&rendered),
            "{}: the subject is not in canonical form; it should end with\n{rendered}",
            case.name
        );
    }
}
