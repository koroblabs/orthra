//! A host that is data: a tree-sitter grammar, and a `spell.scm` query that says where its prose is.
//!
//! One implementation runs every such host. A capture's name is the kind of span it makes:
//!
//! - `@comment`, `@doc`, `@string`, `@prose` (the text of markup) and `@declaration` make a span of
//!   that kind, and `@value` a [`SpanKind::Value`]: a value that is code, such as a CI step's shell
//!   command;
//! - `@skip` makes none, and nothing inside it does either: an import's module path is a string,
//!   but it is not prose;
//! - a name starting with `_` is a helper for a predicate, and makes nothing.
//!
//! Any other name is an error in the query. A span is its node without the whitespace at its ends,
//! and a declaration's starts at its first letter or digit.
//! When captures nest, the outer one wins, so the spans are sorted and never overlap. When two
//! capture the same range, a skip wins, then a value, then whichever pattern comes first.
//!
//! A document kept between checks is queried again only where it changed (`capture_reusing`).
//!
//! The grammar and the query load on the first use of the host, once per process. A failure to
//! load is that host's alone: it yields no spans, says why through [`Host::health`], and never
//! panics, so one broken grammar cannot take the other hosts down with it (`CLAUDE.md` §8.1).

use std::{cmp::Reverse, fmt, sync::OnceLock};

use orthra_text::{TextRange, TextSize};
use rustc_hash::FxHashMap;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor, StreamingIterator, Tree};
use tree_sitter_language::LanguageFn;

use crate::{
    Conventions, Host, Span, SpanKind,
    parsed::{Kept, Parsed},
};

mod doc;
#[cfg(test)]
mod tests;

pub(crate) use doc::DocTags;

/// Everything a tree-sitter host is, apart from the code that runs it.
pub(crate) struct Grammar {
    pub(crate) id: &'static str,
    pub(crate) language_ids: &'static [&'static str],
    pub(crate) file_names: &'static [&'static str],
    pub(crate) extensions: &'static [&'static str],
    pub(crate) conventions: Conventions,
    pub(crate) language: LanguageSource,
    /// The host's `spell.scm`.
    pub(crate) query: &'static str,
    /// The host's `vocabulary.txt`: see [`Host::vocabulary`].
    pub(crate) vocabulary: &'static str,
    /// See [`Host::test_files`].
    pub(crate) test_files: &'static [&'static str],
    /// See [`Host::generated_files`].
    pub(crate) generated_files: &'static [&'static str],
    /// What follows a tag in a doc comment: see [`DocTags`].
    pub(crate) doc_tags: DocTags,
    /// Whether a string of one token is a value rather than text ([`SpanKind::Value`]). True for
    /// a data format, whose strings are names, versions and paths, and whose text has spaces.
    pub(crate) one_token_strings_are_values: bool,
}

/// Where a grammar's language comes from. Grammars publish a `LANGUAGE` constant now; the ones
/// last released against tree-sitter 0.21 or 0.22 publish a `language()` function instead (SCSS's,
/// LESS's), which builds the same language from the same C symbol. Both are read when the host loads.
#[derive(Clone, Copy)]
pub(crate) enum LanguageSource {
    Constant(LanguageFn),
    Function(fn() -> Language),
}

impl LanguageSource {
    fn language(self) -> Language {
        match self {
            LanguageSource::Constant(constant) => Language::new(constant),
            LanguageSource::Function(function) => function(),
        }
    }
}

/// A host run from its [`Grammar`]. Each one is a `static`, so its grammar and query load once per
/// process however many engines share it.
pub(crate) struct TreeSitterHost {
    grammar: Grammar,
    /// Delimiters of what a span's text holds that is not the host's language: a template's
    /// `{{ … }}` and `{% … %}` in a page. See [`TreeSitterHost::cutting`].
    cut: &'static [(&'static str, &'static str)],
    loaded: OnceLock<Result<Loaded, String>>,
}

