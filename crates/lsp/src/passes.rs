//! A pass over the project: learned, what is open answered, then every file checked and the closed
//! files' findings published. One at startup, and one whenever a setting or a words file changes,
//! since either changes the findings (`CLAUDE.md` §1.2).

use std::{
    collections::HashSet,
    future::Future,
    pin::Pin,
    sync::{Arc, PoisonError},
};

use lspkit_runtime::{
    panics,
    protocol::{
        Client,
        ls_types::{Diagnostic, MessageType, Uri},
    },
};
use orthra_project::{Config, Project};
use orthra_text::line_index::LineIndex;
use tokio::sync::{Mutex, watch};

use crate::{convert, state::State};

#[derive(Clone)]
pub(crate) struct Passes {
    state: Arc<Mutex<State>>,
    client: Client,
    /// How many passes have finished.
    done: watch::Sender<u64>,
    /// Held for a whole pass, so that a burst of changes runs its passes one after another and
    /// the last one's findings are the ones left standing.
    running: Arc<Mutex<()>>,
}

impl Passes {
    pub(crate) fn new(state: Arc<Mutex<State>>, client: Client) -> Passes {
        Passes {
            state,
            client,
            done: watch::Sender::new(0),
            running: Arc::default(),
        }
    }

    /// A receiver that sees each pass finish. Taken before the pass starts, since it may be over
    /// by the time anything waits on it.
    #[cfg(test)]
    pub(crate) fn finished(&self) -> watch::Receiver<u64> {
        self.done.subscribe()
    }

