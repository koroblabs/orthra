//! The context tier in the server: whether the editor's settings turn it on, and its model
//! installed and loaded in the background, after which code actions the user asks for reorder an
//! unknown word's corrections through it (`actions::Plan::rerank`).
//!
//! Turned on by the editor's initialization options, `"context": {"rank": true}` or `{"rank":
//! {"model": "small"}}`: whether to spend a machine's memory on a model is that machine's question. A model
//! not installed is installed then, as a language's pack is, unless `auto-install` is off: turning
//! the tier on is the consent to fetch it (`docs/04-context.md` rule 7). Nothing waits for any of
//! this, and a model that fails is said once and the tier stays off.
//!
//! A model left unasked for `idleUnload` seconds (ten minutes unless set) is unloaded, its threads
//! with it, and the memory given back; the next code action asked for loads it again in the
//! background, and is answered in the lexical order meanwhile.
//!
//! `"context": {"rewrite": {...}}` names an endpoint the user runs, for the "Rewrite" and
//! "Rephrase" actions (`rewrite`): read here with the rest of the tier's settings.

use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use lspkit_runtime::{
    panics,
    protocol::{Client, ls_types::MessageType},
};
use orthra_engine::context::{ApiKey, Endpoint, Prompts, Rewrite, Runtime, Scorer};
use orthra_project::Config;
use serde_json::Value;
use tokio::{
    sync::{Mutex, watch},
    time::Instant,
};

use crate::state::State;

/// The model a setting that names none asks for.
const DEFAULT_MODEL: &str = "small";

/// How long a model is kept with no code action asking for it, unless the settings say.
const DEFAULT_IDLE: Duration = Duration::from_secs(10 * 60);

/// The context tier as the settings ask for it: on, with this model, kept this long unasked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Wanted {
    pub(crate) model: String,
    pub(crate) idle_unload: Duration,
}

/// What the initialization options say about the built-in model: `{"context": {"rank": true}}`, or
/// `{"rank": {"model": "small", "idleUnload": 600}}`. `None` unless they turn it on.
pub(crate) fn wanted(options: Option<&Value>) -> Option<Wanted> {
    let rank = options?.get("context")?.get("rank")?;
    let rank = match rank {
        Value::Bool(true) => &Value::Null,
        Value::Object(_) => rank,
        _ => return None,
    };
    Some(Wanted {
        model: rank
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_MODEL)
            .to_owned(),
        idle_unload: rank
            .get("idleUnload")
            .and_then(Value::as_u64)
            .map_or(DEFAULT_IDLE, Duration::from_secs),
    })
}

/// Where Ollama serves its OpenAI-compatible API: the endpoint `rewrite` asks when it names none.
const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434/v1";

/// The endpoint the rewrite actions ask, from `{"context": {"rewrite": {"model": "qwen3:4b",
/// "endpoint": "http://127.0.0.1:11434/v1", "allowRemote": false, "apiKey": "...", "prompts":
/// {"rewrite": "...", "rephrase": "..."}}}}`. `None` without `rewrite`; why not, for one that names
/// no model. Independent of `rank`, the built-in model's: either, both or neither may be on.
pub(crate) fn rewrite_endpoint(options: Option<&Value>) -> Result<Option<Endpoint>, String> {
    let Some(rewrite) = options
        .and_then(|options| options.get("context"))
        .and_then(|context| context.get("rewrite"))
        .filter(|rewrite| rewrite.is_object())
    else {
        return Ok(None);
    };
    let text = |value: Option<&Value>| {
        value
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    };
    let model = text(rewrite.get("model")).ok_or("`context.rewrite` names no `model` to run")?;
    let prompts = rewrite.get("prompts");
    Ok(Some(Endpoint {
        url: text(rewrite.get("endpoint")).unwrap_or_else(|| DEFAULT_ENDPOINT.to_owned()),
        model,
        // `true` itself and nothing less: text leaves the machine only when the user writes it.
        allow_remote: rewrite.get("allowRemote").and_then(Value::as_bool) == Some(true),
        api_key: text(rewrite.get("apiKey")).map(ApiKey),
        prompts: Prompts {
            correct: text(prompts.and_then(|prompts| prompts.get(Rewrite::Correct.name()))),
            rephrase: text(prompts.and_then(|prompts| prompts.get(Rewrite::Rephrase.name()))),
        },
    }))
}