impl TreeSitterHost {
    pub(crate) const fn new(grammar: Grammar) -> TreeSitterHost {
        TreeSitterHost::cutting(grammar, &[])
    }

    /// A host that cuts each region between a pair of `delimiters` out of its prose, strings and
    /// comments. A query cannot cut inside a node, and a template language lives inside the text
    /// nodes of the markup it is written in: `{% endblock %}` read as a word was most of what
    /// Django's and Flask's templates reported (2026-11-08). An opening delimiter with no closing
    /// one in the span cuts the rest of it.
    pub(crate) const fn cutting(
        grammar: Grammar,
        delimiters: &'static [(&'static str, &'static str)],
    ) -> TreeSitterHost {
        TreeSitterHost {
            grammar,
            cut: delimiters,
            loaded: OnceLock::new(),
        }
    }

    fn loaded(&self) -> Result<&Loaded, &str> {
        self.loaded
            .get_or_init(|| Loaded::new(&self.grammar))
            .as_ref()
            .map_err(String::as_str)
    }
}

impl TreeSitterHost {
    /// Cuts the regions between [`TreeSitterHost::cut`]'s delimiters out of the prose, string and
    /// comment spans from `from` on, keeping each piece around them that is not only whitespace.
    fn cut_out(&self, text: &str, from: usize, out: &mut Vec<Span>) {
        if self.cut.is_empty() {
            return;
        }
        let swept = out.split_off(from);
        for span in swept {
            if !matches!(
                span.kind,
                SpanKind::Prose | SpanKind::String | SpanKind::Comment
            ) {
                out.push(span);
                continue;
            }
            let (start, end) = (
                usize::from(span.range.start()),
                usize::from(span.range.end()),
            );
            let mut at = start;
            let mut piece = start;
            while at < end {
                let rest = &text[at..end];
                let Some((offset, close)) = self
                    .cut
                    .iter()
                    .filter_map(|(open, close)| rest.find(open).map(|i| (i, (open.len(), *close))))
                    .min_by_key(|(i, _)| *i)
                else {
                    break;
                };
                let open_at = at + offset;
                let (open_len, close) = close;
                let resume = text[open_at + open_len..end]
                    .find(close)
                    .map_or(end, |i| open_at + open_len + i + close.len());
                Self::keep(text, piece..open_at, span.kind, out);
                piece = resume;
                at = resume;
            }
            Self::keep(text, piece..end, span.kind, out);
        }
    }

    /// `range` as a span of `kind`, without the whitespace at its ends, unless nothing is left.
    fn keep(text: &str, range: std::ops::Range<usize>, kind: SpanKind, out: &mut Vec<Span>) {
        if let Some(range) = Loaded::trimmed(text, range) {
            out.push(Span { range, kind });
        }
    }
}

impl fmt::Debug for TreeSitterHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TreeSitterHost")
            .field("id", &self.grammar.id)
            .finish_non_exhaustive()
    }
}

