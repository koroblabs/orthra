//! Markdown: the text of paragraphs, headings, lists, tables and link labels is prose. Inline code
//! is code, held to known typos as it is between backticks in a comment. Fenced and indented code,
//! HTML, front matter and link destinations are not read.
//!
//! The parser is pulldown-cmark, not tree-sitter-md: they extract the same prose, and tree-sitter-md
//! costs forty times as much (`docs/01-architecture.md` §3).

use orthra_text::{TextRange, TextSize};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::{Conventions, Host, Span, SpanKind};

#[derive(Debug)]
pub struct Markdown;

impl Host for Markdown {
    fn id(&self) -> &'static str {
        "markdown"
    }

    fn language_ids(&self) -> &'static [&'static str] {
        &["markdown"]
    }

    fn file_names(&self) -> &'static [&'static str] {
        &[]
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["md", "markdown", "mdx"]
    }

    fn conventions(&self) -> Conventions {
        Conventions {
            block_comment: Some(("<!--", "-->")),
            ..Conventions::default()
        }
    }

    fn spans(&self, text: &str, out: &mut Vec<Span>) {
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
        let size = |at: usize| TextSize::try_from(at).expect("a document under 4 GiB");
        let mut not_prose = 0usize;
        for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
            match event {
                Event::Start(Tag::CodeBlock(_) | Tag::HtmlBlock | Tag::MetadataBlock(_)) => {
                    not_prose += 1
                }
                Event::End(TagEnd::CodeBlock | TagEnd::HtmlBlock | TagEnd::MetadataBlock(_)) => {
                    not_prose = not_prose.saturating_sub(1);
                }
                Event::Text(_) if not_prose == 0 => out.push(Span {
                    range: TextRange::new(size(range.start), size(range.end)),
                    kind: SpanKind::Prose,
                }),
                Event::Code(_) if not_prose == 0 => out.push(Span {
                    range: TextRange::new(size(range.start), size(range.end)),
                    kind: SpanKind::Value,
                }),
                _ => {}
            }
        }
    }
}
