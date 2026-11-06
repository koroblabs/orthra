//! Flow tests: the real server, driven in process (`docs/02-testing.md` level 6).
//!
//! Every test below this layer asks a question of a data structure; these ask it of a server, in
//! an order, with state carried between the steps. Readiness, versions and what an edit does to
//! the next answer exist only here. The service is driven directly rather than over a pipe, so a
//! test is deterministic, and the project is written to a directory of its own, because the
//! server learns it from the disk.

mod flow;

use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use lspkit_flow::{Client, Flow};
use lspkit_runtime::protocol::ls_types::Uri;
use orthra_testkit::annotate::Annotation;
use orthra_text::{
    TextRange,
    line_index::{Encoding, LineCol, LineIndex},
};
use serde_json::{Value, json};

use orthra_engine::context::{Connector, Scorer};

use crate::{context::Models, install::Installer, server::Backend};

/// The context tier's models a test's server is handed: none unless the test says.
#[derive(Default)]
struct Engines {
    scorer: Option<Arc<dyn Scorer>>,
    models: Option<Arc<dyn Models>>,
    connector: Option<Arc<dyn Connector>>,
}

/// What [`Fixture::next_pass`] hands out.
struct NextPass {
    passes: tokio::sync::watch::Receiver<u64>,
    before: u64,
}

/// A server driven through lspkit's [`Flow`], and the project it was pointed at.
struct Fixture {
    flow: Flow<Backend>,
    root: PathBuf,
    /// The user's settings directory, a test's own: no test writes to the person's.
    user: PathBuf,
    encoding: Encoding,
}

impl Fixture {
    /// A server over a project of `files`, `(path, text)`, written to a directory of its own. A
    /// pack it is asked to install fails to, since a test has no network.
    fn new(files: &[(&str, &str)]) -> Fixture {
        Fixture::installing(
            files,
            Arc::new(|_: &str, _: &std::path::Path| Err("a test has no network".to_owned())),
        )
    }

    /// [`Fixture::new`], with packs installed by `installer`.
    fn installing(files: &[(&str, &str)], installer: Installer) -> Fixture {
        Fixture::with(files, installer, Engines::default())
    }

    /// [`Fixture::new`], installing and loading the context tier's models through `models`.
    fn modeling(files: &[(&str, &str)], models: Arc<dyn Models>) -> Fixture {
        Fixture::with(
            files,
            Arc::new(|_: &str, _: &std::path::Path| Err("a test has no network".to_owned())),
            Engines {
                models: Some(models),
                ..Engines::default()
            },
        )
    }

    /// [`Fixture::new`], with the context tier scoring through `scorer`.
    fn scoring(files: &[(&str, &str)], scorer: Arc<dyn Scorer>) -> Fixture {
        Fixture::with(
            files,
            Arc::new(|_: &str, _: &std::path::Path| Err("a test has no network".to_owned())),
            Engines {
                scorer: Some(scorer),
                ..Engines::default()
            },
        )
    }

    /// [`Fixture::new`], rewriting sentences through what `connector` connects to, when the
    /// settings name an endpoint.
    fn rewriting(files: &[(&str, &str)], connector: Arc<dyn Connector>) -> Fixture {
        Fixture::with(
            files,
            Arc::new(|_: &str, _: &std::path::Path| Err("a test has no network".to_owned())),
            Engines {
                connector: Some(connector),
                ..Engines::default()
            },
        )
    }

