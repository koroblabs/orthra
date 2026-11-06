//! "Rewrite sentence (local model)": offered when somebody asks for the actions with the cursor in
//! text meant to be read, never as the cursor moves; resolved into an edit by the model the
//! settings name; no edit for a document changed since, nor from a model that fails, which is
//! shown once; and the findings are the lexical tier's either way.

use std::sync::{Arc, Mutex};

use expect_test::expect;
use orthra_engine::context::{Connector, Endpoint, FakeRewriter, Rewriter, ScoreError};
use serde_json::{Value, json};

use crate::tests::{Fixture, zed_client};

const NOTES: &str = "Fixx the leter. It are\nwrapped `here` today.\n\n```\ncode here\n```\n";
const FILES: &[(&str, &str)] = &[("notes.md", NOTES)];

/// How a pretend model answers.
#[derive(Debug, Clone, Copy, Default)]
enum Answer {
    /// As [`FakeRewriter`] does: upper-cased.
    #[default]
    Fake,
    /// With the sentence as it was.
    Same,
    /// Not at all.
    Broken,
}

/// A connector of the test's own, and the rewriters it makes: what each was asked, in order, and
/// no network.
#[derive(Debug, Default)]
struct Pretend {
    asked: Arc<Mutex<Vec<String>>>,
    answer: Answer,
}

impl Pretend {
    fn answering(answer: Answer) -> Arc<Pretend> {
        Arc::new(Pretend {
            answer,
            ..Pretend::default()
        })
    }

    fn asked(&self) -> String {
        or_none(self.asked.lock().unwrap().join("\n"))
    }
}

impl Connector for Pretend {
    fn connect(&self, endpoint: &Endpoint) -> Result<Arc<dyn Rewriter>, ScoreError> {
        self.asked.lock().unwrap().push(format!(
            "connect {} {} allow_remote={}",
            endpoint.url, endpoint.model, endpoint.allow_remote
        ));
        Ok(Arc::new(Recording {
            asked: Arc::clone(&self.asked),
            answer: self.answer,
        }))
    }
}

#[derive(Debug)]
struct Recording {
    asked: Arc<Mutex<Vec<String>>>,
    answer: Answer,
}

impl Rewriter for Recording {
    fn rewrite(&self, sentence: &str, lang: &str) -> Result<String, ScoreError> {
        self.asked
            .lock()
            .unwrap()
            .push(format!("rewrite {lang} {sentence:?}"));
        match self.answer {
            Answer::Fake => FakeRewriter.rewrite(sentence, lang),
            Answer::Same => Ok(sentence.to_owned()),
            Answer::Broken => Err(ScoreError(
                "http://127.0.0.1:11434/v1/chat/completions: io: Connection refused".to_owned(),
            )),
        }
    }
}

/// The settings that name an endpoint, as Zed's `initialization_options` would.
fn http() -> Value {
    json!({ "context": { "http": { "endpoint": "http://127.0.0.1:11434/v1", "model": "qwen3:4b" } } })
}

async fn started(files: &[(&str, &str)], connector: Arc<Pretend>, options: Value) -> Fixture {
    let mut server = Fixture::rewriting(files, connector);
    server.initialize_with(zed_client(), options).await;
    server.initialized().await;
    server.open("notes.md").await;
    server
}

fn at(line: u32, character: u32) -> Value {
    json!({ "start": { "line": line, "character": character }, "end": { "line": line, "character": character } })
}

/// The actions at `range`, asked for in `context`.
async fn actions(server: &mut Fixture, range: Value, context: Value) -> Vec<Value> {
    server.actions_in("notes.md", range, context).await
}

fn asked() -> Value {
    json!({ "diagnostics": [], "triggerKind": 1 })
}

/// `text`, or `none` for nothing: a snapshot line never ends in a space.
fn or_none(text: String) -> String {
    if text.is_empty() {
        "none".to_owned()
    } else {
        text
    }
}