impl Host for &'static TreeSitterHost {
    fn id(&self) -> &'static str {
        self.grammar.id
    }

    fn language_ids(&self) -> &'static [&'static str] {
        self.grammar.language_ids
    }

    fn file_names(&self) -> &'static [&'static str] {
        self.grammar.file_names
    }

    fn extensions(&self) -> &'static [&'static str] {
        self.grammar.extensions
    }

    fn conventions(&self) -> Conventions {
        self.grammar.conventions
    }

    fn spans(&self, text: &str, out: &mut Vec<Span>) {
        if let Ok(loaded) = self.loaded()
            && let Some(tree) = loaded.parse(text, None)
        {
            let mut found = Vec::new();
            loaded.capture(
                text,
                tree.root_node(),
                None,
                &mut QueryCursor::new(),
                &mut found,
            );
            let from = out.len();
            loaded.sweep(text, found, out);
            self.cut_out(text, from, out);
        }
    }

    fn spans_reusing(&self, text: &str, parsed: &mut Parsed, out: &mut Vec<Span>) {
        // The old tree stays alive until the new one is parsed and queried: a node of the new
        // tree with an old node's id is then that same node, shared, and never a stranger at a
        // freed address.
        let old = parsed.take(self.grammar.id, text.len());
        let Ok(loaded) = self.loaded() else {
            return;
        };
        let Some(tree) = loaded.parse(text, old.as_ref().map(|kept| &kept.tree)) else {
            return;
        };
        let (found, captured) =
            loaded.capture_reusing(text, &tree, old.as_ref().map(|kept| &kept.captured));
        let from = out.len();
        loaded.sweep(text, found, out);
        self.cut_out(text, from, out);
        parsed.kept = Some(Kept {
            host: self.grammar.id,
            tree,
            len: text.len(),
            captured,
        });
    }

    fn health(&self) -> Result<(), String> {
        self.loaded().map(|_| ()).map_err(str::to_owned)
    }

    fn vocabulary(&self) -> &'static str {
        self.grammar.vocabulary
    }

    fn test_files(&self) -> &'static [&'static str] {
        self.grammar.test_files
    }

    fn generated_files(&self) -> &'static [&'static str] {
        self.grammar.generated_files
    }
}

/// One capture of the query, as the sweep reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Captured {
    range: TextRange,
    what: Found,
    /// The pattern that captured it: of two at the same range, the earlier pattern's wins.
    pattern: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Found {
    Span(SpanKind),
    /// Claims its range for nothing.
    Skip,
    Test,
}

/// The largest node a kept document is queried in whole; anything larger is opened, and its
/// children queried in whole or opened in turn. A sixteenth of the document, so a keystroke
/// queries about that much, and never less than this, where opening costs more than it saves.
///
/// Every test fixture is smaller than a release build's, so a test build opens nearly every node:
/// the answer does not depend on where the tree is cut, and the tests hold every fixture's spans,
/// after every edit, to a query of the whole tree.
const SMALLEST_OPENED: usize = if cfg!(debug_assertions) { 16 } else { 4096 };

/// What a capture's name asks for.
#[derive(Debug, Clone, Copy)]
enum Capture {
    Span(SpanKind),
    Skip,
    /// A region of test code: the string literals inside it are test data, not text, and are
    /// left out. `#[cfg(test)] mod tests` and `#[test] fn` in Rust.
    Test,
    Helper,
}

/// A grammar that loaded, and its compiled query.
struct Loaded {
    language: Language,
    query: Query,
    /// By capture index.
    captures: Vec<Capture>,
    doc_tags: DocTags,
    one_token_strings_are_values: bool,
}

impl Loaded {
    fn new(grammar: &Grammar) -> Result<Loaded, String> {
        let language = grammar.language.language();
        // A grammar built for another ABI fails here, when the language is set, and not at compile
        // time. Setting it once now makes that this host's load error rather than a silent parse.
        Parser::new()
            .set_language(&language)
            .map_err(|err| format!("grammar: {err}"))?;
        let query =
            Query::new(&language, grammar.query).map_err(|err| format!("spell.scm: {err}"))?;
        let captures = query
            .capture_names()
            .iter()
            .map(|name| match *name {
                "comment" => Ok(Capture::Span(SpanKind::Comment)),
                "doc" => Ok(Capture::Span(SpanKind::DocComment)),
                "string" => Ok(Capture::Span(SpanKind::String)),
                "prose" => Ok(Capture::Span(SpanKind::Prose)),
                "declaration" => Ok(Capture::Span(SpanKind::Declaration)),
                "value" => Ok(Capture::Span(SpanKind::Value)),
                "skip" => Ok(Capture::Skip),
                "test" => Ok(Capture::Test),
                helper if helper.starts_with('_') => Ok(Capture::Helper),
                unknown => Err(format!("spell.scm: unknown capture @{unknown}")),
            })
            .collect::<Result<_, _>>()?;
        Ok(Loaded {
            language,
            query,
            captures,
            doc_tags: grammar.doc_tags,
            one_token_strings_are_values: grammar.one_token_strings_are_values,
        })
    }

