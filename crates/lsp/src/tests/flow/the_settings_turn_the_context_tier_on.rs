//! The editor's settings turn the context tier on: the model is installed when it is not, as a
//! language's pack is, then loaded, and from then on a code action somebody asks for reorders an
//! unknown word's corrections. Off in the settings, or with auto-install off and no model, nothing
//! is installed or loaded, and the second is said.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use expect_test::expect;
use orthra_engine::context::{FakeScorer, Scorer};
use serde_json::{Value, json};

use crate::{
    context::{Models, Phase},
    tests::{Fixture, zed_client},
};

const NOTES: (&str, &str) = ("notes.md", "Fixx the leter, and send it.\n");

/// Models of the test's own: what was asked of them, and a model file that is only a name.
#[derive(Default)]
struct Pretend {
    asked: Mutex<Vec<String>>,
}

impl Models for Pretend {
    fn installed(&self, name: &str, models: &Path) -> Result<Option<PathBuf>, String> {
        self.asked
            .lock()
            .unwrap()
            .push(format!("installed? {name}"));
        let file = models.join(name);
        Ok(file.is_file().then_some(file))
    }

    fn install(&self, name: &str, models: &Path) -> Result<PathBuf, String> {
        self.asked.lock().unwrap().push(format!("install {name}"));
        std::fs::create_dir_all(models).map_err(|err| err.to_string())?;
        std::fs::write(models.join(name), "a model").map_err(|err| err.to_string())?;
        Ok(models.join(name))
    }

    fn load(&self, file: &Path) -> Result<Arc<dyn Scorer>, String> {
        let name = file.file_name().unwrap().to_string_lossy();
        self.asked.lock().unwrap().push(format!("load {name}"));
        Ok(Arc::new(FakeScorer))
    }
}

/// The corrections offered for `leter` when somebody asks for the actions.
async fn offered(server: &mut Fixture) -> String {
    server.open("notes.md").await;
    let range =
        json!({ "start": { "line": 0, "character": 10 }, "end": { "line": 0, "character": 10 } });
    let actions = server
        .actions_in(
            "notes.md",
            range,
            json!({ "diagnostics": [], "triggerKind": 1 }),
        )
        .await;
    actions
        .iter()
        .filter_map(|action| action["title"].as_str())
        .filter(|title| title.starts_with("Change to"))
        .map(|title| title.trim_start_matches("Change to "))
        .collect::<Vec<_>>()
        .join(" ")
}

async fn started(files: &[(&str, &str)], options: Value) -> (Fixture, Arc<Pretend>) {
    let models = Arc::new(Pretend::default());
    let mut server = Fixture::modeling(files, Arc::clone(&models) as Arc<dyn Models>);
    let mut phase = server.flow.server().context_phase();
    server.initialize_with(zed_client(), options).await;
    server.initialized().await;
    phase
        .wait_for(|phase| matches!(phase, Phase::On | Phase::Off))
        .await
        .expect("the context tier is turned on or found off");
    server.flow.settle().await;
    (server, models)
}

fn said(server: &Fixture) -> String {
    ["window/showMessage", "window/logMessage"]
        .iter()
        .flat_map(|method| server.flow.params(method))
        .filter_map(|params| params["message"].as_str().map(str::to_owned))
        .filter(|message| message.contains("model") || message.contains("context"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
async fn turned_on_the_model_is_installed_loaded_and_reorders() {
    let (mut server, models) = started(&[NOTES], json!({ "context": { "rank": true } })).await;
    let order = offered(&mut server).await;
    expect![[r#"
        installed? small, install small, load small
        orthra: the context tier is on, with model small: 0.0 MB
        `later` `liter` `lefter` `letter` `leer`"#]]
    .assert_eq(&format!(
        "{}\n{}\n{order}",
        models.asked.lock().unwrap().join(", "),
        said(&server)
    ));
}

#[tokio::test]
async fn off_or_with_auto_install_off_nothing_is_installed_and_the_order_is_lexical() {
    let (mut off, off_models) = started(&[NOTES], json!({ "context": { "rank": false } })).await;
    let (mut manual, manual_models) = started(
        &[NOTES, ("orthra.toml", "auto-install = false\n")],
        json!({ "context": { "rank": { "model": "small" } } }),
    )
    .await;
    let (off_order, manual_order) = (offered(&mut off).await, offered(&mut manual).await);
    let asked = |models: &Pretend| models.asked.lock().unwrap().join(", ");
    expect![[r#"
        off: [] `leer` `letter` `lefter` `liter` `later`
        auto-install off: [installed? small] `leer` `letter` `lefter` `liter` `later`
        orthra: model small is not installed, and auto-install is off: install it with `orthra model install small`. The context tier stays off."#]].assert_eq(&format!(
        "off: [{}] {}\nauto-install off: [{}] {}\n{}",
        asked(&off_models),
        off_order,
        asked(&manual_models),
        manual_order,
        said(&manual)
    ));
}

/// The corrections offered for `leter`, asked for, on the paused clock: the open document is
/// already published.
async fn asked(server: &mut Fixture) -> String {
    let range =
        json!({ "start": { "line": 0, "character": 10 }, "end": { "line": 0, "character": 10 } });
    let actions = server
        .actions_in(
            "notes.md",
            range,
            json!({ "diagnostics": [], "triggerKind": 1 }),
        )
        .await;
    actions
        .iter()
        .filter_map(|action| action["title"].as_str())
        .filter(|title| title.starts_with("Change to"))
        .map(|title| title.trim_start_matches("Change to "))
        .collect::<Vec<_>>()
        .join(" ")
}

#[tokio::test(start_paused = true)]
async fn left_unasked_the_model_is_unloaded_and_the_next_asked_action_loads_it_again() {
    let (mut server, models) = started(
        &[NOTES],
        json!({ "context": { "rank": { "idleUnload": 60 } } }),
    )
    .await;
    server.open("notes.md").await;
    let mut phase = server.flow.server().context_phase();
    let mut log = vec![format!("on: {}", asked(&mut server).await)];

    // A minute unasked.
    tokio::time::advance(std::time::Duration::from_secs(61)).await;
    phase
        .wait_for(|phase| *phase == Phase::Unloaded)
        .await
        .expect("unloaded");
    log.push(format!("unloaded, asked: {}", asked(&mut server).await));
    phase
        .wait_for(|phase| *phase == Phase::On)
        .await
        .expect("loaded again");
    log.push(format!("loaded again, asked: {}", asked(&mut server).await));
    server.flow.settle().await;
    log.push(models.asked.lock().unwrap().join(", "));
    log.push(said(&server));
    expect![[r#"
        on: `later` `liter` `lefter` `letter` `leer`
        unloaded, asked: `leer` `letter` `lefter` `liter` `later`
        loaded again, asked: `later` `liter` `lefter` `letter` `leer`
        installed? small, install small, load small, installed? small, load small
        orthra: the context tier is on, with model small: 0.0 MB
        orthra: model small unloaded after 60 s unasked (0.0 MB); the next code action asked loads it again
        orthra: the context tier is on, with model small: 0.0 MB"#]].assert_eq(&log.join("\n"));
}
