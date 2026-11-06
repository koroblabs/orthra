//! What the server holds between messages.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use lspkit_runtime::{
    progress::Progress,
    protocol::ls_types::{Diagnostic, DiagnosticSeverity, Uri},
    reported::Reported,
};
use orthra_engine::{Engine, Finding};
use orthra_project::Config;
use orthra_text::line_index::{Encoding, LineIndex};

use crate::{convert, documents::Documents, install::Installer};

pub(crate) struct State {
    /// Built from the project's settings and the languages the client asked for, at `initialize`
    /// and again whenever a setting changes.
    pub(crate) engine: Option<Engine>,
    pub(crate) config: Option<Config>,
    pub(crate) documents: Documents,
    pub(crate) encoding: Encoding,
    pub(crate) roots: Vec<PathBuf>,
    /// The user's settings directory, `~/.config/orthra`: where "add to your words" writes.
    pub(crate) user: Option<PathBuf>,
    /// Where installed packs are, `~/.local/share/orthra/packs`.
    pub(crate) packs: Option<PathBuf>,
    /// The context tier, if the editor's settings turn it on.
    pub(crate) context: Option<crate::context::Wanted>,
    /// What rewrites a sentence when somebody asks, if the editor's settings name an endpoint.
    pub(crate) rewriting: Option<crate::rewrite::Rewriting>,
    /// How a missing pack is installed: fetched from its source, or a test's own.
    pub(crate) installer: Installer,
    /// The packs being installed now, so a second pass does not start a second install.
    pub(crate) installing: HashSet<String>,
    /// The packs whose install failed this session: said once, and not tried again until restart.
    pub(crate) install_failed: HashSet<String>,
    /// Whether the client draws progress bars, from `initialize`.
    pub(crate) progress: Arc<Progress>,
    /// The languages the editor asked for, kept for when the settings are read again.
    pub(crate) editor_languages: Vec<String>,
    /// The severity the editor asked for, which wins over every settings file.
    pub(crate) editor_severity: Option<String>,
    /// Whether the client can be asked to watch the settings files.
    pub(crate) watch: bool,
    /// Whether the project has been learned. Until it is, nothing is reported: half a project's
    /// vocabulary reports the other half's names as unknown words, and a reader cannot tell that
    /// from a real finding.
    pub(crate) learned: bool,
    /// The closed files whose findings were published, so that a pass that finds them clean
    /// clears them.
    pub(crate) closed: Reported<Uri>,
}

impl State {
    pub(crate) fn new(
        user: Option<PathBuf>,
        packs: Option<PathBuf>,
        installer: Installer,
    ) -> State {
        State {
            engine: None,
            config: None,
            documents: Documents::default(),
            // The protocol's default, until `initialize` says what the client speaks.
            encoding: Encoding::Utf16,
            roots: Vec::new(),
            user,
            packs,
            context: None,
            rewriting: None,
            installer,
            installing: HashSet::new(),
            install_failed: HashSet::new(),
            progress: Arc::new(Progress::new()),
            editor_languages: Vec::new(),
            editor_severity: None,
            watch: false,
            learned: false,
            closed: Reported::default(),
        }
    }

    /// Where installed models are: beside installed packs, `<data>/orthra/models`.
    pub(crate) fn models_dir(&self) -> Option<PathBuf> {
        Some(self.packs.as_deref()?.parent()?.join("models"))
    }

    /// The settings for the project at the first root, with the editor's languages added.
    pub(crate) fn read_config(&self) -> Config {
        let mut config = match self.roots.first() {
            Some(root) => Config::load(root, self.user.as_deref()),
            None => Config::builtin(Path::new(".")),
        };
        config.set_packs(self.packs.clone());
        config.add_languages(self.editor_languages.iter().cloned());
        if let Some(severity) = &self.editor_severity {
            config.set_severity(severity);
        }
        config
    }

    /// The findings for the open document at `uri`, with its text: none for a file the project
    /// ignores, which is open and silent.
    pub(crate) fn findings(&mut self, uri: &Uri) -> Option<(Vec<Finding>, &str, i32)> {
        let State {
            engine,
            config,
            documents,
            ..
        } = self;
        let engine = engine.as_mut()?;
        let (document, parsed) = documents.parsed_for(uri)?;
        let ignored = config
            .as_ref()
            .is_some_and(|config| config.is_ignored(Path::new(&document.path)));
        let findings = if ignored {
            Vec::new()
        } else {
            engine.check_document(&document.path, &document.language, &document.text, parsed)
        };
        Some((findings, &document.text, document.version))
    }

    /// The severity findings are shown with, from the settings.
    pub(crate) fn severity(&self) -> DiagnosticSeverity {
        convert::severity(self.config.as_ref().map_or("warning", Config::severity))
    }

    /// The findings for `uri`, as the client shows them.
    pub(crate) fn diagnostics(&mut self, uri: &Uri) -> Option<(Vec<Diagnostic>, i32)> {
        let encoding = self.encoding;
        let severity = self.severity();
        let (findings, text, version) = self.findings(uri)?;
        let index = LineIndex::new(text);
        let found = findings
            .iter()
            .filter_map(|finding| convert::diagnostic(&index, text, finding, encoding, severity))
            .collect();
        Some((found, version))
    }
}
