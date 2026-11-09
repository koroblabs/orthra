//! The handlers: what the server does with each message. The state they share is in `state`, a
//! pass over the project in `passes`, what can be done about a finding in `actions`, and the
//! sentence a model rewrites when asked in `rewrite`.

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use lspkit_runtime::{
    client, panics,
    protocol::{Client, LanguageServer, jsonrpc, ls_types::*},
    quiet,
};
use orthra_engine::context::{Connector, Runtime};
use orthra_project::Config;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{
    actions,
    context::{self, FromSource, Models, Tier},
    documents::{Document, Refused},
    install::{self, Installer},
    passes::Passes,
    rewrite,
    state::State,
};

pub(crate) struct Backend {
    pub(crate) client: Client,
    state: Arc<Mutex<State>>,
    passes: Passes,
    /// The context tier: its scorer, which reorders an unknown word's corrections once the settings
    /// turn it on and its model is loaded, and whether it is loading or was unloaded.
    tier: Arc<Tier>,
    /// What installs and loads the context tier's model: none in a binary with no model runtime.
    models: Option<Arc<dyn Models>>,
    /// Whether the context tier's failure has been said: once a session, as its contract says.
    context_failed: AtomicBool,
    /// What connects to a model that rewrites a sentence: none in a binary without `ml-http`.
    connector: Option<Arc<dyn Connector>>,
    /// Whether a rewrite's failure has been shown: once a session, and logged after that.
    rewrite_failed: AtomicBool,
}

impl Backend {
    pub(crate) fn new(
        client: Client,
        models: Option<Arc<dyn Runtime>>,
        connector: Option<Arc<dyn Connector>>,
    ) -> Backend {
        let mut backend = Backend::with_dirs(
            client,
            Config::user_dir(),
            Config::packs_dir(),
            install::from_source(),
        );
        backend.models = models.map(|runtime| Arc::new(FromSource(runtime)) as Arc<dyn Models>);
        backend.connector = connector;
        backend
    }

    /// A server whose user settings are in `user` and whose installed packs are in `packs`: the
    /// person's, or a test's own directories, so that no test reads or writes the person's.
    pub(crate) fn with_dirs(
        client: Client,
        user: Option<PathBuf>,
        packs: Option<PathBuf>,
        installer: Installer,
    ) -> Backend {
        let state = Arc::new(Mutex::new(State::new(user, packs, installer)));
        Backend {
            passes: Passes::new(Arc::clone(&state), client.clone()),
            client,
            state,
            tier: Arc::default(),
            models: None,
            context_failed: AtomicBool::new(false),
            connector: None,
            rewrite_failed: AtomicBool::new(false),
        }
    }

    /// The same server, with the context tier scoring through `scorer` from the start.
    #[cfg(test)]
    pub(crate) fn scoring(self, scorer: Arc<dyn orthra_engine::context::Scorer>) -> Backend {
        self.tier.set(scorer);
        self
    }

    /// The same server, installing and loading the context tier's models through `models`.
    #[cfg(test)]
    pub(crate) fn modeling(mut self, models: Arc<dyn Models>) -> Backend {
        self.models = Some(models);
        self
    }

    /// The same server, rewriting sentences through what `connector` connects to.
    #[cfg(test)]
    pub(crate) fn connecting(mut self, connector: Arc<dyn Connector>) -> Backend {
        self.connector = Some(connector);
        self
    }

    /// Where the context tier stands, to wait on.
    #[cfg(test)]
    pub(crate) fn context_phase(&self) -> tokio::sync::watch::Receiver<context::Phase> {
        self.tier.phase()
    }

    #[cfg(test)]
    pub(crate) fn passes(&self) -> tokio::sync::watch::Receiver<u64> {
        self.passes.finished()
    }

    /// Publishes the findings for `uri`, once the project is learned.
    async fn publish(&self, uri: &Uri) {
        let found = {
            let mut state = self.state.lock().await;
            if !state.learned {
                return;
            }
            state.diagnostics(uri)
        };
        if let Some((found, version)) = found {
            self.client
                .publish_diagnostics(uri.clone(), found, Some(version))
                .await;
        }
    }

