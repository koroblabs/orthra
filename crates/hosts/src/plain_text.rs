//! A text file: all of it is prose.

use orthra_text::{TextRange, TextSize};

use crate::{Conventions, Host, Span, SpanKind};

#[derive(Debug)]
pub struct PlainText;

impl Host for PlainText {
    fn id(&self) -> &'static str {
        "plain-text"
    }

    fn language_ids(&self) -> &'static [&'static str] {
        &["plaintext", "text"]
    }

    /// The files of a project with no extension that are text for people: its license, its
    /// authors, its changes. An editor opens them as plain text, and on disk they are read the
    /// same. `LICENSE.md` stays Markdown, by its extension.
    fn file_names(&self) -> &'static [&'static str] {
        &[
            "AUTHORS",
            "CHANGELOG",
            "CHANGES",
            "CONTRIBUTORS",
            "COPYING",
            "COPYING-*",
            "COPYRIGHT",
            "CREDITS",
            "HISTORY",
            "INSTALL",
            "LICENCE",
            "LICENSE",
            "LICENSE-*",
            "MAINTAINERS",
            "NEWS",
            "NOTICE",
            "README",
            "THANKS",
            "TODO",
            "UNLICENSE",
        ]
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["txt", "text"]
    }

    fn conventions(&self) -> Conventions {
        Conventions::default()
    }

    fn spans(&self, text: &str, out: &mut Vec<Span>) {
        out.push(Span {
            range: TextRange::up_to(TextSize::of(text)),
            kind: SpanKind::Prose,
        });
    }
}