    /// `text`'s tree. With `old`, the tree of the text before the edits it has been told, the
    /// parts the edits did not touch are reused rather than parsed again.
    fn parse(&self, text: &str, old: Option<&Tree>) -> Option<Tree> {
        let mut parser = Parser::new();
        parser.set_language(&self.language).ok()?;
        parser.parse(text, old)
    }

    /// What the query captures in `node`, by patterns that start no deeper below it than
    /// `max_start_depth`, or at any depth for `None`.
    fn capture(
        &self,
        text: &str,
        node: Node<'_>,
        max_start_depth: Option<u32>,
        cursor: &mut QueryCursor,
        out: &mut Vec<Captured>,
    ) {
        cursor.set_max_start_depth(max_start_depth);
        let mut matches = cursor.matches(&self.query, node, text.as_bytes());
        while let Some(found_match) = matches.next() {
            for capture in found_match.captures() {
                let what = match self.captures[capture.index as usize] {
                    Capture::Span(kind) => Found::Span(kind),
                    Capture::Skip => Found::Skip,
                    Capture::Test => Found::Test,
                    Capture::Helper => continue,
                };
                let range = match what {
                    Found::Span(SpanKind::Declaration) => {
                        Self::name(text, capture.node.byte_range())
                    }
                    _ => Self::trimmed(text, capture.node.byte_range()),
                };
                if let Some(range) = range {
                    out.push(Captured {
                        range,
                        what,
                        pattern: found_match.pattern_index as u32,
                    });
                }
            }
        }
    }

    /// Everything the query captures in `tree`, a kept document's, with what the last check
    /// captured in each node it queried whole (`old`, by node), and the same for the next one.
    ///
    /// The tree is cut into nodes queried whole and nodes opened: a node larger than the
    /// smallest one opened ([`SMALLEST_OPENED`], or a sixteenth of the text) is opened, and each
    /// of its children is queried whole or opened in turn. An opened node is asked only for the
    /// patterns that start at it or at one of its children, which covers a pattern over
    /// siblings, `#[cfg(test)]` and the `mod tests` after it, whichever of them is opened. Every
    /// other match starts inside exactly one node queried whole, and stays inside it. A node
    /// queried whole whose id the old tree had is the same node, shared by the new parse because
    /// the edits did not reach it, so what it captured is what it captured last time.
    fn capture_reusing(
        &self,
        text: &str,
        tree: &Tree,
        old: Option<&FxHashMap<usize, Box<[Captured]>>>,
    ) -> (Vec<Captured>, FxHashMap<usize, Box<[Captured]>>) {
        let smallest = (text.len() / 16).next_power_of_two().max(SMALLEST_OPENED);
        let mut cursor = QueryCursor::new();
        let mut found = Vec::new();
        let mut captured = FxHashMap::default();
        let mut own = Vec::new();
        let mut open = vec![tree.root_node()];
        while let Some(node) = open.pop() {
            if node.byte_range().len() > smallest && node.child_count() > 0 {
                self.capture(text, node, Some(1), &mut cursor, &mut found);
                let mut walk = node.walk();
                open.extend(node.children(&mut walk));
                continue;
            }
            let Ok(base) = TextSize::try_from(node.start_byte()) else {
                continue;
            };
            let kept = match old.and_then(|old| old.get(&node.id())) {
                Some(kept) => kept.clone(),
                None => {
                    own.clear();
                    self.capture(text, node, None, &mut cursor, &mut own);
                    own.iter()
                        .map(|capture| Captured {
                            range: capture.range - base,
                            ..*capture
                        })
                        .collect()
                }
            };
            found.extend(kept.iter().map(|capture| Captured {
                range: capture.range + base,
                ..*capture
            }));
            captured.insert(node.id(), kept);
        }
        (found, captured)
    }

