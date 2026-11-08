//! "Rewrite sentence (local model)": a code action that hands the sentence at the cursor to a model
//! the user runs, and writes what it answers over it (`docs/04-context.md` §2 C). With text
//! selected, it is "Rewrite selection", which hands over what is selected, beside "Rephrase
//! selection", which asks for the same meaning said more naturally.
//!
//! It is offered only when somebody asks for the actions, and only with the cursor, or the start of
//! the selection, in text meant to be read. The action carries no edit: only the document, its
//! version and the range to rewrite.
//! The edit is made in `codeAction/resolve`, once the action is chosen, off the server's lock and
//! on a blocking thread. That is the one place the context tier may take seconds, because the
//! person chose it and is waiting (rule 3). It is never a diagnostic and never made on its own.
//! A document changed since gets no edit, and neither does a model that fails, which is said.

use std::{collections::HashMap, sync::Arc, time::Duration};

use lspkit_runtime::protocol::ls_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit, Uri, WorkspaceEdit,
};
use orthra_engine::context::{Connector, Endpoint, Rewrite, Rewriter};
use orthra_text::line_index::LineIndex;
use serde_json::{Value, json};

use crate::state::State;

/// The kind the action is, and the kinds the server says it offers with it.
pub(crate) const KIND: CodeActionKind = CodeActionKind::REFACTOR_REWRITE;

/// The most a rewrite is waited for. The HTTP rewriter gives up at 20 s on its own; this is for a
/// rewriter that does not, so that a request is answered whatever it does.
const BUDGET: Duration = Duration::from_secs(30);

/// A rewriter, and what its action is called in the client's menu.
pub(crate) struct Rewriting {
    rewriter: Arc<dyn Rewriter>,
    /// Where the model is, as the menu says it: "local model", or the endpoint.
    model: String,
}

impl Rewriting {
    /// The model is "local model", or the endpoint named when the settings allow it to be another
    /// machine: a project's `.zed/settings.json` can set the editor's settings, and the menu must
    /// not say "local" of text that may leave the machine.
    fn new(rewriter: Arc<dyn Rewriter>, endpoint: &Endpoint) -> Rewriting {
        let model = if endpoint.allow_remote {
            format!("model at {}", endpoint.url)
        } else {
            "local model".to_owned()
        };
        Rewriting { rewriter, model }
    }

    /// "Rewrite sentence (local model)", "Rewrite selection (...)", "Rephrase selection (...)".
    fn title(&self, how: Rewrite, what: &str) -> String {
        let verb = match how {
            Rewrite::Correct => "Rewrite",
            Rewrite::Rephrase => "Rephrase",
        };
        format!("{verb} {what} ({})", self.model)
    }
}

/// The rewriter the settings ask for, through `connector`, and the endpoint it asks: `None` when
/// they ask for none, and why there is none when they ask for one that cannot be had.
pub(crate) fn connect(
    asked: Result<Option<Endpoint>, String>,
    connector: Option<&Arc<dyn Connector>>,
) -> Result<Option<(Rewriting, Endpoint)>, String> {
    let Some(endpoint) = asked? else {
        return Ok(None);
    };
    let connector = connector.ok_or("this orthra was built without `ml-http`")?;
    let rewriter = connector.connect(&endpoint).map_err(|err| err.0)?;
    Ok(Some((Rewriting::new(rewriter, &endpoint), endpoint)))
}

/// Whether a client that asked `only` for some kinds of action asked for this one: `refactor`
/// and `refactor.rewrite` do, `quickfix` does not.
pub(crate) fn wanted(only: Option<&[CodeActionKind]>) -> bool {
    let kind = KIND;
    let kind = kind.as_str();
    only.is_none_or(|kinds| {
        kinds.iter().any(|asked| {
            kind.strip_prefix(asked.as_str())
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
        })
    })
}

/// The actions for what is selected in the open document `uri`, rewritten or rephrased, or the one
/// for the sentence at the cursor when nothing is: when a rewriter is connected and the selection
/// starts, or the cursor is, in text meant to be read. Rephrasing a sentence nobody selected would
/// be one more entry in every menu.
pub(crate) fn offer(state: &mut State, uri: &Uri, selected: Range) -> Vec<CodeActionOrCommand> {
    offered(state, uri, selected).unwrap_or_default()
}

