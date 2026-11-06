//! The incumbent corpus, run through the engine: which cases pass, which do not, and which wait for
//! a host. A scoreboard, not a test of each case, so it moves as the engine does and every change
//! to it is a line in a diff.

use std::{fmt::Write as _, path::Path};

use expect_test::expect;
use orthra_testkit::{annotate::Annotation, corpus::Case};

use super::subject::{Orthra, packs};
use crate::Engine;

const INCUMBENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpora/incumbent");

#[test]
fn incumbent_scoreboard() {
    let mut board = String::new();
    for case in Case::load_dir(Path::new(INCUMBENT)) {
        let subject = case.fixture.subject();
        let mut engine = Engine::with_packs(packs());
        let language = subject
            .host
            .clone()
            .or_else(|| engine.language_for_path(&subject.path).map(str::to_owned));
        let Some(language) = language else {
            writeln!(
                board,
                "{:<38} waits for a host ({})",
                case.name, subject.path
            )
            .unwrap();
            continue;
        };
        let wants_ukrainian = case
            .fixture
            .files()
            .iter()
            .any(|f| f.path == "/orthra.toml" && f.text.contains("uk-UA"));
        if wants_ukrainian {
            writeln!(board, "{:<38} waits for uk-UA (Phase 4)", case.name).unwrap();
            continue;
        }
        let found: Vec<Annotation> = engine
            .check(&language, &subject.text)
            .iter()
            .map(|finding| Annotation::new(finding.range, Orthra::label(finding)))
            .collect();
        let expected = case.expected();
        if found == expected {
            writeln!(board, "{:<38} passes", case.name).unwrap();
            continue;
        }
        let show = |a: &Annotation| format!("{} `{}`", a.label, &subject.text[a.range]);
        let extra: Vec<String> = found
            .iter()
            .filter(|a| !expected.contains(a))
            .map(show)
            .collect();
        let missing: Vec<String> = expected
            .iter()
            .filter(|a| !found.contains(a))
            .map(show)
            .collect();
        writeln!(
            board,
            "{:<38} extra [{}] missing [{}]",
            case.name,
            extra.join(", "),
            missing.join(", ")
        )
        .unwrap();
    }
    expect![[r#"
        01-escapes-and-placeholders            passes
        02-identifier-shapes                   passes
        03-use-sites-are-not-ours              passes
        04-urls-paths-hashes                   passes
        05-markdown-code-is-not-prose          passes
        06-ukrainian-comment                   waits for uk-UA (Phase 4)
        07-ukrainian-without-the-pack          passes
        08-mixed-script                        waits for uk-UA (Phase 4)
        09-technical-vocabulary                passes
        10-acronyms-and-brands                 passes
        11-known-typos-keep-their-case         passes
        12-possessives-hyphens-contractions    passes
        13-commit-message                      passes
        14-numbers-and-units                   passes
        15-camel-case-declarations             passes
        16-curly-apostrophes                   passes
        17-contractions-in-escaped-strings     passes
        18-uuids-in-every-form                 passes
        19-random-identifiers                  passes
    "#]]
    .assert_eq(&board);
}