    /// The settings read again, the engine built from them, and a pass over the project.
    ///
    /// Boxed, because a pack installed by [`Passes::install_missing`] reloads in turn: the future
    /// names itself, and only a box gives it a size and lets it be sent to a task.
    pub(crate) fn reload(self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            let _running = self.running.lock().await;
            let config = {
                let mut state = self.state.lock().await;
                let config = state.read_config();
                let engine = match config.engine() {
                    Ok(engine) => engine,
                    Err(err) => {
                        drop(state);
                        self.client
                            .show_message(MessageType::ERROR, format!("orthra: {err}"))
                            .await;
                        return;
                    }
                };
                state.engine = Some(engine);
                state.config = Some(config.clone());
                state.learned = false;
                config
            };
            for problem in config.problems() {
                self.client
                    .show_message(MessageType::WARNING, format!("orthra: {problem}"))
                    .await;
            }
            self.install_missing().await;
            self.pass().await;
        })
    }

    /// Installs the packs the settings enable and the engine does not have, each on a task of its
    /// own, and checks the project again when one is in: see `install`. With `auto-install` off, or
    /// nowhere to install to, it says how instead. A pack being installed, or one that failed this
    /// session, is left alone.
    pub(crate) async fn install_missing(&self) {
        let (tags, packs, auto, installer, progress) = {
            let mut state = self.state.lock().await;
            let Some(engine) = &state.engine else {
                return;
            };
            let tags: Vec<String> = engine
                .missing_packs()
                .iter()
                .filter(|tag| {
                    !state.installing.contains(*tag) && !state.install_failed.contains(*tag)
                })
                .cloned()
                .collect();
            let auto = state.config.as_ref().is_some_and(Config::auto_install);
            let packs = state.packs.clone().filter(|_| auto);
            if packs.is_some() {
                state.installing.extend(tags.iter().cloned());
            }
            (
                tags,
                packs,
                auto,
                Arc::clone(&state.installer),
                Arc::clone(&state.progress),
            )
        };
        let Some(packs) = packs else {
            for tag in &tags {
                let why = if auto {
                    "there is nowhere to install it"
                } else {
                    "auto-install is off"
                };
                self.client
                    .show_message(MessageType::WARNING, crate::not_installed(tag, why))
                    .await;
            }
            return;
        };
        for tag in tags {
            let (passes, installer, progress, packs) = (
                self.clone(),
                Arc::clone(&installer),
                Arc::clone(&progress),
                packs.clone(),
            );
            panics::spawn(
                self.client.clone(),
                {
                    let tag = tag.clone();
                    move || format!("installing {tag}")
                },
                async move {
                    let bar = progress
                        .start(
                            &passes.client,
                            &format!("orthra/install/{tag}"),
                            &format!("Installing {tag}"),
                            1,
                            "pack",
                        )
                        .await;
                    let installed = {
                        let tag = tag.clone();
                        tokio::task::spawn_blocking(move || installer(&tag, &packs))
                            .await
                            .unwrap_or_else(|stopped| {
                                Err(format!("the install stopped: {stopped}"))
                            })
                    };
                    {
                        let mut state = passes.state.lock().await;
                        state.installing.remove(&tag);
                        if installed.is_err() {
                            state.install_failed.insert(tag.clone());
                        }
                    }
                    match installed {
                        Ok(what) => {
                            bar.finish(format!("{what} installed")).await;
                            passes
                                .client
                                .show_message(
                                    MessageType::INFO,
                                    format!("orthra: installed {what}; checking the project again"),
                                )
                                .await;
                            passes.reload().await;
                        }
                        Err(why) => {
                            bar.finish("not installed").await;
                            passes
                                .client
                                .show_message(
                                    MessageType::WARNING,
                                    format!(
                                        "orthra: {tag} could not be installed: {why}. Nothing written in it is checked; install it with `orthra pack install {tag}`."
                                    ),
                                )
                                .await;
                        }
                    }
                },
            );
        }
    }

    /// A pass over the project with the engine and settings in place.
    pub(crate) async fn run(self) {
        let _running = self.running.lock().await;
        self.pass().await;
    }

    async fn pass(&self) {
        let (sibling, config, roots, encoding, severity) = {
            let state = self.state.lock().await;
            (
                state.engine.as_ref().map(|engine| engine.sibling()),
                state.config.clone(),
                state.roots.clone(),
                state.encoding,
                state.severity(),
            )
        };
        let (Some(mut engine), Some(config)) = (sibling, config) else {
            return;
        };
        let learned = tokio::task::spawn_blocking(move || {
            let project = Project::discover(&roots, &engine, &config);
            project.learn(&mut engine);
            (engine, project)
        })
        .await;
        let (engine, project) = match learned {
            Ok(learned) => learned,
            Err(err) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("orthra could not learn the project: {err}"),
                    )
                    .await;
                return;
            }
        };

        // What is open is answered first, and the rest of the project after.
        let (published, fork) = {
            let mut state = self.state.lock().await;
            let fork = engine.fork();
            state.engine = Some(engine);
            state.learned = true;
            let uris: Vec<Uri> = state.documents.iter().map(|(uri, _)| uri.clone()).collect();
            let published: Vec<_> = uris
                .into_iter()
                .filter_map(|uri| Some((state.diagnostics(&uri)?, uri)))
                .collect();
            (published, fork)
        };
        for ((found, version), uri) in published {
            self.client
                .publish_diagnostics(uri, found, Some(version))
                .await;
        }

        // Every file checked, on a fork of the engine so that requests are answered meanwhile,
        // and the closed files' findings published: an editor lists every problem in the project,
        // not only the open files'. Only the diagnostics are kept.
        let swept = tokio::task::spawn_blocking(move || {
            let closed = std::sync::Mutex::new(Vec::new());
            project.check(&fork, |at, file, found| {
                let Some((text, findings)) = found else {
                    return;
                };
                let index = LineIndex::new(text);
                let diagnostics: Vec<Diagnostic> = findings
                    .iter()
                    .filter_map(|finding| {
                        convert::diagnostic(&index, text, finding, encoding, severity)
                    })
                    .collect();
                if !diagnostics.is_empty()
                    && let Some(uri) = Uri::from_file_path(file)
                {
                    closed.lock().unwrap_or_else(PoisonError::into_inner).push((
                        at,
                        uri,
                        diagnostics,
                    ));
                }
            });
            // Published in the project's order, whichever thread checked a file first.
            let mut closed = closed.into_inner().unwrap_or_else(PoisonError::into_inner);
            closed.sort_unstable_by_key(|(at, ..)| *at);
            closed
                .into_iter()
                .map(|(_, uri, diagnostics)| (uri, diagnostics))
                .collect::<Vec<_>>()
        })
        .await;
        // A pass that stopped part-way is no answer about the files it did not reach: none is
        // cleared, and the panic goes on to whoever spawned the pass, which reports it.
        let swept = swept.unwrap_or_else(|stopped| std::panic::resume_unwind(stopped.into_panic()));
        let (closed, cleared) = {
            let mut state = self.state.lock().await;
            let closed: Vec<_> = swept
                .into_iter()
                .filter(|(uri, _)| state.documents.get(uri).is_none())
                .collect();
            let now: HashSet<Uri> = closed.iter().map(|(uri, _)| uri.clone()).collect();
            // Published by an earlier pass, and clean now: a word added, a file ignored. An open
            // file is never among them: opening it forgot it.
            let cleared = state.closed.unseen(&now);
            for (uri, _) in &closed {
                state.closed.note(uri, true);
            }
            (closed, cleared)
        };
        for uri in cleared {
            self.client.publish_diagnostics(uri, Vec::new(), None).await;
        }
        for (uri, found) in closed {
            self.client.publish_diagnostics(uri, found, None).await;
        }
        // The pass parsed every file and dropped each tree, and the engine before this one is
        // gone: glibc keeps what tree-sitter freed until it is asked to give it back. On ripgrep,
        // helix and phrost that was 8 to 12 MB of the 26 to 31 MB a process held (2026-11-09).
        let _ = tokio::task::spawn_blocking(lspkit_heap::release_freed).await;
        self.done.send_modify(|done| *done += 1);
    }
}