fn offered(state: &mut State, uri: &Uri, selected: Range) -> Option<Vec<CodeActionOrCommand>> {
    let State {
        engine,
        documents,
        encoding,
        rewriting,
        ..
    } = state;
    let rewriting = rewriting.as_ref()?;
    let engine = engine.as_ref()?;
    let (document, parsed) = documents.parsed_for(uri)?;
    let text = &document.text;
    let index = LineIndex::new(text);
    let at = lspkit_text::lsp::offset(&index, text, selected.start, *encoding)?;
    let end = lspkit_text::lsp::offset(&index, text, selected.end, *encoding)?;
    let sentence = engine.sentence_at(&document.path, &document.language, text, at, parsed)?;
    let (what, range, kinds): (_, _, &[Rewrite]) =
        if end > at && !text[usize::from(at)..usize::from(end)].trim().is_empty() {
            (
                "selection",
                selected,
                &[Rewrite::Correct, Rewrite::Rephrase],
            )
        } else {
            (
                "sentence",
                lspkit_text::lsp::range(&index, text, sentence, *encoding)?,
                &[Rewrite::Correct],
            )
        };
    let version = document.version;
    Some(
        kinds
            .iter()
            .map(|&how| {
                let asked = Asked {
                    uri: uri.clone(),
                    version,
                    range,
                    how,
                };
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: rewriting.title(how, what),
                    kind: Some(KIND),
                    data: Some(asked.data()),
                    ..CodeAction::default()
                })
            })
            .collect(),
    )
}

/// What a rewrite action carries to `codeAction/resolve`: the document, the version it was
/// offered for, and where the sentence is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Asked {
    uri: Uri,
    version: i32,
    range: Range,
    how: Rewrite,
}

impl Asked {
    fn data(&self) -> Value {
        json!({ "rewrite": {
            "uri": self.uri,
            "version": self.version,
            "range": self.range,
            "how": self.how.name(),
        } })
    }

    /// The rewrite `action` asks for, if it is a rewrite: every other action has its edit already.
    pub(crate) fn of(action: &CodeAction) -> Option<Asked> {
        let asked = action.data.as_ref()?.get("rewrite")?;
        Some(Asked {
            uri: serde_json::from_value(asked.get("uri")?.clone()).ok()?,
            version: i32::try_from(asked.get("version")?.as_i64()?).ok()?,
            range: serde_json::from_value(asked.get("range")?.clone()).ok()?,
            how: Rewrite::named(asked.get("how")?.as_str()?)?,
        })
    }

    /// The rewrite to run, read under the server's lock: `None` when the document is not the one
    /// the action was offered for, since the range is no longer that sentence.
    pub(crate) fn job(&self, state: &State) -> Option<Job> {
        let rewriter = Arc::clone(&state.rewriting.as_ref()?.rewriter);
        let document = state.documents.get(&self.uri)?;
        if document.version != self.version {
            return None;
        }
        let text = &document.text;
        let index = LineIndex::new(text);
        let offset = |at: Position| lspkit_text::lsp::offset(&index, text, at, state.encoding);
        let (start, end) = (offset(self.range.start)?, offset(self.range.end)?);
        let sentence = text.get(usize::from(start)..usize::from(end))?.to_owned();
        let lang = state
            .engine
            .as_ref()
            .and_then(|engine| engine.language_of(&sentence))
            .unwrap_or("und")
            .to_owned();
        Some(Job {
            sentence,
            lang,
            how: self.how,
            rewriter,
        })
    }

    /// Whether the document is still the one the rewrite was made for: it may have changed while
    /// the model ran.
    pub(crate) fn is_current(&self, state: &State) -> bool {
        state
            .documents
            .get(&self.uri)
            .is_some_and(|document| document.version == self.version)
    }

    /// The edit that writes `rewrite` over the sentence.
    pub(crate) fn edit(&self, rewrite: String) -> WorkspaceEdit {
        WorkspaceEdit {
            changes: Some(HashMap::from([(
                self.uri.clone(),
                vec![TextEdit::new(self.range, rewrite)],
            )])),
            ..WorkspaceEdit::default()
        }
    }
}

/// A sentence to rewrite, in its language, and the rewriter to ask.
pub(crate) struct Job {
    sentence: String,
    lang: String,
    how: Rewrite,
    rewriter: Arc<dyn Rewriter>,
}

impl Job {
    /// The sentence rewritten, `None` when the model left it as it was, or why there is none.
    /// Asked off the server's thread, and waited for at most [`BUDGET`].
    pub(crate) async fn run(self) -> Result<Option<String>, String> {
        let Job {
            sentence,
            lang,
            how,
            rewriter,
        } = self;
        let asked = sentence.clone();
        let rewritten = tokio::task::spawn_blocking(move || rewriter.rewrite(&asked, &lang, how));
        match tokio::time::timeout(BUDGET, rewritten).await {
            Ok(Ok(Ok(rewrite))) => Ok((rewrite != sentence).then_some(rewrite)),
            Ok(Ok(Err(err))) => Err(err.0),
            Ok(Err(_)) => Err("the rewriter panicked".to_owned()),
            Err(_) => Err(format!("no answer in {} s", BUDGET.as_secs())),
        }
    }
}
