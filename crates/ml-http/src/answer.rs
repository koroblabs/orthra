//! What a model answered, read as a rewrite of the sentence it was given, or refused as not one.
//!
//! Taken off, conservatively, only when it wraps the whole answer: a thinking model's
//! `<think>...</think>` in front, a code fence, and quotes the sentence did not start with.
//! Refused: an empty answer, one with more lines than the sentence, and one much longer than it.
//! A refusal leaves the document as it is and says why; an edit nobody can trust is worse.

use orthra_context::ScoreError;
use serde_json::Value;

#[cfg(test)]
mod tests;

/// The most a rewrite may be, against the sentence's length in characters: three times as long,
/// and forty more for a short one. A model that answers longer is explaining, not rewriting.
fn longest(sentence: usize) -> usize {
    3 * sentence + 40
}

/// The quotes a model may put around its whole answer.
const QUOTES: &[(char, char)] = &[
    ('"', '"'),
    ('“', '”'),
    ('\'', '\''),
    ('‘', '’'),
    ('«', '»'),
    ('`', '`'),
];

/// The rewrite of `sentence` in `body`, an OpenAI-compatible chat completion:
/// `{"choices": [{"message": {"content": "..."}, "finish_reason": "stop"}]}`.
pub(crate) fn read(sentence: &str, body: &str) -> Result<String, ScoreError> {
    let answer: Value = serde_json::from_str(body)
        .map_err(|err| ScoreError(format!("the answer is not JSON ({err})")))?;
    let choice = &answer["choices"][0];
    if choice["finish_reason"] == "length" {
        return Err(ScoreError("the model's answer was cut short".to_owned()));
    }
    let content = choice["message"]["content"]
        .as_str()
        .ok_or_else(|| ScoreError("the answer has no message in it".to_owned()))?;
    cleaned(sentence, content)
}

/// `content` as a rewrite of `sentence`: what wraps it taken off, or why it is not one.
pub(crate) fn cleaned(sentence: &str, content: &str) -> Result<String, ScoreError> {
    let mut answer = content.trim();
    if let Some(thought) = answer.strip_prefix("<think>")
        && let Some(end) = thought.find("</think>")
    {
        answer = thought[end + "</think>".len()..].trim();
    }
    if !sentence.starts_with("```")
        && let Some(fenced) = answer
            .strip_prefix("```")
            .and_then(|inner| inner.strip_suffix("```"))
    {
        // A fence's first line may name a language: "```text".
        answer = fenced
            .split_once('\n')
            .map_or(fenced, |(_, body)| body)
            .trim();
    }
    for &(open, close) in QUOTES {
        let quoted = answer.len() >= open.len_utf8() + close.len_utf8()
            && answer.starts_with(open)
            && answer.ends_with(close);
        if !quoted || sentence.starts_with(open) {
            continue;
        }
        // Only quotes around the whole answer: inside them, the sentence's own quotes and no more.
        // "`foo` is `bar`" is a model's code formatting, not `foo` is `bar` quoted.
        let inner = &answer[open.len_utf8()..answer.len() - close.len_utf8()];
        let marks = |text: &str| text.chars().filter(|&c| c == open || c == close).count();
        if marks(inner) == marks(sentence) {
            answer = inner.trim();
        }
        break;
    }
    if answer.is_empty() {
        return Err(ScoreError("the model answered nothing".to_owned()));
    }
    let (asked, got) = (sentence.chars().count(), answer.chars().count());
    if got > longest(asked) {
        return Err(ScoreError(format!(
            "the model answered {got} characters for a sentence of {asked}, which is not a rewrite of it"
        )));
    }
    if answer.matches('\n').count() > sentence.matches('\n').count() {
        return Err(ScoreError(
            "the model answered more lines than the sentence has, which is not a rewrite of it"
                .to_owned(),
        ));
    }
    Ok(answer.to_owned())
}
