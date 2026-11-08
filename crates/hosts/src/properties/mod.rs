//! Java properties: comments, keys, and the text of values.
//!
//! Read by hand, with no tree-sitter grammar. `tree-sitter-properties` 0.3.0's external scanner
//! keeps its end-of-file state in a process-wide `static bool reached_eof`, so two parses on two
//! threads corrupt each other's state: `orthra check` on 238 properties files hung in five runs
//! out of six, at 1.4 GB after six seconds and 46 GB when left running (2026-11-09). The format is
//! one line per entry, which is what this reads.
//!
//! A line is blank, a comment (`#` or `!` first), or `key = value`, where `=`, `:` or white space
//! ends the key, and a value goes on past a line break that a backslash escapes.

use orthra_text::{TextRange, TextSize};

use crate::{Conventions, Host, Span, SpanKind};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct Properties;

impl Host for Properties {
    fn id(&self) -> &'static str {
        "properties"
    }

    fn language_ids(&self) -> &'static [&'static str] {
        &["properties", "java-properties"]
    }

    fn file_names(&self) -> &'static [&'static str] {
        &[]
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["properties"]
    }

    /// `\n`, `\t` and `é` in a value are escapes. Both comment markers start a comment, and
    /// `#` is the one the shift dimension writes.
    fn conventions(&self) -> Conventions {
        Conventions {
            backslash_escapes: true,
            doubled_escapes: false,
            line_comment: Some("#"),
            block_comment: None,
        }
    }

    fn vocabulary(&self) -> &'static str {
        include_str!("vocabulary.txt")
    }

    fn spans(&self, text: &str, out: &mut Vec<Span>) {
        let mut lines = Lines { text, at: 0 };
        while let Some(line) = lines.next_line() {
            let first = line.start + line.trim_start_len(text);
            match text.as_bytes().get(first) {
                None => {}
                Some(b'#' | b'!') => Self::push(text, first, line.end, SpanKind::Comment, out),
                Some(_) => Self::entry(text, first, line, &mut lines, out),
            }
        }
    }
}

/// One physical line: its text without the line break.
#[derive(Clone, Copy)]
struct Line {
    start: usize,
    end: usize,
}

impl Line {
    fn trim_start_len(self, text: &str) -> usize {
        text[self.start..self.end]
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t' | 0x0c))
            .count()
    }

    /// Whether a backslash escapes the line break: an odd run of them ends the line.
    fn continues(self, text: &str) -> bool {
        text[self.start..self.end]
            .bytes()
            .rev()
            .take_while(|b| *b == b'\\')
            .count()
            % 2
            == 1
    }
}

struct Lines<'a> {
    text: &'a str,
    at: usize,
}

impl Lines<'_> {
    fn next_line(&mut self) -> Option<Line> {
        if self.at >= self.text.len() {
            return None;
        }
        let start = self.at;
        let rest = &self.text[start..];
        let (end, next) = match rest.find('\n') {
            Some(i) => (start + i, start + i + 1),
            None => (self.text.len(), self.text.len()),
        };
        self.at = next;
        let end = if self.text[start..end].ends_with('\r') {
            end - 1
        } else {
            end
        };
        Some(Line { start, end })
    }
}

impl Properties {
    /// `key = value`, from the key's first character on its first line.
    fn entry(text: &str, first: usize, line: Line, lines: &mut Lines, out: &mut Vec<Span>) {
        let bytes = text.as_bytes();
        let mut i = first;
        while i < line.end {
            match bytes[i] {
                b'\\' => i += 2,
                b'=' | b':' | b' ' | b'\t' | 0x0c => break,
                _ => i += 1,
            }
        }
        let key_end = i.min(line.end);
        Self::key(text, first, key_end, out);
        while i < line.end && matches!(bytes[i], b' ' | b'\t' | 0x0c) {
            i += 1;
        }
        if i < line.end && matches!(bytes[i], b'=' | b':') {
            i += 1;
        }
        while i < line.end && matches!(bytes[i], b' ' | b'\t' | 0x0c) {
            i += 1;
        }

        // The value's pieces, one per physical line.
        let mut pieces: Vec<(usize, usize)> = Vec::new();
        let (mut start, mut current) = (i.min(line.end), line);
        loop {
            let continues = current.continues(text);
            let end = if continues {
                current.end - 1
            } else {
                current.end
            };
            if start < end {
                pieces.push((start, end));
            }
            if !continues {
                break;
            }
            let Some(next) = lines.next_line() else {
                break;
            };
            start = next.start + next.trim_start_len(text);
            current = next;
        }

        let kind = Self::kind(text, &pieces);
        for (start, end) in pieces {
            Self::push(text, start, end, kind, out);
        }
    }

    /// A value is text where it has words: a message. One token is a name, a port or a URL, and a
    /// Spring placeholder (`${DB_URL:jdbc:h2:mem}`) makes the value a setting: both are held to
    /// known typos only.
    fn kind(text: &str, pieces: &[(usize, usize)]) -> SpanKind {
        let words = |&&(start, end): &&(usize, usize)| {
            let piece = text[start..end].trim();
            piece.contains(char::is_whitespace)
                || ["\\n", "\\t", "\\r"]
                    .iter()
                    .any(|escape| piece.contains(escape))
        };
        let joined_has_placeholder = pieces.iter().any(|&(s, e)| text[s..e].contains("${"));
        if joined_has_placeholder || (pieces.len() <= 1 && !pieces.iter().any(|p| words(&p))) {
            SpanKind::Value
        } else {
            SpanKind::String
        }
    }

    /// A key is the name its code looks a message up by, or the sentence itself, as in JabRef's
    /// `Add\ group=Add group`. It is held to known typos only: as a declaration its words would be
    /// accepted everywhere, and a typo in a sentence's value would pass beside its key.
    fn key(text: &str, start: usize, end: usize, out: &mut Vec<Span>) {
        Self::push(text, start, end, SpanKind::Value, out);
    }

    fn push(text: &str, start: usize, end: usize, kind: SpanKind, out: &mut Vec<Span>) {
        let Some(piece) = text.get(start..end) else {
            return;
        };
        let lead = piece.len() - piece.trim_start().len();
        let end = start + piece.trim_end().len();
        let start = start + lead;
        if start >= end {
            return;
        }
        let (Ok(start), Ok(end)) = (TextSize::try_from(start), TextSize::try_from(end)) else {
            return;
        };
        out.push(Span {
            range: TextRange::new(start, end),
            kind,
        });
    }
}
