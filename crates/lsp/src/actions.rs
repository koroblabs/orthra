//! What can be done about a finding: change the word, here or wherever the file has the same
//! finding, add it to the project's words or the user's, or ignore it in this file or on its line.
//! And what can be done about the file: apply every fix that is the only one its finding has, at
//! once, as a `source.fixAll` action a client can also run when it saves.
//!
//! Suggestions are computed here, for the one word asked about, and never while diagnosing
//! (`CLAUDE.md` §5). They cost half a millisecond at the 95th percentile, so the actions carry
//! their edits whole and nothing is left for `codeAction/resolve`.
//!
//! The actions are planned under the server's lock and built after it, so that the context tier,
//! when it is on, can reorder an unknown word's corrections in between without holding anything
//! up ([`rerank`]).

use std::{collections::HashMap, sync::Arc, time::Duration};

use lspkit_runtime::protocol::ls_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, Position, Range,
    TextEdit, Uri, WorkspaceEdit,
};
use orthra_engine::{
    IGNORE_DIRECTIVE, NEXT_LINE,
    context::{self, Scorer, Window},
};
use orthra_project::config::{self, Config};
use orthra_text::{
    TextSize,
    line_index::{Encoding, LineIndex},
};
use serde_json::{Value, json};

use crate::{convert, state::State};

#[cfg(test)]
mod tests;

/// The command an "add to words" action runs: `{"scope": "project" | "user", "word": "qzxv"}`.
pub(crate) const ADD_WORD: &str = "orthra.addWord";

/// The kind of the action that applies every exact fix in a file. A client runs it on its own when
/// asked to: Zed's `"code_actions_on_format": { "source.fixAll.orthra": true }`.
pub(crate) const FIX_ALL: CodeActionKind = CodeActionKind::new("source.fixAll.orthra");

/// How many suggestions a finding offers.
const SUGGESTIONS: usize = 5;

/// The most a code action waits for the context tier to reorder its corrections: `CLAUDE.md` §3's
/// target for code actions with context re-ranking. ELECTRA-small in Q8_0 took 17 ms at the 95th
/// percentile on 4 threads (`docs/04-context.md` §4). Past it the lexical order stands.
pub(crate) const RERANK_BUDGET: Duration = Duration::from_millis(80);

/// What the actions for the findings in a range will be, worked out under the server's lock.
pub(crate) struct Plan {
    uri: Uri,
    has_project: bool,
    has_user: bool,
    findings: Vec<Planned>,
    /// Every finding in the file whose fix is the only one: a known typo with one correction, a
    /// word in two scripts. Where it is, its word, and the fix.
    exact: Vec<(Range, String, String)>,
}

/// One finding's actions.
struct Planned {
    diagnostic: Diagnostic,
    word: String,
    fixes: Vec<String>,
    /// The first fix is the fix, and the client may apply it on its own: a known typo's, and a mixed
    /// word's in one script.
    preferred: bool,
    /// Where an unknown word is, for the context tier to score its corrections in.
    window: Option<Window>,
    /// Every place the file has the same finding, the same word under the same check, when there
    /// is more than this one. Only a finding is ever changed: a use of a name is never one.
    everywhere: Vec<Range>,
    ignore: Option<TextEdit>,
    ignore_line: Option<TextEdit>,
}