    /// Publishes once the buffer has been still for [`quiet::QUIET`], and not at all if it has not:
    /// a later change has scheduled its own wait, and the version says which one is the latest.
    /// The findings are published at the version they were computed for, which a change landing
    /// in between moves with the text.
    fn publish_when_quiet(&self, uri: Uri, version: i32) {
        let (asked, asked_about) = (Arc::clone(&self.state), uri.clone());
        let (state, client, named) = (Arc::clone(&self.state), self.client.clone(), uri.clone());
        quiet::when_quiet(
            self.client.clone(),
            move || format!("checking {}", named.as_str()),
            version,
            move || async move {
                let state = asked.lock().await;
                let document = state.documents.get(&asked_about)?;
                state.learned.then_some(document.version)
            },
            move || async move {
                let found = state.lock().await.diagnostics(&uri);
                if let Some((found, version)) = found {
                    client.publish_diagnostics(uri, found, Some(version)).await;
                }
            },
        );
    }

    /// A pass over the project on a task of its own: `reload` reads the settings again first. A
    /// panic in it is reported, not lost with the task, which would leave every finding unpublished
    /// for the rest of the session and look exactly like a clean project.
    fn spawn_pass(&self, reload: bool) {
        let passes = self.passes.clone();
        panics::spawn(
            self.client.clone(),
            || "checking the project".to_owned(),
            async move {
                if reload {
                    passes.reload().await;
                } else {
                    passes.run().await;
                }
            },
        );
    }

    /// Asks the client to say when a settings or words file changes, since each changes the
    /// findings. Asked on a task of its own and given up on after a few seconds: messages are
    /// handled in order, and a client that never answers would otherwise hold every one after it.
    /// A client that cannot be asked is not: the files are read again on restart.
    fn watch_settings(&self) {
        let (state, client) = (Arc::clone(&self.state), self.client.clone());
        panics::spawn(
            self.client.clone(),
            || "watching orthra's settings files".to_owned(),
            async move {
                let files = {
                    let state = state.lock().await;
                    match &state.config {
                        Some(config) if state.watch => config.watched(state.user.as_deref()),
                        _ => return,
                    }
                };
                let watchers = files
                    .iter()
                    .map(|file| FileSystemWatcher {
                        glob_pattern: GlobPattern::String(file.to_string_lossy().into_owned()),
                        kind: None,
                    })
                    .collect();
                client::watch_files(
                    &client,
                    "orthra-settings",
                    "watch orthra's settings files",
                    watchers,
                )
                .await;
            },
        );
    }
}