/// Installs and loads models: a runtime and the sources a model's definition pins, or a test's.
pub(crate) trait Models: Send + Sync {
    /// The installed file of model `name` under `models`, if it is installed.
    fn installed(&self, name: &str, models: &Path) -> Result<Option<PathBuf>, String>;
    /// Installs model `name` under `models`, and returns its file.
    fn install(&self, name: &str, models: &Path) -> Result<PathBuf, String>;
    /// The scorer the installed `file` makes.
    fn load(&self, file: &Path) -> Result<Arc<dyn Scorer>, String>;
}

/// Models built by `runtime` from the sources their definitions pin.
pub(crate) struct FromSource(pub(crate) Arc<dyn Runtime>);

impl Models for FromSource {
    fn installed(&self, name: &str, models: &Path) -> Result<Option<PathBuf>, String> {
        let model = orthra_install::models::model(name).map_err(|err| err.0)?;
        Ok(model.is_installed(models).then(|| model.file(models)))
    }

    fn install(&self, name: &str, models: &Path) -> Result<PathBuf, String> {
        let model = orthra_install::models::model(name).map_err(|err| err.0)?;
        orthra_install::models::install_model(&model, orthra_install::fetch, &*self.0, models)
            .map_err(|err| err.0)
    }

    fn load(&self, file: &Path) -> Result<Arc<dyn Scorer>, String> {
        // Read here, the request path never: once, when the tier is turned on.
        let bytes = std::fs::read(file).map_err(|err| format!("{}: {err}", file.display()))?;
        self.0.load(&bytes).map_err(|err| err.0)
    }
}

/// Where the tier stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Not yet looked at: before `initialized`.
    Starting,
    /// Not asked for, or not to be had.
    Off,
    /// Being installed or loaded.
    Loading,
    /// Loaded: asked code actions are reordered.
    On,
    /// Unloaded after being left unasked; the next asked code action loads it again.
    Unloaded,
}

/// The context tier's state, shared by the server's handlers and the tasks that load and unload.
pub(crate) struct Tier {
    scorer: RwLock<Option<Arc<dyn Scorer>>>,
    /// When a code action last used the scorer, in milliseconds since `epoch`.
    used: AtomicU64,
    epoch: Instant,
    loading: AtomicBool,
    phase: watch::Sender<Phase>,
}

impl Default for Tier {
    fn default() -> Tier {
        Tier {
            scorer: RwLock::new(None),
            used: AtomicU64::new(0),
            epoch: Instant::now(),
            loading: AtomicBool::new(false),
            phase: watch::channel(Phase::Starting).0,
        }
    }
}

impl Tier {
    /// The scorer, for a code action somebody asked for: and that it was asked for now.
    pub(crate) fn scorer(&self) -> Option<Arc<dyn Scorer>> {
        let scorer = self.scorer.read().ok()?.clone()?;
        self.used.store(self.now(), Ordering::Relaxed);
        Some(scorer)
    }

    /// Puts `scorer` in place, as a test does with no model to load.
    pub(crate) fn set(&self, scorer: Arc<dyn Scorer>) {
        if let Ok(mut held) = self.scorer.write() {
            *held = Some(scorer);
        }
        self.used.store(self.now(), Ordering::Relaxed);
        self.phase.send_replace(Phase::On);
    }

    /// Whether the model was unloaded for being left unasked, and is to be loaded again when asked:
    /// not when it never loaded, which was said once and stays so.
    pub(crate) fn unloaded(&self) -> bool {
        *self.phase.borrow() == Phase::Unloaded
    }

    #[cfg(test)]
    pub(crate) fn phase(&self) -> watch::Receiver<Phase> {
        self.phase.subscribe()
    }