fn titles(actions: &[Value]) -> String {
    or_none(
        actions
            .iter()
            .map(|action| action["title"].as_str().unwrap_or("?"))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

/// `value` as JSON, with the fixture's directory taken out of its URIs.
fn shown(server: &Fixture, value: &Value) -> String {
    let root = server.uri("notes.md");
    serde_json::to_string_pretty(value)
        .unwrap()
        .replace(root.as_str(), "file:///notes.md")
}

/// The one rewrite action among `actions`.
fn the_rewrite(actions: &[Value]) -> Value {
    let rewrites: Vec<&Value> = actions
        .iter()
        .filter(|action| action["kind"] == "refactor.rewrite")
        .collect();
    assert_eq!(rewrites.len(), 1, "{actions:#?}");
    rewrites[0].clone()
}

/// What the server said in messages and the log about rewriting.
fn said(server: &Fixture) -> String {
    or_none(
        ["window/showMessage", "window/logMessage"]
            .iter()
            .flat_map(|method| {
                server
                    .flow
                    .params(method)
                    .into_iter()
                    .map(move |params| (method, params))
            })
            .filter_map(|(method, params)| {
                let message = params["message"].as_str()?;
                (message.contains("ewrit") || message.contains("sentence"))
                    .then(|| format!("{method}: {message}"))
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[tokio::test]
async fn asked_the_sentence_at_the_cursor_is_offered_then_rewritten() {
    let pretend = Pretend::answering(Answer::Fake);
    let mut server = started(FILES, Arc::clone(&pretend), http()).await;
    let mut lexical = Fixture::new(FILES);
    lexical.initialize(zed_client()).await;
    lexical.initialized().await;
    lexical.open("notes.md").await;
    assert_eq!(
        server.published("notes.md"),
        lexical.published("notes.md"),
        "the findings are the lexical tier's"
    );

    // `are`, on the first line, in a sentence that wraps onto the second.
    let offered = actions(&mut server, at(0, 20), asked()).await;
    let older = actions(&mut server, at(0, 20), json!({ "diagnostics": [] })).await;
    assert_eq!(offered, older, "an older client sends no trigger kind");
    let moved = actions(
        &mut server,
        at(0, 20),
        json!({ "diagnostics": [], "triggerKind": 2 }),
    )
    .await;
    let quick_fixes_only = actions(
        &mut server,
        at(0, 20),
        json!({ "diagnostics": [], "triggerKind": 1, "only": ["quickfix"] }),
    )
    .await;
    let refactors = actions(
        &mut server,
        at(0, 20),
        json!({ "diagnostics": [], "triggerKind": 1, "only": ["refactor"] }),
    )
    .await;
    let on_a_finding = actions(&mut server, at(0, 10), asked()).await;
    let in_code = actions(&mut server, at(4, 2), asked()).await;
    expect![[r#"
        asked: Rewrite sentence (local model)
        the cursor moved: none
        only quick fixes: none
        only refactors: Rewrite sentence (local model)
        on a finding: Change to `leer`, Change to `letter`, Change to `lefter`, Change to `liter`, Change to `later`, Add `leter` to the project's words, Add `leter` to your words, Ignore `leter` in this file, Rewrite sentence (local model)
        in a code block: none"#]]
    .assert_eq(
        &[
            ("asked", &offered),
            ("the cursor moved", &moved),
            ("only quick fixes", &quick_fixes_only),
            ("only refactors", &refactors),
            ("on a finding", &on_a_finding),
            ("in a code block", &in_code),
        ]
        .map(|(when, actions)| format!("{when}: {}", titles(actions)))
        .join("\n"),
    );

    let action = the_rewrite(&offered);
    expect![[r#"
        {
          "data": {
            "rewrite": {
              "range": {
                "end": {
                  "character": 21,
                  "line": 1
                },
                "start": {
                  "character": 16,
                  "line": 0
                }
              },
              "uri": "file:///notes.md",
              "version": 1
            }
          },
          "kind": "refactor.rewrite",
          "title": "Rewrite sentence (local model)"
        }"#]]
    .assert_eq(&shown(&server, &action));

    let resolved = server.request("codeAction/resolve", action).await;
    expect![[r#"
        {
          "data": {
            "rewrite": {
              "range": {
                "end": {
                  "character": 21,
                  "line": 1
                },
                "start": {
                  "character": 16,
                  "line": 0
                }
              },
              "uri": "file:///notes.md",
              "version": 1
            }
          },
          "edit": {
            "changes": {
              "file:///notes.md": [
                {
                  "newText": "IT ARE\nWRAPPED `HERE` TODAY.",
                  "range": {
                    "end": {
                      "character": 21,
                      "line": 1
                    },
                    "start": {
                      "character": 16,
                      "line": 0
                    }
                  }
                }
              ]
            }
          },
          "kind": "refactor.rewrite",
          "title": "Rewrite sentence (local model)"
        }"#]]
    .assert_eq(&shown(&server, &resolved));
    expect![[r#"
        connect http://127.0.0.1:11434/v1 qwen3:4b allow_remote=false
        rewrite en-US "It are\nwrapped `here` today."
        window/logMessage: orthra: "Rewrite sentence" asks qwen3:4b at http://127.0.0.1:11434/v1"#]]
    .assert_eq(&format!("{}\n{}", pretend.asked(), said(&server)));
}

#[tokio::test]
async fn the_server_says_it_resolves_actions_only_when_it_can_rewrite() {
    let provider = |options: Value| async move {
        let mut server = Fixture::rewriting(FILES, Pretend::answering(Answer::Fake));
        let result = server.initialize_with(zed_client(), options).await;
        result["capabilities"]["codeActionProvider"].to_string()
    };
    expect![[r#"
        {"codeActionKinds":["quickfix","refactor.rewrite"],"resolveProvider":true}
        {"codeActionKinds":["quickfix"],"resolveProvider":false}"#]]
    .assert_eq(&format!(
        "{}\n{}",
        provider(http()).await,
        provider(Value::Null).await
    ));
}

#[tokio::test]
async fn a_document_changed_since_the_action_was_offered_gets_no_edit() {
    let pretend = Pretend::answering(Answer::Fake);
    let mut server = started(FILES, Arc::clone(&pretend), http()).await;
    let action = the_rewrite(&actions(&mut server, at(0, 20), asked()).await);
    let uri = server.uri("notes.md");
    server
        .notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": 2 },
                "contentChanges": [{
                    "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                    "text": "Now: ",
                }],
            }),
        )
        .await;
    let resolved = server.request("codeAction/resolve", action.clone()).await;
    assert_eq!(resolved, action, "the action, as it was offered");
    expect!["connect http://127.0.0.1:11434/v1 qwen3:4b allow_remote=false"]
        .assert_eq(&pretend.asked());
}

#[tokio::test]
async fn a_model_that_fails_gives_no_edit_and_is_shown_once() {
    let broken = Pretend::answering(Answer::Broken);
    let mut server = started(FILES, Arc::clone(&broken), http()).await;
    let action = the_rewrite(&actions(&mut server, at(0, 20), asked()).await);
    let first = server.request("codeAction/resolve", action.clone()).await;
    let second = server.request("codeAction/resolve", action.clone()).await;
    assert_eq!([&first, &second], [&action, &action], "no edit");

    let same = Pretend::answering(Answer::Same);
    let mut unchanged = started(FILES, Arc::clone(&same), http()).await;
    let action = the_rewrite(&actions(&mut unchanged, at(0, 20), asked()).await);
    let resolved = unchanged
        .request("codeAction/resolve", action.clone())
        .await;
    assert_eq!(resolved, action, "no edit");
    expect![[r#"
        window/showMessage: orthra: the sentence was not rewritten: http://127.0.0.1:11434/v1/chat/completions: io: Connection refused
        window/logMessage: orthra: "Rewrite sentence" asks qwen3:4b at http://127.0.0.1:11434/v1
        window/logMessage: orthra: the sentence was not rewritten: http://127.0.0.1:11434/v1/chat/completions: io: Connection refused
        --
        window/showMessage: orthra: the model left the sentence as it was
        window/logMessage: orthra: "Rewrite sentence" asks qwen3:4b at http://127.0.0.1:11434/v1"#]]
    .assert_eq(&format!("{}\n--\n{}", said(&server), said(&unchanged)));
}

#[tokio::test]
async fn an_endpoint_allowed_to_be_remote_is_named_where_local_would_be() {
    let pretend = Pretend::answering(Answer::Fake);
    let options = json!({ "context": { "http": {
        "endpoint": "https://models.example.com/v1", "model": "big", "allowRemote": true,
    } } });
    let mut server = started(FILES, Arc::clone(&pretend), options).await;
    let offered = actions(&mut server, at(0, 20), asked()).await;
    expect![[r#"
        Rewrite sentence (model at https://models.example.com/v1)
        connect https://models.example.com/v1 big allow_remote=true"#]]
    .assert_eq(&format!("{}\n{}", titles(&offered), pretend.asked()));
}

#[tokio::test]
async fn no_endpoint_no_rewrite_and_one_that_cannot_be_had_is_said() {
    let options = [
        json!({ "context": { "enabled": true } }),
        json!({ "context": { "enabled": false, "http": { "model": "qwen3:4b" } } }),
        json!({ "context": { "http": { "endpoint": "http://127.0.0.1:11434/v1" } } }),
    ];
    let mut got = Vec::new();
    for options in options {
        let pretend = Pretend::answering(Answer::Fake);
        let mut server = started(FILES, Arc::clone(&pretend), options.clone()).await;
        let offered = actions(&mut server, at(0, 20), asked()).await;
        got.push(format!(
            "{options}\n  actions: {}\n  asked: {}\n  said: {}",
            titles(&offered),
            pretend.asked(),
            said(&server)
        ));
    }
    // A binary built without `ml-http` has no connector to hand the server.
    let mut without = Fixture::new(FILES);
    without.initialize_with(zed_client(), http()).await;
    without.initialized().await;
    got.push(format!("without ml-http\n  said: {}", said(&without)));
    expect![[r#"
        {"context":{"enabled":true}}
          actions: none
          asked: none
          said: none
        {"context":{"enabled":false,"http":{"model":"qwen3:4b"}}}
          actions: none
          asked: none
          said: none
        {"context":{"http":{"endpoint":"http://127.0.0.1:11434/v1"}}}
          actions: none
          asked: none
          said: window/showMessage: orthra: `context.http` names no `model` for its endpoint to run, so there is no "Rewrite sentence" action
        without ml-http
          said: window/showMessage: orthra: this orthra was built without `ml-http`, so there is no "Rewrite sentence" action"#]]
    .assert_eq(&got.join("\n"));
}