/// The plan for the findings in `range` of the open document at `uri`; `None` with nothing open.
pub(crate) fn plan(state: &mut State, uri: &Uri, range: Range) -> Option<Plan> {
    let encoding = state.encoding;
    let severity = state.severity();
    let has_project = !state.roots.is_empty();
    let has_user = state.user.is_some();
    let (findings, text, _) = state.findings(uri)?;
    let text = text.to_owned();
    let (path, language) = {
        let document = state.documents.get(uri).expect("findings came from it");
        (document.path.clone(), document.language.clone())
    };
    let engine = state.engine.as_ref()?;
    let comment = engine.comment_syntax(&path, &language);
    let index = LineIndex::new(&text);
    let mut planned = Vec::new();
    let mut exact = Vec::new();
    for finding in &findings {
        let Some(diagnostic) = convert::diagnostic(&index, &text, finding, encoding, severity)
        else {
            continue;
        };
        if let [fix] = finding.fixes.as_slice()
            && matches!(finding.code.0, "known-typo" | "mixed-script")
        {
            exact.push((
                diagnostic.range,
                text[finding.range].to_owned(),
                fix.clone(),
            ));
        }
        if !overlaps(diagnostic.range, range) {
            continue;
        }
        let word = &text[finding.range];
        let everywhere: Vec<Range> = findings
            .iter()
            .filter(|other| other.code == finding.code && text[other.range] == *word)
            .filter_map(|other| convert::diagnostic(&index, &text, other, encoding, severity))
            .map(|other| other.range)
            .collect();
        let unknown = finding.fixes.is_empty() && finding.code.0 == "unknown-word";
        let fixes = if unknown {
            engine.suggest(word, SUGGESTIONS)
        } else {
            finding.fixes.clone()
        };
        planned.push(Planned {
            window: (unknown && fixes.len() > 1).then(|| Window::around(&text, finding.range)),
            // A known typo's fix, and a mixed word in one script, are the fix: nothing to choose.
            preferred: matches!(finding.code.0, "known-typo" | "mixed-script"),
            ignore: comment
                .and_then(|(open, close)| ignore_edit(&text, &index, encoding, word, open, close)),
            ignore_line: comment.and_then(|(open, close)| {
                next_line_edit(&text, &index, encoding, finding.range.start(), open, close)
            }),
            everywhere: if everywhere.len() > 1 {
                everywhere
            } else {
                Vec::new()
            },
            word: word.to_owned(),
            fixes,
            diagnostic,
        });
    }
    Some(Plan {
        uri: uri.clone(),
        has_project,
        has_user,
        findings: planned,
        exact,
    })
}

/// Whether a client that asked `only` for some kinds of action asked for `kind`: `refactor` asks for
/// `refactor.rewrite`, `source` for `source.fixAll.orthra`, and no `only` for every kind.
pub(crate) fn asked_for(only: Option<&[CodeActionKind]>, kind: &CodeActionKind) -> bool {
    only.is_none_or(|kinds| kinds.iter().any(|asked| is_within(kind, asked)))
}

