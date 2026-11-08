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
    /// Tags whose section is code, up to the next tag: roxygen's `@examples`. The lines after one
    /// are not prose, and the next line that starts with a tag of any other name is.
    pub(crate) code: &'static [&'static str],
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
        code: &[],
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
        let mut in_pre = false;
        let mut at = usize::from(range.start());
        for line in text[range].split_inclusive('\n') {
            let line_start = at;
            at += line.len();
            // A line comment's marker is `//`, `#` or `--` (Ruby's YARD tags follow `#`).
            let marked = line
                .trim_start()
                .trim_start_matches(['/', '*', '!', '-', '#', '\'']);
            let content = marked.trim_start();
            // An indented code block: four spaces past the one that follows the marker. Markdown
            // wants a blank line before it, which is not carried here: a list item's continuation
            // indented that far goes unchecked, and none is misread as a word.
            if scan.depth == 0 && marked.starts_with("     ") && !content.is_empty() {
                continue;
            }
            if scan.depth == 0 && content.starts_with("```") {
                *in_fence = !*in_fence;
                continue;
            }
            // Doxygen's code blocks: `\code` or `@code` to `\endcode`, and `\verbatim`.
            if scan.depth == 0 && Self::command(content, &["code", "verbatim"]) {
                *in_fence = true;
                continue;
            }
            if scan.depth == 0 && Self::command(content, &["endcode", "endverbatim"]) {
                *in_fence = false;
                continue;
            }
            let tag = content
                .strip_prefix('@')
                .filter(|tag| tag.starts_with(char::is_alphabetic));
            if scan.depth == 0 && !self.doc_tags.code.is_empty() && tag.is_some() {
                *in_fence = Self::command(content, self.doc_tags.code);
                if *in_fence {
                    continue;
                }
            }
            if *in_fence {
                continue;
            }
            let Some(prose) = self.prose_start(line, line.len() - content.len(), &mut scan) else {
                continue;
            };
            for piece in Self::outside_markup(&line[prose..], &mut in_pre) {
                let start = line_start + prose;
                if let Some(piece) = Self::trimmed(text, start + piece.start..start + piece.end) {
                    out.push(Span {
                        range: piece,
                        kind: SpanKind::DocComment,
                    });
                }
            }
        }
    }

    /// Whether a doc line's `content` is one of Doxygen's `commands`, written `\name` or `@name`
    /// and ending there or at a space.
    fn command(content: &str, commands: &[&str]) -> bool {
        let Some(rest) = content
            .strip_prefix('\\')
            .or_else(|| content.strip_prefix('@'))
        else {
            return false;
        };
        commands.iter().any(|name| {
            rest.strip_prefix(name)
                .is_some_and(|after| after.is_empty() || after.starts_with(char::is_whitespace))
        })
    }

    /// The parts of one doc line that are prose: not an HTML `<pre>` block, which can run over
    /// several lines and is carried in `in_pre`; not `<code>...</code>`; and not an inline tag,
    /// `{@code max}` or `{@link List#of}`, whose content is code or a reference. Javadoc's
    /// examples are written so, and `abcxyz` in `StringUtils.replaceChars("abcba", ...)` is not a
    /// word of the doc. An opening with no end on its line is left as prose.
    fn outside_markup(line: &str, in_pre: &mut bool) -> Vec<std::ops::Range<usize>> {
        let bytes = line.as_bytes();
        let starts = |at: usize, tag: &str| {
            bytes
                .get(at..at + tag.len())
                .is_some_and(|head| head.eq_ignore_ascii_case(tag.as_bytes()))
        };
        let mut pieces = Vec::new();
        let (mut from, mut at) = (0, 0);
        while at < bytes.len() {
            if *in_pre {
                if starts(at, "</pre>") {
                    *in_pre = false;
                    at += "</pre>".len();
                    from = at;
                } else {
                    at += 1;
                }
                continue;
            }
            if starts(at, "<pre>") || starts(at, "<pre ") {
                pieces.push(from..at);
                *in_pre = true;
                at += "<pre".len();
                continue;
            }
            let skipped = if starts(at, "<code>") {
                line[at..]
                    .to_ascii_lowercase()
                    .find("</code>")
                    .map(|end| end + "</code>".len())
            } else if starts(at, "{@") {
                Self::braced(&line[at..])
            } else {
                None
            };
            match skipped {
                Some(len) => {
                    pieces.push(from..at);
                    at += len;
                    from = at;
                }
                None => at += 1,
            }
        }
        if !*in_pre {
            pieces.push(from..bytes.len());
        }
        pieces
    }

    /// The length of the `{...}` that `text` starts with, braces nested, if it closes.
    fn braced(text: &str) -> Option<usize> {
        let mut depth = 0u32;
        for (at, byte) in text.bytes().enumerate() {
            match byte {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(at + 1);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Where the prose of a doc comment's `line` starts, after its tag and the tag's type; `None`
    /// when the line has none.
    fn prose_start(&self, line: &str, content: usize, scan: &mut TypeScan) -> Option<usize> {
        if scan.depth > 0 {
            let end = scan.token(line, content)?;
            return Some(scan.name(line, end));
        }
        // A tag is `@param`, or Doxygen's `\param`: a backslash before a letter.
        let rest = &line[content..];
        let Some(tag) = rest.strip_prefix('@').or_else(|| {
            rest.strip_prefix('\\')
                .filter(|tag| tag.starts_with(|c: char| c.is_ascii_alphabetic()))
        }) else {
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
