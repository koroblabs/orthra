//! The four dimensions, and the one call that runs a fixture through all of them.

use std::fmt;

use lspkit_fuzz::{Files, Sweep};
use orthra_testkit::fixture::Fixture;
use orthra_text::{
    TextSize,
    line_index::{Encoding, LineCol, LineIndex},
};

use crate::subject::{Edit, Observed, Session, Source, Subject};

/// How many fuzzed edits `check_all` puts every fixture through, beside the ones it is given.
const FUZZ_EDITS: usize = 24;

/// What the fuzzed edits paste in. It is what breaks a spell checker's incremental state: a typo
/// appearing and disappearing, a word cut in half or joined to its neighbor, a newline splitting a
/// span, quotes and comment markers changing what a host considers prose, and characters wider than
/// a byte.
const PALETTE: &[&str] = &[
    "", "e", "teh", " ", "\n", "е", "👋", "'", "-", "\"", "//", "the ",
];

/// One dimension that did not hold, with both sides rendered so the difference is readable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub dimension: &'static str,
    pub detail: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` does not hold\n{}", self.dimension, self.detail)
    }
}

/// Every dimension over one fixture, plus [`FUZZ_EDITS`] fuzzed edits. Returns the subject's
/// findings for the fixture as written, for the caller to snapshot.
///
/// # Panics
///
/// With every violation of the other dimensions at once, not just the first one; and, once those
/// hold, with the seed, the edit and the shrunk list of edits of a fuzzed sequence that went wrong.
pub fn check_all<S: Subject>(subject: &S, fixture: &Fixture, edits: &[Edit]) -> Vec<Observed> {
    let text = &fixture.subject().text;
    let violations: Vec<String> = [
        incremental(subject, fixture, edits),
        cold_warm(subject, fixture),
        shift(subject, fixture),
        context_subset(subject, fixture),
    ]
    .into_iter()
    .filter_map(Result::err)
    .map(|violation| violation.to_string())
    .collect();
    assert!(violations.is_empty(), "{}", violations.join("\n\n"));
    fuzzed(subject, fixture);
    Observed::sorted(subject.open(fixture, text).findings())
}

/// `incremental == from-scratch`: a document edited live must say exactly what a fresh one opened
/// on the final text says. Findings are asked for after every edit, so a cache gets every chance
/// to go stale.
pub fn incremental<S: Subject>(
    subject: &S,
    fixture: &Fixture,
    edits: &[Edit],
) -> Result<(), Violation> {
    const NAME: &str = "incremental == from-scratch";
    let mut expected_text = fixture.subject().text.clone();
    let mut live = subject.open(fixture, &expected_text);
    live.findings();
    for edit in edits {
        live.edit(edit);
        edit.apply(&mut expected_text);
        live.findings();
    }
    if live.text() != expected_text {
        return Err(Violation {
            dimension: NAME,
            detail: format!(
                "the session's text drifted from the edits applied to it\nexpected:\n{expected_text}\nactual:\n{}",
                live.text()
            ),
        });
    }
    let incremental = Observed::sorted(live.findings());
    let scratch = Observed::sorted(subject.open(fixture, &expected_text).findings());
    compare(
        NAME,
        &expected_text,
        ("from scratch", &scratch),
        ("incremental", &incremental),
    )
}

/// `incremental == from-scratch` under [`FUZZ_EDITS`] seeded random edits to the fixture's subject,
/// compared after each one: lspkit-fuzz's sweep, which names the seed and the edit that went wrong
/// and shrinks the list to the edits that still fail. The sweep also holds the session's text to
/// the edits applied to it, since both renderings begin with it.
///
/// # Panics
///
/// When the live session and a fresh one disagree.
pub fn fuzzed<S: Subject>(subject: &S, fixture: &Fixture) {
    let (path, text) = (&fixture.subject().path, &fixture.subject().text);
    let files: Files = [(path.clone(), text.clone())].into();
    let mut sweep = lspkit_fuzz::Subject {
        open: |files: &Files| subject.open(fixture, &files[path]),
        edited: |live: &mut S::Session, _: &str, edit: &Edit, _: &str| live.edit(edit),
        answer: |live: &mut S::Session| {
            let findings = Observed::sorted(live.findings());
            Rendered(show(live.text(), &findings))
        },
        scratch: |files: &Files| {
            let findings = Observed::sorted(subject.open(fixture, &files[path]).findings());
            Rendered(show(&files[path], &findings))
        },
    };
    Sweep {
        seeds: &[fuzz_seed(path)],
        edits: FUZZ_EDITS,
        palette: PALETTE,
        span: 3,
    }
    .run(&files, &mut sweep);
}

/// A text with its findings under it, as the sweep compares and prints it.
#[derive(PartialEq)]
struct Rendered(String);

impl fmt::Display for Rendered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// `cold == warm`: asking twice, and asking again after an epoch bump, changes nothing.
pub fn cold_warm<S: Subject>(subject: &S, fixture: &Fixture) -> Result<(), Violation> {
    const NAME: &str = "cold == warm";
    let text = &fixture.subject().text;
    let mut session = subject.open(fixture, text);
    let cold = Observed::sorted(session.findings());
    let warm = Observed::sorted(session.findings());
    compare(NAME, text, ("cold", &cold), ("warm", &warm))?;
    session.bump_epoch();
    let bumped = Observed::sorted(session.findings());
    compare(
        NAME,
        text,
        ("cold", &cold),
        ("after an epoch bump", &bumped),
    )
}

/// `shift`: the same document under a neutral line of multi-byte text says the same things, one
/// line down, at the same column in every encoding. An ASCII fixture cannot tell a byte offset from
/// a character offset; this dimension makes every fixture able to.
pub fn shift<S: Subject>(subject: &S, fixture: &Fixture) -> Result<(), Violation> {
    const NAME: &str = "shift";
    let Some(line) = subject.neutral_line(fixture) else {
        return Ok(());
    };
    assert!(
        line.ends_with('\n') && line.matches('\n').count() == 1,
        "a neutral line is one line, with its terminator: {line:?}"
    );
    assert!(
        line.chars().any(|c| c.len_utf8() > 1 && c.len_utf16() == 1)
            && line.chars().any(|c| c.len_utf16() == 2),
        "a neutral line needs a multi-byte character and an astral one, or it shifts nothing an \
         encoding bug could see: {line:?}"
    );

    let text = &fixture.subject().text;
    let shifted_text = format!("{line}{text}");
    let by = TextSize::of(line.as_str());
    let base = Observed::sorted(subject.open(fixture, text).findings());
    let shifted = Observed::sorted(subject.open(fixture, &shifted_text).findings());
    let expected: Vec<Observed> = base
        .iter()
        .map(|finding| Observed {
            range: finding.range + by,
            ..finding.clone()
        })
        .collect();
    compare(
        NAME,
        &shifted_text,
        ("the original, one line down", &expected),
        ("shifted", &shifted),
    )?;

    // And as a client sees them: the same column, one line down, in every encoding.
    let (before, after) = (LineIndex::new(text), LineIndex::new(&shifted_text));
    for finding in &base {
        for encoding in Encoding::ALL {
            let want = before
                .line_col_in(text, finding.range.start(), encoding)
                .map(|pos| LineCol {
                    line: pos.line + 1,
                    ..pos
                });
            let got = after.line_col_in(&shifted_text, finding.range.start() + by, encoding);
            if want != got {
                return Err(Violation {
                    dimension: NAME,
                    detail: format!("{finding:?} in {encoding:?}: expected {want:?}, got {got:?}"),
                });
            }
        }
    }
    Ok(())
}

/// `context-off ⊆ context-on`: turning the context tier on may add findings of its own, and may
/// never remove, move or reword a lexical one — whether it is turned on mid-session or from the
/// start.
pub fn context_subset<S: Subject>(subject: &S, fixture: &Fixture) -> Result<(), Violation> {
    const NAME: &str = "context-off ⊆ context-on";
    let text = &fixture.subject().text;
    let mut session = subject.open(fixture, text);
    session.set_context(false);
    let off = Observed::sorted(session.findings());
    if let Some(stray) = off.iter().find(|finding| finding.source == Source::Context) {
        return Err(Violation {
            dimension: NAME,
            detail: format!("a context finding with the context tier off: {stray:?}"),
        });
    }

    session.set_context(true);
    let toggled = lexical(session.findings());
    compare(
        NAME,
        text,
        ("context off", &off),
        ("turned on mid-session, lexical only", &toggled),
    )?;

    let mut fresh = subject.open(fixture, text);
    fresh.set_context(true);
    let from_start = lexical(fresh.findings());
    compare(
        NAME,
        text,
        ("context off", &off),
        ("on from the start, lexical only", &from_start),
    )
}

fn lexical(findings: Vec<Observed>) -> Vec<Observed> {
    Observed::sorted(
        findings
            .into_iter()
            .filter(|finding| finding.source == Source::Lexical)
            .collect(),
    )
}

fn compare(
    dimension: &'static str,
    text: &str,
    (expected_name, expected): (&str, &[Observed]),
    (actual_name, actual): (&str, &[Observed]),
) -> Result<(), Violation> {
    if expected == actual {
        return Ok(());
    }
    Err(Violation {
        dimension,
        detail: format!(
            "{expected_name}:\n{}{actual_name}:\n{}",
            show(text, expected),
            show(text, actual)
        ),
    })
}

/// A rendering when every range can be drawn, and a list when one cannot. The report of a
/// violation must not panic: the broken range is usually the violation.
fn show(text: &str, findings: &[Observed]) -> String {
    let drawable = findings.iter().all(|finding| {
        text.get(std::ops::Range::<usize>::from(finding.range))
            .is_some()
    });
    if drawable {
        Observed::render(text, findings)
    } else {
        format!(
            "(a range is outside the text or inside a character, so as a list)\n{findings:#?}\n"
        )
    }
}

/// A seed per fixture, so two fixtures get different edits and one fixture always gets the same.
fn fuzz_seed(path: &str) -> u64 {
    path.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}
