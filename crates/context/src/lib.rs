//! The context tier's contract: the [`Scorer`] a model is used through, and the one thing it may
//! do with orthra's corrections, [`rerank`] them; and the [`Rewriter`] that rewrites a sentence
//! somebody asked to have rewritten. No model runtime lives here; runtimes are feature-gated
//! leaves (`docs/04-context.md`).
//!
//! The lexical tier says whether a word is a word, and offers its corrections. The context tier
//! only says which of those fits the sentence best: it never accepts or rejects a word, never
//! proposes one, and a scorer that fails or is slow leaves the lexical order as it was. A rewrite
//! is the one exception to "never proposes", and it is never a finding: an action a person chose,
//! whose edit they see.

use std::{collections::HashMap, fmt, sync::Arc};

use orthra_text::{TextRange, TextSize};

pub mod sentence;

#[cfg(test)]
mod tests;

/// A model's opinion of words in a sentence.
pub trait Scorer: Send + Sync + fmt::Debug {
    /// A score for each of `candidates` written in `slot` of `sentence`: higher is likelier. The
    /// candidates come best first by the lexical tier, so a scorer may weigh that order in, as
    /// its own measurement says to.
    fn score(
        &self,
        sentence: &str,
        slot: TextRange,
        candidates: &[&str],
    ) -> Result<Vec<f32>, ScoreError>;

    /// The memory the model holds, in bytes, as it counts it: what the server reports on a line of
    /// its own while the tier is on (`docs/04-context.md` §5).
    fn bytes(&self) -> usize {
        0
    }
}

/// What runs a model in process: it makes the file a model is installed as from what the model's
/// sources give, and reads that file into a [`Scorer`]. `orthra-ml-local` is one; the binary hands
/// it to whoever installs or loads a model, since nothing else may know a model runtime exists.
pub trait Runtime: Send + Sync + fmt::Debug {
    /// The installed model file, from the downloads by their file names (`model.safetensors`,
    /// `config.json`, `vocab.txt`) and the weight its definition gives the lexical order.
    fn build(&self, files: &HashMap<String, Vec<u8>>, prior: f32) -> Result<Vec<u8>, ScoreError>;

    /// The scorer an installed model file makes.
    fn load(&self, model: &[u8]) -> Result<Arc<dyn Scorer>, ScoreError>;
}

/// A scorer that could not answer, and why: the corrections keep the lexical order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreError(pub String);

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ScoreError {}

/// The sentence a word is scored in: its line, cut to [`WINDOW_BYTES`] around it at character
/// boundaries, and where the word is in it. Owned, so it can be scored on a thread of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub sentence: String,
    pub slot: TextRange,
}

/// The most of a line a word is scored in, in bytes. A line of prose is short -- 18 tokens at the
/// median on the typo sample -- and a 32-token window scored as well as 128 (2026-11-06).
pub const WINDOW_BYTES: usize = 400;

impl Window {
    /// The window around `range` of `text`.
    pub fn around(text: &str, range: TextRange) -> Window {
        let (start, end) = (usize::from(range.start()), usize::from(range.end()));
        let line_start = text[..start].rfind('\n').map_or(0, |at| at + 1);
        let line_end = text[end..].find('\n').map_or(text.len(), |at| end + at);
        let room = WINDOW_BYTES.saturating_sub(end - start) / 2;
        let mut from = line_start.max(start.saturating_sub(room));
        while !text.is_char_boundary(from) {
            from += 1;
        }
        let mut to = line_end.min(end + room);
        while !text.is_char_boundary(to) {
            to -= 1;
        }
        let offset = |at: usize| TextSize::try_from(at - from).unwrap_or_default();
        Window {
            sentence: text[from..to].to_owned(),
            slot: TextRange::new(offset(start), offset(end)),
        }
    }
}

/// `corrections`, given best first by the lexical tier, in the order `scorer` puts them, the
/// best-scored first and the lexical order among equals. The same corrections, never another:
/// a scorer whose answer does not fit them is an error, and the caller keeps the lexical order.
pub fn rerank(
    scorer: &dyn Scorer,
    window: &Window,
    corrections: Vec<String>,
) -> Result<Vec<String>, ScoreError> {
    if corrections.len() < 2 {
        return Ok(corrections);
    }
    let candidates: Vec<&str> = corrections.iter().map(String::as_str).collect();
    let scores = scorer.score(&window.sentence, window.slot, &candidates)?;
    if scores.len() != corrections.len() {
        return Err(ScoreError(format!(
            "{} scores for {} corrections",
            scores.len(),
            corrections.len()
        )));
    }
    if let Some(score) = scores.iter().find(|score| !score.is_finite()) {
        return Err(ScoreError(format!("a score of {score}")));
    }
    let mut order: Vec<usize> = (0..corrections.len()).collect();
    // Stable: equals keep the lexical order.
    order.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
    let mut corrections: Vec<Option<String>> = corrections.into_iter().map(Some).collect();
    Ok(order
        .into_iter()
        .filter_map(|at| corrections[at].take())
        .collect())
}