/// Whether `kind` is `asked` or one of its sub-kinds.
fn is_within(kind: &CodeActionKind, asked: &CodeActionKind) -> bool {
    kind.as_str()
        .strip_prefix(asked.as_str())
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

impl Plan {
    /// Whether to offer to apply every exact fix in the file. When a client asks for that kind by
    /// name, as it does when it saves, whenever there is one. In the menu somebody opened on a
    /// finding that has one, when the file has exact fixes of more than one word: one word's are
    /// already offered as "Change every `teh`". Never in a menu a client asks for on its own as
    /// the cursor moves, which would light a bulb on every line of the file.
    pub(crate) fn offers_fix_all(&self, only: Option<&[CodeActionKind]>, asked: bool) -> bool {
        if self.exact.is_empty() {
            return false;
        }
        if let Some(kinds) = only {
            return kinds.iter().any(|kind| is_within(&FIX_ALL, kind));
        }
        let here = self.findings.iter().any(|planned| {
            self.exact
                .iter()
                .any(|(range, _, _)| *range == planned.diagnostic.range)
        });
        let first = &self.exact[0].1;
        asked && here && self.exact.iter().any(|(_, word, _)| word != first)
    }

    /// The actions: each finding's corrections first, in the order the plan holds them, when
    /// `quick_fixes` are asked for, and then the one that applies every exact fix, with `fix_all`.
    pub(crate) fn actions(self, quick_fixes: bool, fix_all: bool) -> Vec<CodeActionOrCommand> {
        let uri = &self.uri;
        let mut actions = Vec::new();
        let findings = if quick_fixes {
            self.findings
        } else {
            Vec::new()
        };
        for planned in findings {
            let diagnostic = &planned.diagnostic;
            for (i, fix) in planned.fixes.iter().enumerate() {
                let edit = TextEdit::new(diagnostic.range, fix.clone());
                actions.push(quick_fix(
                    format!("Change to `{fix}`"),
                    diagnostic,
                    edit_of(uri, vec![edit]),
                    i == 0 && planned.preferred,
                ));
            }
            let word = &planned.word;
            if let Some(fix) = planned.fixes.first()
                && !planned.everywhere.is_empty()
            {
                let edits = planned
                    .everywhere
                    .iter()
                    .map(|&range| TextEdit::new(range, fix.clone()))
                    .collect();
                actions.push(quick_fix(
                    format!("Change every `{word}` in this file to `{fix}`"),
                    diagnostic,
                    edit_of(uri, edits),
                    false,
                ));
            }
            if self.has_project {
                actions.push(add_word(word, "project", "the project's words", diagnostic));
            }
            if self.has_user {
                actions.push(add_word(word, "user", "your words", diagnostic));
            }
            if let Some(edit) = planned.ignore {
                actions.push(quick_fix(
                    format!("Ignore `{word}` in this file"),
                    diagnostic,
                    edit_of(uri, vec![edit]),
                    false,
                ));
            }
            if let Some(edit) = planned.ignore_line {
                actions.push(quick_fix(
                    "Ignore this line".to_owned(),
                    diagnostic,
                    edit_of(uri, vec![edit]),
                    false,
                ));
            }
        }
        if fix_all && !self.exact.is_empty() {
            let count = self.exact.len();
            let edits = self
                .exact
                .into_iter()
                .map(|(range, _, fix)| TextEdit::new(range, fix))
                .collect();
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Apply every exact fix in this file ({count})"),
                kind: Some(FIX_ALL),
                edit: Some(edit_of(uri, edits)),
                ..CodeAction::default()
            }));
        }
        actions
    }

    /// Each unknown word's corrections in the order `scorer` puts them, scored off the server's
    /// thread and waited for at most `budget` a word. A scorer that is slower, fails or panics
    /// leaves the lexical order, and its failure is returned to be said once: the context tier is
    /// optional, and failing quietly is its contract (`docs/04-context.md` §5). Never a word that
    /// was not a correction already.
    pub(crate) async fn rerank(
        &mut self,
        scorer: &Arc<dyn Scorer>,
        budget: Duration,
    ) -> Option<String> {
        let mut failed = None;
        for planned in &mut self.findings {
            let Some(window) = planned.window.take() else {
                continue;
            };
            let scorer = Arc::clone(scorer);
            let lexical = planned.fixes.clone();
            let scored =
                tokio::task::spawn_blocking(move || context::rerank(&*scorer, &window, lexical));
            match tokio::time::timeout(budget, scored).await {
                Ok(Ok(Ok(order))) => planned.fixes = order,
                Ok(Ok(Err(err))) => {
                    failed = Some(format!("the context tier could not score: {err}"))
                }
                Ok(Err(_)) => failed = Some("the context tier's scorer panicked".to_owned()),
                Err(_) => {
                    failed = Some(format!(
                        "the context tier took longer than {} ms",
                        budget.as_millis()
                    ));
                }
            }
        }
        failed
    }
}

/// Writes the word an [`ADD_WORD`] command names into the words file its scope names. The pass
/// that follows reads it back, as it would after any edit to the file.
pub(crate) fn add_to_words(state: &State, arguments: &[Value]) -> Result<(), String> {
    let argument = arguments.first().ok_or("orthra.addWord needs a word")?;
    let word = argument["word"]
        .as_str()
        .filter(|word| !word.trim().is_empty())
        .ok_or("orthra.addWord needs a word")?;
    let file = match argument["scope"].as_str() {
        Some("project") => state
            .config
            .as_ref()
            .ok_or("no project is open")?
            .project_words(),
        Some("user") => Config::user_words(state.user.as_deref().ok_or("no user directory")?),
        _ => return Err("orthra.addWord's scope is `project` or `user`".to_owned()),
    };
    config::add_word(&file, word.trim()).map_err(|err| format!("{}: {err}", file.display()))
}