    /// The spans the captures make: the outer capture where they nest, the earliest pattern's
    /// where two capture one range, and no string of test code.
    fn sweep(&self, text: &str, mut found: Vec<Captured>, out: &mut Vec<Span>) {
        let tests: Vec<TextRange> = found
            .iter()
            .filter(|capture| capture.what == Found::Test)
            .map(|capture| capture.range)
            .collect();
        found.retain(|capture| capture.what != Found::Test);
        // Outer before inner at the same start, so the sweep keeps the outer capture and drops
        // whatever it holds. At the same range a skip comes first, then a value, which is code, then
        // the rest by pattern: `run: cargo test` is one node captured both as a value and as a
        // string. A match found twice, by an opened node and inside its child, sorts next to
        // itself and is dropped as covered.
        found.sort_by_key(|capture| {
            let rank = match capture.what {
                Found::Skip => 0,
                Found::Span(SpanKind::Value) => 1,
                Found::Span(_) | Found::Test => 2,
            };
            (
                capture.range.start(),
                Reverse(capture.range.end()),
                rank,
                capture.pattern,
            )
        });
        let found = found.into_iter().map(|capture| {
            let kind = match capture.what {
                Found::Span(kind) => Some(kind),
                Found::Skip | Found::Test => None,
            };
            (capture.range, kind)
        });
        let mut covered = TextSize::from(0);
        let mut in_fence = false;
        for (range, kind) in found {
            if range.start() < covered {
                continue;
            }
            covered = range.end();
            match kind {
                None => {}
                Some(SpanKind::String) if tests.iter().any(|test| test.contains_range(range)) => {}
                Some(SpanKind::DocComment) => self.doc_lines(text, range, &mut in_fence, out),
                Some(SpanKind::String)
                    if self.one_token_strings_are_values && Self::one_token(&text[range]) =>
                {
                    in_fence = false;
                    out.push(Span {
                        range,
                        kind: SpanKind::Value,
                    });
                }
                Some(kind) => {
                    in_fence = false;
                    // A string the grammar split only at an apostrophe, escaped or not, is one
                    // span, so that `didn\'t` is one word: `It didn` and `t work` are not.
                    // tree-sitter-dart ends a piece of a double-quoted string at a bare `'`.
                    if let Some(last) = out.last_mut()
                        && kind == SpanKind::String
                        && last.kind == SpanKind::String
                        && matches!(
                            &text[last.range.end().into()..range.start().into()],
                            "\\'" | "\\’" | "'" | "’"
                        )
                    {
                        last.range = TextRange::new(last.range.start(), range.end());
                        continue;
                    }
                    out.push(Span { range, kind });
                }
            }
        }
    }

    /// Whether a string is one token: no space in it, written or escaped. `"Hello\nworld"` is two.
    fn one_token(string: &str) -> bool {
        !string.contains(char::is_whitespace)
            && !["\\n", "\\t", "\\r"]
                .iter()
                .any(|escape| string.contains(escape))
    }

    /// A declared name's range from its first letter or digit, or `None` if it has none: CSS's
    /// `--brand-color` declares `brand-color`. A name is never a flag, as a chunk that starts with
    /// `-` otherwise reads, and `--recieve-color` went unreported.
    fn name(text: &str, range: std::ops::Range<usize>) -> Option<TextRange> {
        let lead = text.get(range.clone())?.find(char::is_alphanumeric)?;
        Self::trimmed(text, range.start + lead..range.end)
    }

    /// A node's range without the whitespace at its ends, or `None` if nothing is left. A line
    /// comment's node holds its line break, and JSX text the indentation around it.
    fn trimmed(text: &str, range: std::ops::Range<usize>) -> Option<TextRange> {
        let piece = text.get(range.clone())?;
        let start = range.start + (piece.len() - piece.trim_start().len());
        let end = range.end - (piece.len() - piece.trim_end().len());
        if start >= end {
            return None;
        }
        Some(TextRange::new(
            TextSize::try_from(start).ok()?,
            TextSize::try_from(end).ok()?,
        ))
    }
}