/// The scorer every test uses, with no model: deterministic, and adversarial. It scores the
/// lexical tier's last correction highest and its first lowest, so a test that passes with it
/// cannot be relying on a model being right (`docs/04-context.md` §3).
#[derive(Debug, Default, Clone, Copy)]
pub struct FakeScorer;

impl Scorer for FakeScorer {
    fn score(&self, _: &str, _: TextRange, candidates: &[&str]) -> Result<Vec<f32>, ScoreError> {
        Ok((0..candidates.len()).map(|at| at as f32).collect())
    }
}

/// What a rewrite asks of a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rewrite {
    /// The spelling and grammar corrected, and the wording kept wherever it is right.
    #[default]
    Correct,
    /// The same meaning, said more naturally.
    Rephrase,
}

impl Rewrite {
    /// The name a setting and an action's data give it: `rewrite`, `rephrase`.
    pub fn name(self) -> &'static str {
        match self {
            Rewrite::Correct => "rewrite",
            Rewrite::Rephrase => "rephrase",
        }
    }

    /// The rewrite `name` names, if it names one.
    pub fn named(name: &str) -> Option<Rewrite> {
        [Rewrite::Correct, Rewrite::Rephrase]
            .into_iter()
            .find(|rewrite| rewrite.name() == name)
    }
}

/// A model that rewrites a sentence, or a selection: corrected, or rephrased. The one generative
/// use of a model in orthra, and the one place the context tier may take seconds, because
/// somebody chose the action and is waiting for it (`docs/04-context.md` §2 C, rule 3).
pub trait Rewriter: Send + Sync + fmt::Debug {
    /// `text` rewritten as `how` asks. `lang` is the tag of the enabled language the text is
    /// written in, `en-US`, or `und` when no enabled language speaks its script.
    fn rewrite(&self, text: &str, lang: &str, how: Rewrite) -> Result<String, ScoreError>;
}

/// Where text is sent to be rewritten: an OpenAI-compatible endpoint the user runs (Ollama,
/// llama.cpp's server, LM Studio), the model it serves, and how to ask it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// The API's base, `http://127.0.0.1:11434/v1`: a request goes to `<url>/chat/completions`.
    pub url: String,
    pub model: String,
    /// Whether the endpoint may be on another machine. Off unless the user writes it: text stays
    /// on the machine (rule 7, `CLAUDE.md` §5).
    pub allow_remote: bool,
    /// Sent as `Authorization: Bearer`, for a server that asks for one.
    pub api_key: Option<ApiKey>,
    /// The settings' own prompt for each rewrite, in place of orthra's. `{language}` in one is
    /// where the language is named.
    pub prompts: Prompts,
}

/// The prompts the settings give, each in place of orthra's own.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Prompts {
    pub correct: Option<String>,
    pub rephrase: Option<String>,
}

impl Prompts {
    /// The settings' prompt for `how`, if they give one.
    pub fn of(&self, how: Rewrite) -> Option<&str> {
        match how {
            Rewrite::Correct => self.correct.as_deref(),
            Rewrite::Rephrase => self.rephrase.as_deref(),
        }
    }
}

/// An API key: shown as `<api key>` wherever an endpoint is printed, so that no log holds it.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(pub String);

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<api key>")
    }
}

/// What reaches a model that rewrites: `orthra-ml-http`'s, which the binary hands to the server,
/// since nothing else may know an HTTP client exists.
pub trait Connector: Send + Sync + fmt::Debug {
    /// A rewriter that asks `endpoint`, or why there can be none: an endpoint that is not on this
    /// machine, say. Sends nothing: a connector checks, and the first request is a rewrite.
    fn connect(&self, endpoint: &Endpoint) -> Result<Arc<dyn Rewriter>, ScoreError>;
}

/// The rewriter every test uses, with no model: deterministic, and never right by accident. A
/// correction upper-cases the text and a rephrasing lower-cases it, which no model does, so an
/// edit it made cannot be mistaken for the text it replaced, nor one rewrite for the other.
#[derive(Debug, Default, Clone, Copy)]
pub struct FakeRewriter;

impl Rewriter for FakeRewriter {
    fn rewrite(&self, text: &str, _: &str, how: Rewrite) -> Result<String, ScoreError> {
        Ok(match how {
            Rewrite::Correct => text.to_uppercase(),
            Rewrite::Rephrase => text.to_lowercase(),
        })
    }
}