    fn now(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// How long since a code action last asked.
    fn idle(&self) -> Duration {
        Duration::from_millis(self.now().saturating_sub(self.used.load(Ordering::Relaxed)))
    }
}

/// Turns the context tier on if the settings ask for it and it is not on or on its way: the model
/// installed if it is not (with a progress bar), then loaded, then watched for idleness. Started
/// on `initialized`, and again by the first code action asked after an unload.
pub(crate) fn start(
    client: &Client,
    state: &Arc<Mutex<State>>,
    models: Option<&Arc<dyn Models>>,
    tier: &Arc<Tier>,
) {
    if tier.loading.swap(true, Ordering::AcqRel) {
        return;
    }
    let (client, state, models, tier) = (
        client.clone(),
        Arc::clone(state),
        models.cloned(),
        Arc::clone(tier),
    );
    panics::spawn(
        client.clone(),
        || "turning the context tier on".to_owned(),
        async move {
            let wanted = load(&client, &state, models, &tier).await;
            tier.loading.store(false, Ordering::Release);
            if let Some(wanted) = wanted {
                watch_idle(client, tier, wanted).await;
            }
        },
    );
}

/// Installs and loads the model the settings want, into `tier`; the settings it loaded under, or
/// `None` when the tier stays off.
async fn load(
    client: &Client,
    state: &Arc<Mutex<State>>,
    models: Option<Arc<dyn Models>>,
    tier: &Tier,
) -> Option<Wanted> {
    let (wanted, dir, auto, progress) = {
        let state = state.lock().await;
        (
            state.context.clone(),
            state.models_dir(),
            state.config.as_ref().is_some_and(Config::auto_install),
            Arc::clone(&state.progress),
        )
    };
    let Some(wanted) = wanted else {
        tier.phase.send_replace(Phase::Off);
        return None;
    };
    tier.phase.send_replace(Phase::Loading);
    let say = |message: String| {
        let client = client.clone();
        async move {
            client
                .show_message(MessageType::WARNING, format!("orthra: {message}"))
                .await;
        }
    };
    let (Some(models), Some(dir)) = (models, dir) else {
        say("the context tier is on in the settings, and this orthra has no model runtime or nowhere to keep a model; it stays off".to_owned()).await;
        tier.phase.send_replace(Phase::Off);
        return None;
    };
    let name = wanted.model.clone();
    let file = {
        let (models, name, dir) = (Arc::clone(&models), name.clone(), dir.clone());
        tokio::task::spawn_blocking(move || models.installed(&name, &dir))
            .await
            .unwrap_or_else(|stopped| Err(format!("{stopped}")))
    };
    let file = match file {
        Ok(Some(file)) => Ok(file),
        Ok(None) if !auto => Err(format!(
            "model {name} is not installed, and auto-install is off: install it with `orthra model install {name}`"
        )),
        Ok(None) => {
            let bar = progress
                .start(
                    client,
                    &format!("orthra/model/{name}"),
                    &format!("Installing model {name}"),
                    1,
                    "model",
                )
                .await;
            let installed = {
                let (models, name, dir) = (Arc::clone(&models), name.clone(), dir.clone());
                tokio::task::spawn_blocking(move || models.install(&name, &dir))
                    .await
                    .unwrap_or_else(|stopped| Err(format!("the install stopped: {stopped}")))
            };
            bar.finish(if installed.is_ok() {
                "installed"
            } else {
                "not installed"
            })
            .await;
            installed.map_err(|why| {
                format!("model {name} could not be installed: {why}; install it with `orthra model install {name}`")
            })
        }
        Err(why) => Err(why),
    };
    let scorer = match file {
        Ok(file) => tokio::task::spawn_blocking(move || {
            let scorer = models.load(&file);
            // The file was read whole and its tensors copied out of it: give it back.
            lspkit_heap::release_freed();
            scorer
        })
        .await
        .unwrap_or_else(|stopped| Err(format!("{stopped}")))
        .map_err(|why| format!("model {name} does not load: {why}")),
        Err(why) => Err(why),
    };
    match scorer {
        Ok(scorer) => {
            let cost = scorer.bytes();
            tier.set(scorer);
            client
                .log_message(
                    MessageType::INFO,
                    format!(
                        "orthra: the context tier is on, with model {name}: {:.1} MB",
                        cost as f64 / 1e6
                    ),
                )
                .await;
            Some(wanted)
        }
        Err(why) => {
            say(format!("{why}. The context tier stays off.")).await;
            tier.phase.send_replace(Phase::Off);
            None
        }
    }
}

/// Unloads the model once it has gone `wanted.idle_unload` without a code action asking for it,
/// and gives its memory back: the weights, and the threads it computes on with their buffers.
async fn watch_idle(client: Client, tier: Arc<Tier>, wanted: Wanted) {
    loop {
        let idle = tier.idle();
        if idle < wanted.idle_unload {
            tokio::time::sleep(wanted.idle_unload - idle).await;
            continue;
        }
        let scorer = tier.scorer.write().ok().and_then(|mut held| held.take());
        let freed = scorer.as_ref().map_or(0, |scorer| scorer.bytes());
        drop(scorer);
        tier.phase.send_replace(Phase::Unloaded);
        let _ = tokio::task::spawn_blocking(lspkit_heap::release_freed).await;
        client
            .log_message(
                MessageType::INFO,
                format!(
                    "orthra: model {} unloaded after {} s unasked ({:.1} MB); the next code action asked loads it again",
                    wanted.model,
                    wanted.idle_unload.as_secs(),
                    freed as f64 / 1e6
                ),
            )
            .await;
        return;
    }
}
