//! The passage a sentence is looked for in, when somebody asks for one to be rewritten: the text
//! meant to be read around the cursor, as the host's spans give it (`docs/04-context.md` §2 C).
//! The sentence inside it is `orthra_context::sentence`'s to find.

use orthra_hosts::{Span, SpanKind};
use orthra_text::{TextRange, TextSize};

#[cfg(test)]
mod tests;

/// The passage around `at`, or `None` where there is no text meant to be read:
/// - in prose (Markdown, plain text, a commit message), the spans one paragraph's text makes,
///   through its inline code and markup and across a line that wraps, from the start of its first
///   line to the end of its last, so that a bullet or a link's bracket is where the sentence can
///   tell it from a word;
/// - in a comment or a doc comment, the comment on the cursor's line: the next line starts with
///   the comment's marker, which a model is not handed to rewrite;
/// - nowhere else: code, a string, a name or a value is not a sentence.
pub(crate) fn around(text: &str, spans: &[Span], at: TextSize) -> Option<TextRange> {
    let holds = |span: &Span, kinds: &[SpanKind]| {
        span.range.contains_inclusive(at) && kinds.contains(&span.kind)
    };
    let in_prose = [SpanKind::Prose, SpanKind::Value];
    if let Some(i) = spans.iter().position(|span| holds(span, &in_prose)) {
        let mut first = i;
        while first > 0 && joins(text, &spans[first - 1], &spans[first], &in_prose) {
            first -= 1;
        }
        let mut last = i;
        while last + 1 < spans.len() && joins(text, &spans[last], &spans[last + 1], &in_prose) {
            last += 1;
        }
        // Inline code alone, a TOML value say, is not prose.
        if spans[first..=last]
            .iter()
            .any(|span| span.kind == SpanKind::Prose)
        {
            let (start, end) = (spans[first].range.start(), spans[last].range.end());
            return Some(TextRange::new(line_start(text, start), line_end(text, end)));
        }
    }
    let comments = [SpanKind::Comment, SpanKind::DocComment];
    let comment = spans.iter().find(|span| holds(span, &comments))?;
    Some(TextRange::new(
        comment.range.start().max(line_start(text, at)),
        comment.range.end().min(line_end(text, at)),
    ))
}

/// Whether `before` and `after` are one passage's: both of `kinds`, and between them markup on
/// the same line, or a line break and nothing else. A blank line, a list's next bullet or a code
/// fence ends a passage.
fn joins(text: &str, before: &Span, after: &Span, kinds: &[SpanKind]) -> bool {
    if !kinds.contains(&before.kind)
        || !kinds.contains(&after.kind)
        || after.range.start() < before.range.end()
    {
        return false;
    }
    let gap = &text[TextRange::new(before.range.end(), after.range.start())];
    !gap.contains('\n') || (gap.trim().is_empty() && gap.matches('\n').count() == 1)
}

fn line_start(text: &str, at: TextSize) -> TextSize {
    let at = usize::from(at);
    size(text[..at].rfind('\n').map_or(0, |line| line + 1))
}

fn line_end(text: &str, at: TextSize) -> TextSize {
    let at = usize::from(at);
    size(text[at..].find('\n').map_or(text.len(), |line| at + line))
}

fn size(at: usize) -> TextSize {
    TextSize::try_from(at).expect("a document under 4 GiB")
}