    fn with(files: &[(&str, &str)], installer: Installer, engines: Engines) -> Fixture {
        let Engines {
            scorer,
            models,
            connector,
        } = engines;
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "orthra-flow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().expect("a file is in a directory"))
                .expect("the fixture's directory");
            fs::write(&path, text).expect("the fixture's file");
        }
        let user = root.with_extension("user");
        let user_dir = user.clone();
        // A pass leaves its publications to tasks of its own, so a step gives them turns.
        let flow = Flow::new(
            move |client| {
                let packs = user_dir.join("packs");
                let mut backend =
                    Backend::with_dirs(client, Some(user_dir), Some(packs), installer);
                if let Some(scorer) = scorer {
                    backend = backend.scoring(scorer);
                }
                if let Some(models) = models {
                    backend = backend.modeling(models);
                }
                if let Some(connector) = connector {
                    backend = backend.connecting(connector);
                }
                backend
            },
            Client::Answering,
        )
        .settling_for(16);
        Fixture {
            flow,
            root,
            user,
            encoding: Encoding::Utf16,
        }
    }

    /// Where the server looks for installed packs.
    fn packs(&self) -> PathBuf {
        self.user.join("packs")
    }

    fn uri(&self, path: &str) -> Uri {
        Uri::from_file_path(self.root.join(path)).expect("an absolute path")
    }

    async fn request(&mut self, method: &'static str, params: Value) -> Value {
        let (_, result) = self.flow.request(method, params).await.into_parts();
        result.expect("the request succeeds")
    }

    async fn notify(&mut self, method: &'static str, params: Value) {
        self.flow.notify(method, params).await;
    }

    /// `initialize`, from a client with `capabilities`, rooted at the project. Returns what the
    /// server says it can do.
    async fn initialize(&mut self, capabilities: Value) -> Value {
        self.initialize_with(capabilities, Value::Null).await
    }

    /// [`Fixture::initialize`], with the editor's initialization `options`.
    async fn initialize_with(&mut self, capabilities: Value, options: Value) -> Value {
        let root = self.uri("");
        let result = self
            .request(
                "initialize",
                json!({
                    "capabilities": capabilities,
                    "workspaceFolders": [{ "uri": root, "name": "project" }],
                    "initializationOptions": options,
                }),
            )
            .await;
        if result["capabilities"]["positionEncoding"] == "utf-8" {
            self.encoding = Encoding::Utf8;
        }
        result
    }

    /// `initialized`, and the first pass over the project finished.
    async fn initialized(&mut self) {
        let mut passes = self.flow.server().passes();
        self.notify("initialized", json!({})).await;
        passes
            .wait_for(|done| *done >= 1)
            .await
            .expect("the server outlives startup");
        self.flow.settle().await;
    }

    /// A ticket for the next pass, taken before the step that starts it: the pass may be over by
    /// the time anything waits on it.
    fn next_pass(&self) -> NextPass {
        let passes = self.flow.server().passes();
        let before = *passes.borrow();
        NextPass { passes, before }
    }

    /// Waits for the pass a ticket was taken for to finish.
    async fn pass_finished(&self, mut next: NextPass) {
        next.passes
            .wait_for(|done| *done > next.before)
            .await
            .expect("the server outlives the pass");
        self.flow.settle().await;
    }

    /// The code actions for `range` in `path`, by title, each with what it does.
    async fn actions(&mut self, path: &str, range: Value) -> Vec<Value> {
        self.actions_in(path, range, json!({ "diagnostics": [] }))
            .await
    }

    /// The actions for `range`, asked for in `context`: with a `triggerKind`, say.
    async fn actions_in(&mut self, path: &str, range: Value, context: Value) -> Vec<Value> {
        let uri = self.uri(path);
        let actions = self
            .request(
                "textDocument/codeAction",
                json!({
                    "textDocument": { "uri": uri },
                    "range": range,
                    "context": context,
                }),
            )
            .await;
        actions.as_array().cloned().unwrap_or_default()
    }

    async fn open(&mut self, path: &str) {
        let text = fs::read_to_string(self.root.join(path)).expect("the fixture's file");
        let uri = self.uri(path);
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": {
                "uri": uri, "languageId": "", "version": 1, "text": text,
            }}),
        )
        .await;
    }

    /// Every publication for `path`, in order: what a client's list of problems went through.
    fn publications(&self, path: &str) -> Vec<Vec<Value>> {
        self.flow.publications(self.uri(path).as_str())
    }

    /// Lets a change's quiet pass, on a paused clock, so that what it publishes is published.
    async fn after_quiet(&self) {
        self.flow.after(Duration::from_millis(350)).await;
    }

    /// The diagnostics last published for `path`, or `None` if none were.
    fn published(&self, path: &str) -> Option<Vec<Value>> {
        self.publications(path).pop()
    }

    /// Whether the server has sent the client a request or notification called `method`.
    fn sent(&self, method: &str) -> bool {
        self.flow.methods().iter().any(|sent| sent == method)
    }

    /// `diagnostics` drawn under `text`, each labeled with its code and severity, their ranges
    /// read back in the encoding the server chose.
    fn render(&self, text: &str, diagnostics: &[Value]) -> String {
        let index = LineIndex::new(text);
        let at = |position: &Value| {
            let pos = LineCol {
                line: position["line"].as_u64().expect("a line") as u32,
                col: position["character"].as_u64().expect("a character") as u32,
            };
            index
                .offset_in(text, pos, self.encoding)
                .expect("a position in the text")
        };
        let annotations: Vec<Annotation> = diagnostics
            .iter()
            .map(|diagnostic| {
                let range = TextRange::new(
                    at(&diagnostic["range"]["start"]),
                    at(&diagnostic["range"]["end"]),
                );
                let label = format!(
                    "{} {}: {}",
                    diagnostic["code"].as_str().unwrap_or("?"),
                    diagnostic["severity"],
                    diagnostic["message"].as_str().unwrap_or("?")
                );
                Annotation::new(range, label)
            })
            .collect();
        Annotation::render(text, &annotations)
    }

    fn text(&self, path: &str) -> String {
        fs::read_to_string(self.root.join(path)).expect("the fixture's file")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.user);
    }
}

/// The capabilities Zed declares: UTF-8 offered, files watched for the server, and diagnostics it
/// could pull, which orthra pushes anyway.
fn zed_client() -> Value {
    json!({
        "general": { "positionEncodings": ["utf-8", "utf-16"] },
        "textDocument": { "diagnostic": { "dynamicRegistration": false } },
        "workspace": { "didChangeWatchedFiles": { "dynamicRegistration": true } },
    })
}

/// A client that is sent diagnostics, and speaks only the protocol's default UTF-16.
fn pushed_client() -> Value {
    json!({})
}
