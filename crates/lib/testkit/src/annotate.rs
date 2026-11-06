//! The annotated-text rendering: the document, with carets under what was found.
//!
//! ```text
//! > /// Retruns the frist element.
//!       ^^^^^^^ unknown-word
//!                   ^^^^^ known-typo -> first
//! > fn main() {}
//! ```
//!
//! Every line of the text is shown, prefixed `> `. Each annotation gets its own line under the line
//! it starts on. This has two consequences, and both are deliberate:
//!
//! - **A clean result is the document with no carets.** It is never an empty string, so an empty
//!   snapshot can only mean an empty document. phrost nearly shipped three accepted-empty
//!   snapshots as coverage in one session; this format makes that mistake impossible to make
//!   silently.
//! - **Every rendering has a line that starts in column 0.** expect-test infers the indentation to
//!   strip from the snapshot, and padding on every line makes it fight `cargo fmt` forever.

use orthra_text::{
    TextRange, TextSize,
    line_index::{Encoding, LineIndex},
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub range: TextRange,
    pub label: String,
}

impl Annotation {
    pub fn new(range: TextRange, label: impl Into<String>) -> Annotation {
        Annotation {
            range,
            label: label.into(),
        }
    }

    /// The inverse of [`Annotation::render`]: the text, and what is annotated in it, in render
    /// order.
    ///
    /// This is what lets an expectation be written as the snapshot it will be compared with, so a
    /// corpus case reads the same way a failing test prints. Only what a single line can say is
    /// accepted.
    ///
    /// # Panics
    ///
    /// On a line that is neither `> text`, `>` nor a caret line. On a `(+N lines)` annotation: the
    /// rendering shows where such a range starts, not where it ends, so it cannot be read back.
    /// On a caret line before any text, or with nothing but padding.
    pub fn parse(rendered: &str) -> (String, Vec<Annotation>) {
        let mut text = String::new();
        let mut annotations = Vec::new();
        let mut line_start: Option<usize> = None;
        for line in rendered.lines() {
            if let Some(content) = line.strip_prefix("> ").or((line == ">").then_some("")) {
                line_start = Some(text.len());
                text.push_str(content);
                text.push('\n');
                continue;
            }
            let Some(caret_line) = line.strip_prefix("  ") else {
                panic!("an annotated line is `> text`, `>`, or carets indented by two: {line:?}");
            };
            let start =
                line_start.unwrap_or_else(|| panic!("a caret line before any text: {line:?}"));
            let content = &text[start..text.len() - 1];
            annotations.push(Self::parse_caret_line(content, start, caret_line));
        }
        (text, annotations)
    }

    fn parse_caret_line(content: &str, line_start: usize, caret_line: &str) -> Annotation {
        let pad = caret_line
            .find('^')
            .unwrap_or_else(|| panic!("a caret line with no carets: {caret_line:?}"));
        let column = caret_line[..pad].chars().count();
        let rest = &caret_line[pad..];
        let width = rest.len() - rest.trim_start_matches('^').len();
        let mut label = rest[width..]
            .strip_prefix(' ')
            .unwrap_or(&rest[width..])
            .to_owned();
        assert!(
            !label.contains("(+") || !label.ends_with("lines)"),
            "a range across lines cannot be read back from its rendering: {caret_line:?}"
        );
        let empty = label.ends_with("(empty)");
        if empty {
            label.truncate(label.len() - "(empty)".len());
            label.truncate(label.trim_end().len());
        }

        let offset_of = |chars: usize| {
            content
                .char_indices()
                .map(|(at, _)| at)
                .chain([content.len()])
                .nth(chars)
                .unwrap_or_else(|| panic!("carets past the end of {content:?}: {caret_line:?}"))
        };
        let from = offset_of(column);
        let to = if empty {
            from
        } else {
            offset_of(column + width)
        };
        let size =
            |at: usize| TextSize::try_from(line_start + at).expect("a rendering under 4 GiB");
        Annotation::new(TextRange::new(size(from), size(to)), label)
    }

    /// Renders `text` with `annotations` under it.
    ///
    /// Annotations are ordered by start, then end, then label, so the input's order never shows in
    /// a snapshot. Caret columns count characters, and a tab under a tab stays a tab, so carets line
    /// up in an editor, though not always in a terminal that draws an emoji two cells wide.
    ///
    /// # Panics
    ///
    /// If an annotation reaches past the text or starts inside a character or a `\r\n`. Those are
    /// bugs in whatever produced the annotation, and a rendering that quietly moved them would
    /// hide exactly that.
    pub fn render(text: &str, annotations: &[Annotation]) -> String {
        let index = LineIndex::new(text);
        let mut by_line: Vec<Vec<&Annotation>> = vec![Vec::new(); index.line_count()];
        for annotation in annotations {
            assert!(
                annotation.range.end() <= TextSize::of(text),
                "annotation {annotation:?} reaches past the end of the text"
            );
            let start = index
                .line_col_in(text, annotation.range.start(), Encoding::Utf32)
                .unwrap_or_else(|| panic!("annotation {annotation:?} starts inside a character"));
            by_line[start.line as usize].push(annotation);
        }

        // A text ending in a terminator has an empty last line. Showing it would add a `>` the
        // document does not visibly have, unless something is annotated there.
        let last = index.line_count() - 1;
        let shown = if index.line_content(text, last as u32).unwrap().is_empty()
            && by_line[last].is_empty()
        {
            last
        } else {
            last + 1
        };

        let mut out = String::new();
        for (line, annotations) in by_line.iter_mut().enumerate().take(shown) {
            let range = index.line_content(text, line as u32).unwrap();
            let content = &text[range];
            if content.is_empty() {
                out.push_str(">\n");
            } else {
                out.push_str("> ");
                out.push_str(content);
                out.push('\n');
            }
            annotations.sort_by(|a, b| {
                (a.range.start(), a.range.end(), &a.label).cmp(&(
                    b.range.start(),
                    b.range.end(),
                    &b.label,
                ))
            });
            for annotation in annotations.iter() {
                Self::caret_line(text, &index, range, annotation, &mut out);
            }
        }
        out
    }

    fn caret_line(
        text: &str,
        index: &LineIndex,
        line: TextRange,
        annotation: &Annotation,
        out: &mut String,
    ) {
        out.push_str("  ");
        for c in text[TextRange::new(line.start(), annotation.range.start())].chars() {
            out.push(if c == '\t' { '\t' } else { ' ' });
        }
        let on_this_line = TextRange::new(
            annotation.range.start(),
            annotation.range.end().min(line.end()),
        );
        let width = text[on_this_line].chars().count().max(1);
        out.extend(std::iter::repeat_n('^', width));

        let mut notes = Vec::new();
        if annotation.range.is_empty() {
            notes.push("(empty)".to_owned());
        }
        if annotation.range.end() > line.end() {
            let end = index.line_col_in(text, annotation.range.end(), Encoding::Utf32);
            let start = index.line_col_in(text, annotation.range.start(), Encoding::Utf32);
            if let (Some(end), Some(start)) = (end, start) {
                notes.push(format!("(+{} lines)", end.line - start.line));
            }
        }
        for part in std::iter::once(annotation.label.as_str())
            .chain(notes.iter().map(String::as_str))
            .filter(|part| !part.is_empty())
        {
            out.push(' ');
            out.push_str(part);
        }
        out.push('\n');
    }
}
