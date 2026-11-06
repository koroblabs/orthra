//! The engine as the harness's subject, so every dimension runs on the real pipeline.

use std::sync::{Arc, OnceLock};

use orthra_harness::subject::{Edit, Observed, Session, Subject};
use orthra_packs::Packs;
use orthra_testkit::fixture::Fixture;

use crate::{Engine, Parsed};

/// Packs are read once per test process, as they are once per server.
pub(super) fn packs() -> Arc<Packs> {
    static PACKS: OnceLock<Arc<Packs>> = OnceLock::new();
    PACKS
        .get_or_init(|| Arc::new(Packs::builtin(&["en-US"]).expect("en-US loads")))
        .clone()
}

#[derive(Debug)]
pub(super) struct Orthra;

impl Orthra {
    /// The subject's language: its `host=` flag, or what its path says.
    pub(super) fn language(fixture: &Fixture) -> String {
        let subject = fixture.subject();
        subject.host.clone().unwrap_or_else(|| {
            Engine::with_packs(packs())
                .language_for_path(&subject.path)
                .unwrap_or_else(|| panic!("no host claims {}", subject.path))
                .to_owned()
        })
    }

    /// A finding as a snapshot shows it: the code, then the fixes when there are any.
    pub(super) fn label(finding: &crate::Finding) -> String {
        match finding.fixes.as_slice() {
            [] => finding.code.0.to_owned(),
            fixes => format!("{} -> {}", finding.code.0, fixes.join(", ")),
        }
    }
}

impl Subject for Orthra {
    type Session = OrthraSession;

    fn open(&self, fixture: &Fixture, text: &str) -> OrthraSession {
        OrthraSession {
            engine: Engine::with_packs(packs()),
            language: Orthra::language(fixture),
            text: text.to_owned(),
            parsed: Parsed::default(),
        }
    }

    fn neutral_line(&self, fixture: &Fixture) -> Option<String> {
        // Cyrillic with only English enabled is never checked, so the line is inert wherever it is
        // prose. In code it has to be a comment, or it is a syntax error that can change how the
        // rest of the file parses.
        let language = Orthra::language(fixture);
        let conventions = Engine::with_packs(packs()).conventions(&language)?;
        Some(match conventions.line_comment {
            Some(marker) => format!("{marker} Привіт 👋\n"),
            None => "Привіт 👋\n".to_owned(),
        })
    }
}

#[derive(Debug)]
pub(super) struct OrthraSession {
    engine: Engine,
    language: String,
    text: String,
    /// Kept between checks and told of every edit, as the server keeps it: `incremental ==
    /// from-scratch` holds a reused parse to the fresh one's findings.
    parsed: Parsed,
}

impl Session for OrthraSession {
    fn text(&self) -> &str {
        &self.text
    }

    fn edit(&mut self, edit: &Edit) {
        self.parsed.edit(&self.text, edit.range, &edit.insert);
        edit.apply(&mut self.text);
    }

    fn findings(&mut self) -> Vec<Observed> {
        let kept = self
            .engine
            .check_reusing(&self.language, &self.text, &mut self.parsed);
        // What a document's last check kept saves work and never changes an answer: the same
        // findings as a check of the text alone, after every edit of every fixture.
        assert_eq!(
            kept,
            self.engine.check(&self.language, &self.text),
            "a check with what was kept disagrees with one of the text alone:\n{}",
            self.text
        );
        kept.iter()
            .map(|finding| Observed::lexical(finding.range, Orthra::label(finding)))
            .collect()
    }

    fn bump_epoch(&mut self) {
        self.engine.bump_epoch();
    }

    fn set_context(&mut self, _: bool) {
        // No context tier yet: on or off, the same lexical findings, which is the invariant.
    }
}
