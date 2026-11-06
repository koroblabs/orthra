//! The context tier reorders an unknown word's corrections when somebody asks for the actions, and
//! does nothing else: the same corrections, the same findings, and the lexical order whenever the
//! scorer is not asked, is slower than its budget, or fails, which is said once.

use std::sync::{Arc, Mutex};

use expect_test::expect;
use orthra_engine::context::{FakeScorer, ScoreError, Scorer};
use orthra_text::TextRange;
use serde_json::{Value, json};

use crate::tests::{Fixture, zed_client};

const FILES: &[(&str, &str)] = &[("notes.md", "Fixx the leter, and send it.\n")];

/// The corrections offered for `leter`, in their order.
async fn offered(server: &mut Fixture, context: Value) -> String {
    let range =
        json!({ "start": { "line": 0, "character": 10 }, "end": { "line": 0, "character": 10 } });
    let actions = server.actions_in("notes.md", range, context).await;
    actions
        .iter()
        .filter_map(|action| action["title"].as_str())
        .filter(|title| title.starts_with("Change to"))
        .map(|title| title.trim_start_matches("Change to "))
        .collect::<Vec<_>>()
        .join(" ")
}

async fn findings(server: &mut Fixture) -> Vec<Value> {
    server.open("notes.md").await;
    server.published("notes.md").expect("published on open")
}

#[tokio::test(start_paused = true)]
async fn asked_for_the_actions_the_fake_scorer_turns_the_corrections_around() {
    let mut lexical = Fixture::new(FILES);
    lexical.initialize(zed_client()).await;
    lexical.initialized().await;
    let lexical_findings = findings(&mut lexical).await;

    let mut server = Fixture::scoring(FILES, Arc::new(FakeScorer));
    server.initialize(zed_client()).await;
    server.initialized().await;
    assert_eq!(
        findings(&mut server).await,
        lexical_findings,
        "the findings are the lexical tier's"
    );

    let got = [
        (
            "lexical",
            offered(&mut lexical, json!({ "diagnostics": [] })).await,
        ),
        (
            "asked",
            offered(&mut server, json!({ "diagnostics": [], "triggerKind": 1 })).await,
        ),
        (
            "asked by an older client",
            offered(&mut server, json!({ "diagnostics": [] })).await,
        ),
        (
            "the cursor moved",
            offered(&mut server, json!({ "diagnostics": [], "triggerKind": 2 })).await,
        ),
    ]
    .map(|(when, order)| format!("{when}: {order}"));
    expect![[r#"
        lexical: `leer` `letter` `lefter` `liter` `later`
        asked: `later` `liter` `lefter` `letter` `leer`
        asked by an older client: `later` `liter` `lefter` `letter` `leer`
        the cursor moved: `leer` `letter` `lefter` `liter` `later`"#]]
    .assert_eq(&got.join("\n"));
}

/// A scorer that never answers in time: it waits until the test lets it go.
#[derive(Debug)]
struct Stalled(Mutex<std::sync::mpsc::Receiver<()>>);

impl Scorer for Stalled {
    fn score(&self, _: &str, _: TextRange, candidates: &[&str]) -> Result<Vec<f32>, ScoreError> {
        let _ = self.0.lock().expect("one score at a time").recv();
        Ok((0..candidates.len()).map(|at| at as f32).collect())
    }
}

/// A scorer that cannot score.
#[derive(Debug)]
struct Broken;

impl Scorer for Broken {
    fn score(&self, _: &str, _: TextRange, _: &[&str]) -> Result<Vec<f32>, ScoreError> {
        Err(ScoreError("the model is not loaded".to_owned()))
    }
}

fn logged(server: &Fixture) -> String {
    server
        .flow
        .params("window/logMessage")
        .iter()
        .filter_map(|params| params["message"].as_str())
        .filter(|message| message.contains("context tier"))
        .collect::<Vec<_>>()
        .join("\n")
}

// On the real clock: a paused one does not move while a blocking task runs, and the stalled
// scorer's budget would never run out.
#[tokio::test]
async fn a_scorer_slower_than_its_budget_or_broken_leaves_the_lexical_order_and_is_said_once() {
    let (release, stalled) = std::sync::mpsc::channel();
    let mut slow = Fixture::scoring(FILES, Arc::new(Stalled(Mutex::new(stalled))));
    slow.initialize(zed_client()).await;
    slow.initialized().await;
    slow.open("notes.md").await;
    let asked = json!({ "diagnostics": [], "triggerKind": 1 });
    let slow_order = offered(&mut slow, asked.clone()).await;
    release.send(()).expect("the scorer is still waiting");

    let mut broken = Fixture::scoring(FILES, Arc::new(Broken));
    broken.initialize(zed_client()).await;
    broken.initialized().await;
    broken.open("notes.md").await;
    let broken_order = offered(&mut broken, asked.clone()).await;
    let again = offered(&mut broken, asked).await;
    assert_eq!(broken_order, again);

    expect![[r#"
        slow: `leer` `letter` `lefter` `liter` `later`
        orthra: the context tier took longer than 80 ms
        broken: `leer` `letter` `lefter` `liter` `later`
        orthra: the context tier could not score: the model is not loaded"#]]
    .assert_eq(&format!(
        "slow: {slow_order}\n{}\nbroken: {broken_order}\n{}",
        logged(&slow),
        logged(&broken)
    ));
}
