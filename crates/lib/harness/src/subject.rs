//! What the dimensions need from whatever is under test.

use orthra_testkit::{annotate::Annotation, fixture::Fixture};
use orthra_text::TextRange;

/// Which tier said it. The context tier's findings carry their own LSP `source`, and
/// `context-off ⊆ context-on` is checked against this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    Lexical,
    Context,
}

/// One finding, as the harness sees it: where, which tier, and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observed {
    pub range: TextRange,
    pub source: Source,
    pub label: String,
}

impl Observed {
    pub fn lexical(range: TextRange, label: impl Into<String>) -> Observed {
        Observed {
            range,
            source: Source::Lexical,
            label: label.into(),
        }
    }

    pub fn context(range: TextRange, label: impl Into<String>) -> Observed {
        Observed {
            range,
            source: Source::Context,
            label: label.into(),
        }
    }

    /// Findings in the one order every comparison and snapshot uses.
    pub fn sorted(mut findings: Vec<Observed>) -> Vec<Observed> {
        findings.sort_by(|a, b| {
            (a.range.start(), a.range.end(), a.source, &a.label).cmp(&(
                b.range.start(),
                b.range.end(),
                b.source,
                &b.label,
            ))
        });
        findings
    }

    /// `text` with `findings` under it. A context finding says so.
    pub fn render(text: &str, findings: &[Observed]) -> String {
        let annotations: Vec<Annotation> = findings
            .iter()
            .map(|finding| match finding.source {
                Source::Lexical => Annotation::new(finding.range, finding.label.clone()),
                Source::Context => {
                    Annotation::new(finding.range, format!("{} [context]", finding.label))
                }
            })
            .collect();
        Annotation::render(text, &annotations)
    }
}

/// One edit, as LSP sends it: replace `range` of the current text with `insert`. lspkit-fuzz's,
/// which the sweep draws from.
pub use lspkit_fuzz::Edit;

/// Whatever is under test: the engine, or a test double of it.
pub trait Subject {
    type Session: Session;

    /// Open `text` as the fixture's subject, with every other file of the fixture as its
    /// workspace. The text is passed separately because `shift` and `incremental` open the subject
    /// with text the fixture does not hold.
    fn open(&self, fixture: &Fixture, text: &str) -> Self::Session;

    /// A line that is inert in the subject's host, holding a character that is wider than a byte
    /// and one that is wider than a UTF-16 unit. `None` when the host has no such line (JSON has no
    /// comments), which skips `shift` for this fixture.
    fn neutral_line(&self, fixture: &Fixture) -> Option<String>;
}

/// One open document.
pub trait Session {
    fn text(&self) -> &str;

    fn edit(&mut self, edit: &Edit);

    fn findings(&mut self) -> Vec<Observed>;

    /// What a configuration change does to every cache, with nothing actually changed.
    fn bump_epoch(&mut self);

    /// The context tier, on or off. A new session starts with it off.
    fn set_context(&mut self, on: bool);
}