fn overlaps(a: Range, b: Range) -> bool {
    a.start <= b.end && b.start <= a.end
}

fn edit_of(uri: &Uri, edits: Vec<TextEdit>) -> WorkspaceEdit {
    WorkspaceEdit {
        changes: Some(HashMap::from([(uri.clone(), edits)])),
        ..WorkspaceEdit::default()
    }
}

fn quick_fix(
    title: String,
    diagnostic: &Diagnostic,
    edit: WorkspaceEdit,
    preferred: bool,
) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(edit),
        is_preferred: preferred.then_some(true),
        ..CodeAction::default()
    })
}

fn add_word(word: &str, scope: &str, whose: &str, diagnostic: &Diagnostic) -> CodeActionOrCommand {
    let title = format!("Add `{word}` to {whose}");
    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.clone(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        command: Some(Command {
            title,
            command: ADD_WORD.to_owned(),
            arguments: Some(vec![json!({ "scope": scope, "word": word })]),
        }),
        ..CodeAction::default()
    })
}

/// The edit that makes the file ignore `word`: the word added to the file's last directive, or a
/// directive of its own at the end of the file, in the file's comment syntax.
fn ignore_edit(
    text: &str,
    index: &LineIndex,
    encoding: Encoding,
    word: &str,
    open: &str,
    close: &str,
) -> Option<TextEdit> {
    let position = |at: usize| -> Option<Position> {
        lspkit_text::lsp::position(index, text, TextSize::try_from(at).ok()?, encoding)
    };
    // The last directive that lists words: an `ignore-next-line` lists none.
    let last = text
        .rmatch_indices(IGNORE_DIRECTIVE)
        .map(|(at, _)| at)
        .find(|&at| !text[at + IGNORE_DIRECTIVE.len()..].starts_with(NEXT_LINE));
    if let Some(at) = last {
        let line_end = text[at..].find('\n').map_or(text.len(), |end| at + end);
        let line = &text[at..line_end];
        // Before the comment's close, when it has one on this line: `qzxv -->`.
        let insert = match (close.is_empty(), line.rfind(close.trim())) {
            (false, Some(close_at)) => at + line[..close_at].trim_end().len(),
            _ => at + line.trim_end().len(),
        };
        let here = position(insert)?;
        return Some(TextEdit::new(Range::new(here, here), format!(" {word}")));
    }
    let end = position(text.len())?;
    let lead = if text.is_empty() || text.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    let close = if close.is_empty() {
        String::new()
    } else {
        format!(" {close}")
    };
    Some(TextEdit::new(
        Range::new(end, end),
        format!("{lead}{open} {IGNORE_DIRECTIVE} {word}{close}\n"),
    ))
}

/// The edit that makes the line holding `at` ignored: a directive on a line of its own before it,
/// indented as it is, in the file's comment syntax.
fn next_line_edit(
    text: &str,
    index: &LineIndex,
    encoding: Encoding,
    at: TextSize,
    open: &str,
    close: &str,
) -> Option<TextEdit> {
    let at = usize::from(at);
    let line_start = text[..at].rfind('\n').map_or(0, |newline| newline + 1);
    let line = &text[line_start..];
    let indent = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
    let here =
        lspkit_text::lsp::position(index, text, TextSize::try_from(line_start).ok()?, encoding)?;
    let close = if close.is_empty() {
        String::new()
    } else {
        format!(" {close}")
    };
    Some(TextEdit::new(
        Range::new(here, here),
        format!("{indent}{open} {IGNORE_DIRECTIVE}{NEXT_LINE}{close}\n"),
    ))
}
