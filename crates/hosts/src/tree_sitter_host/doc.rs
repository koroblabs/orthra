//! The prose of a doc comment: its lines, without fenced examples, tags, and the types tags carry.

use orthra_text::TextRange;

use super::Loaded;
use crate::{Span, SpanKind};

/// The tags at the start of a doc comment's line, and what follows each. Data, per host: PHPDoc's
/// `@param` is followed by a type, its `@author` by a name. Any other tag is skipped with its
/// arguments, `@ORM\Column(type="string")`, and what follows it is read.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DocTags {
    /// Prefixes a tool puts on a tag it shares with the language: `@phpstan-param` is `@param`.
    pub(crate) prefixes: &'static [&'static str],
    /// Tags followed by a type, which is code, and then perhaps a `$name`:
    /// `@param array{id: int, name: string} $user The user to greet.` A type's brackets can run
    /// over several lines, and every line they hold is code.
    pub(crate) typed: &'static [&'static str],
    /// Tags whose line is not prose: a name, a version, a reference. `@author`, `@since`, `@see`.
    pub(crate) unread: &'static [&'static str],
}

impl DocTags {
    pub(crate) const NONE: DocTags = DocTags {
        prefixes: &[],
        typed: &[],
        unread: &[],
    };
}

/// Where a type that a tag carries has got to, from one line of a doc comment to the next.
#[derive(Default)]
struct TypeScan {
    /// Brackets still open: `array{` at the end of a line.
    depth: u32,
    /// Whether a `$name` may follow the type once it closes.
    named: bool,
}

impl Loaded {
    /// A doc comment's lines, without its fenced code and its tags' arguments. Rustdoc, JSDoc and
    /// TSDoc are Markdown, and the example between ```` ``` ```` fences is code: `"Watsons"` in a
    /// doc example is not a word of the doc. A fence can open in one `///` line and close several
    /// lines later, so whether it is open is carried from one doc comment to the next, and any
    /// other span closes it.
    pub(super) fn doc_lines(
        &self,
        text: &str,
        range: TextRange,
        in_fence: &mut bool,
        out: &mut Vec<Span>,
    ) {
        let mut scan = TypeScan::default();
        let mut at = usize::from(range.start());
        for line in text[range].split_inclusive('\n') {
            let line_start = at;
            at += line.len();
            let content = line
                .trim_start()
                .trim_start_matches(['/', '*', '!', '-'])
                .trim_start();
            if scan.depth == 0 && content.starts_with("```") {
                *in_fence = !*in_fence;
                continue;
            }
            if *in_fence {
                continue;
            }
            let Some(prose) = self.prose_start(line, line.len() - content.len(), &mut scan) else {
                continue;
            };
            if let Some(piece) = Self::trimmed(text, line_start + prose..at) {
                out.push(Span {
                    range: piece,
                    kind: SpanKind::DocComment,
                });
            }
        }
    }

    /// Where the prose of a doc comment's `line` starts, after its tag and the tag's type; `None`
    /// when the line has none.
    fn prose_start(&self, line: &str, content: usize, scan: &mut TypeScan) -> Option<usize> {
        if scan.depth > 0 {
            let end = scan.token(line, content)?;
            return Some(scan.name(line, end));
        }
        let Some(tag) = line[content..].strip_prefix('@') else {
            return Some(content);
        };
        let name_len = tag
            .find(|c: char| !(c.is_alphanumeric() || matches!(c, '-' | '_' | '\\')))
            .unwrap_or(tag.len());
        let name = &tag[..name_len];
        let tags = &self.doc_tags;
        let name = tags
            .prefixes
            .iter()
            .find_map(|prefix| name.strip_prefix(prefix))
            .unwrap_or(name);
        if tags.unread.contains(&name) {
            return None;
        }
        // The tag itself, with any arguments in brackets.
        let after_tag = scan.token(line, content)?;
        if !tags.typed.contains(&name) {
            return Some(after_tag);
        }
        scan.named = true;
        let end = scan.token(line, after_tag)?;
        Some(scan.name(line, end))
    }
}

impl TypeScan {
    /// The end of the token that starts at or after `at` in `line`. Brackets and quotes hold
    /// spaces inside one token, and a space after `:`, `|` or `,`, or before `|`, does not end it:
    /// `callable(int): void`, `int | string`. `None` when the line ends inside a bracket, and the
    /// token goes on in the next line.
    fn token(&mut self, line: &str, mut at: usize) -> Option<usize> {
        let bytes = line.as_bytes();
        if self.depth == 0 {
            while at < bytes.len() && bytes[at].is_ascii_whitespace() {
                at += 1;
            }
        }
        let mut quote = None;
        while at < bytes.len() {
            let byte = bytes[at];
            if let Some(open) = quote {
                if byte == open {
                    quote = None;
                }
                at += 1;
                continue;
            }
            match byte {
                // A quote after a letter is an apostrophe: `O'Brien`, not a string.
                b'\'' | b'"' if at == 0 || !bytes[at - 1].is_ascii_alphanumeric() => {
                    quote = Some(byte);
                }
                b'<' | b'(' | b'{' | b'[' => self.depth += 1,
                b'>' | b')' | b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                byte if byte.is_ascii_whitespace() && self.depth == 0 => {
                    let rest = line[at..].trim_start();
                    let joined = at > 0 && matches!(bytes[at - 1], b':' | b'|' | b',')
                        || rest.starts_with('|');
                    if !joined || rest.is_empty() {
                        return Some(at);
                    }
                }
                _ => {}
            }
            at += 1;
        }
        (self.depth == 0).then_some(at)
    }

    /// After a type, the `$name` it is for, if one follows: `$user`, `&$out`, `...$rest`. It is a
    /// name, not prose.
    fn name(&mut self, line: &str, at: usize) -> usize {
        if !std::mem::take(&mut self.named) {
            return at;
        }
        let rest = &line[at..];
        let start = at + (rest.len() - rest.trim_start().len());
        if line[start..]
            .trim_start_matches(['&', '.'])
            .starts_with('$')
        {
            return line[start..]
                .find(char::is_whitespace)
                .map_or(line.len(), |end| start + end);
        }
        at
    }
}