/// What the client asked for in its initialization options: `{"languages": ["en-GB"],
/// "severity": "information"}`.
fn options(options: Option<&Value>) -> (Vec<String>, Option<String>) {
    let languages = options
        .and_then(|options| options.get("languages"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let severity = options
        .and_then(|options| options.get("severity"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    (languages, severity)
}

/// The path a document is read by: its file's, or the URI itself for a buffer that has no file.
fn path_of(uri: &Uri) -> String {
    match uri.to_file_path() {
        Some(path) => path.to_string_lossy().into_owned(),
        None => uri.as_str().to_owned(),
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        let (languages, severity) = options(params.initialization_options.as_ref());
        let wanted = context::wanted(params.initialization_options.as_ref());
        // Connected now, so that an endpoint that cannot be had is said once, at the start. Nothing
        // is sent until somebody chooses the action.
        let rewriter = match rewrite::connect(
            context::rewrite_endpoint(params.initialization_options.as_ref()),
            self.connector.as_ref(),
        ) {
            Ok(rewriter) => rewriter,
            Err(why) => {
                self.client
                    .show_message(
                        MessageType::WARNING,
                        format!("orthra: {why}, so there is no rewrite action"),
                    )
                    .await;
                None
            }
        };
        if let Some((_, endpoint)) = &rewriter {
            self.client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "orthra: the rewrite actions ask {} at {}",
                        endpoint.model, endpoint.url
                    ),
                )
                .await;
        }
        let rewrites = rewriter.is_some();
        let (encoding, kind) = lspkit_text::lsp::negotiate(&params.capabilities);
        let watch = client::can_watch_files(&params.capabilities);
        // The project's settings and the user's, as `orthra check` reads them, then what the
        // editor asks for.
        let config = {
            let mut state = self.state.lock().await;
            state.roots = client::roots(&params);
            state.editor_languages = languages;
            state.editor_severity = severity;
            state.context = wanted;
            state.rewriting = rewriter.map(|(rewriting, _)| rewriting);
            state.encoding = encoding;
            state.progress.negotiate(&params.capabilities);
            state.watch = watch;
            state.read_config()
        };
        for problem in config.problems() {
            self.client
                .show_message(MessageType::WARNING, format!("orthra: {problem}"))
                .await;
        }
        let engine = config.engine().map_err(jsonrpc::Error::invalid_params)?;
        {
            let mut state = self.state.lock().await;
            state.engine = Some(engine);
            state.config = Some(config);
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(kind),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..TextDocumentSyncOptions::default()
                    },
                )),
                // A rewrite's edit is made once it is chosen; every other action carries its own.
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(if rewrites {
                            vec![CodeActionKind::QUICKFIX, actions::FIX_ALL, rewrite::KIND]
                        } else {
                            vec![CodeActionKind::QUICKFIX, actions::FIX_ALL]
                        }),
                        resolve_provider: Some(rewrites),
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![actions::ADD_WORD.to_owned()],
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: crate::NAME.to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.spawn_pass(false);
        self.watch_settings();
        self.passes.install_missing().await;
        context::start(&self.client, &self.state, self.models.as_ref(), &self.tier);
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        {
            let mut state = self.state.lock().await;
            state.documents.open(
                document.uri.clone(),
                Document {
                    path: path_of(&document.uri),
                    language: document.language_id,
                    text: document.text,
                    version: document.version,
                    diverged: false,
                },
            );
            // Its findings as a closed file are replaced by the ones published now, which are
            // the same until it is edited: never cleared in between, so a list of the project's
            // problems does not flicker when an editor opens the file to show them.
            state.closed.forget(&document.uri);
        }
        self.publish(&document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let (uri, version) = (params.text_document.uri, params.text_document.version);
        let refused = {
            let mut state = self.state.lock().await;
            let encoding = state.encoding;
            state
                .documents
                .change(&uri, version, params.content_changes, encoding)
        };
        match refused {
            Ok(()) => self.publish_when_quiet(uri, version),
            // Said once, when it diverged.
            Err(Refused::NotOpen | Refused::Diverged) => {}
            Err(why) => {
                // Said, not swallowed: every answer after it would be about the wrong text.
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("orthra ignored a change to {}: {why:?}", uri.as_str()),
                    )
                    .await;
                // And what was published is about that text too: cleared, if it diverged.
                if why == Refused::OutOfRange {
                    self.publish(&uri).await;
                }
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // What the saved file declares becomes the project's, as it would on the next start.
        let mut state = self.state.lock().await;
        let State {
            engine, documents, ..
        } = &mut *state;
        let (Some(engine), Some(document)) =
            (engine.as_mut(), documents.get(&params.text_document.uri))
        else {
            return;
        };
        // What it declares is read from the text it has, which a diverged one does not.
        if document.diverged {
            return;
        }
        if let Some(language) = engine.language_for_path(&document.path) {
            engine.learn(language, &document.text);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        // A closed file's problems are still the project's: they are published again, from the
        // text it had when it was closed, as the problems of a closed file.
        let found = {
            let mut state = self.state.lock().await;
            let found = if state.learned {
                state.diagnostics(&uri).map(|(found, _)| found)
            } else {
                None
            };
            state.documents.close(&uri);
            let found = found.unwrap_or_default();
            state.closed.note(&uri, !found.is_empty());
            found
        };
        self.client.publish_diagnostics(uri, found, None).await;
        // Its tree and its words are freed: given back, off the server's thread, since it walks
        // the allocator's arenas and costs milliseconds (`passes.rs`).
        tokio::task::spawn_blocking(lspkit_heap::release_freed);
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        // A settings or a words file changed: the findings are whatever the new settings say.
        self.spawn_pass(true);
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> jsonrpc::Result<Option<CodeActionResponse>> {
        // Only for actions somebody asked for: a client also asks on its own as the cursor moves,
        // to know whether to show that there are any, and a model need not run for that, nor a
        // sentence be offered to one.
        let asked = params.context.trigger_kind != Some(CodeActionTriggerKind::AUTOMATIC);
        let uri = &params.text_document.uri;
        let (mut plan, rewrite) = {
            let mut state = self.state.lock().await;
            if !state.learned {
                return Ok(None);
            }
            let Some(plan) = actions::plan(&mut state, uri, params.range) else {
                return Ok(None);
            };
            let rewrite = if asked && rewrite::wanted(params.context.only.as_deref()) {
                rewrite::offer(&mut state, uri, params.range)
            } else {
                Vec::new()
            };
            (plan, rewrite)
        };
        let scorer = if asked { self.tier.scorer() } else { None };
        // Asked for, with the model unloaded for being left unasked: loaded again, in the
        // background, and this answer in the lexical order. A model that never loaded stays off.
        if asked && scorer.is_none() && self.tier.unloaded() {
            context::start(&self.client, &self.state, self.models.as_ref(), &self.tier);
        }
        if let Some(scorer) = scorer.as_ref()
            && let Some(failed) = plan.rerank(scorer, actions::RERANK_BUDGET).await
            && !self.context_failed.swap(true, Ordering::Relaxed)
        {
            self.client
                .log_message(MessageType::WARNING, format!("orthra: {failed}"))
                .await;
        }
        let only = params.context.only.as_deref();
        let fix_all = plan.offers_fix_all(only, asked);
        let mut actions =
            plan.actions(actions::asked_for(only, &CodeActionKind::QUICKFIX), fix_all);
        actions.extend(rewrite);
        Ok((!actions.is_empty()).then_some(actions))
    }

    async fn code_action_resolve(&self, mut action: CodeAction) -> jsonrpc::Result<CodeAction> {
        let Some(asked) = rewrite::Asked::of(&action) else {
            return Ok(action);
        };
        let job = {
            let state = self.state.lock().await;
            asked.job(&state)
        };
        // The document changed since the action was offered: no edit, for that range is not the
        // sentence any more.
        let Some(job) = job else {
            return Ok(action);
        };
        match job.run().await {
            // Unless it changed while the model ran.
            Ok(Some(rewrite)) => {
                let current = {
                    let state = self.state.lock().await;
                    asked.is_current(&state)
                };
                if current {
                    action.edit = Some(asked.edit(rewrite));
                }
            }
            Ok(None) => {
                self.client
                    .show_message(
                        MessageType::INFO,
                        "orthra: the model left the sentence as it was",
                    )
                    .await;
            }
            Err(why) => {
                let message = format!("orthra: the sentence was not rewritten: {why}");
                if self.rewrite_failed.swap(true, Ordering::Relaxed) {
                    self.client.log_message(MessageType::WARNING, message).await;
                } else {
                    self.client
                        .show_message(MessageType::WARNING, message)
                        .await;
                }
            }
        }
        Ok(action)
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> jsonrpc::Result<Option<LSPAny>> {
        if params.command != actions::ADD_WORD {
            return Err(jsonrpc::Error::invalid_params(format!(
                "orthra has no command {}",
                params.command
            )));
        }
        let added = {
            let state = self.state.lock().await;
            actions::add_to_words(&state, &params.arguments)
        };
        added.map_err(jsonrpc::Error::invalid_params)?;
        // Read back as any edit to the file is, and the project checked again with it.
        self.spawn_pass(true);
        Ok(None)
    }
}
