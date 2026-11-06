//! The documents the editor has open: their text, version and language, edited as the client says.
//!
//! A buffer is stored once, here, and nowhere else (`CLAUDE.md` §5).

use std::collections::HashMap;

use lspkit_runtime::protocol::ls_types::{TextDocumentContentChangeEvent, Uri};
use lspkit_text::lsp::Edit;
use orthra_engine::Parsed;
use orthra_text::line_index::Encoding;

#[derive(Debug)]
pub(crate) struct Document {
    /// The file's path, or the URI itself for a buffer that has none.
    pub(crate) path: String,
    pub(crate) language: String,
    pub(crate) text: String,
    pub(crate) version: i32,
}

#[derive(Debug, Default)]
pub(crate) struct Documents {
    open: HashMap<Uri, Document>,
    /// The parse of the document checked last, told of every edit to it since: a keystroke there
    /// is parsed again where it changed, not from the start. One document's, not every open
    /// one's: a tree's memory grows with its text, and only the document being typed in gains
    /// from keeping one. Checking another document moves it there.
    parsed: Option<(Uri, Parsed)>,
}

/// Why a change was not applied.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Refused {
    /// No such document is open.
    NotOpen,
    /// The client sent a version older than the one held: it and this server disagree about the
    /// order of edits, and applying it would describe the wrong text.
    Stale { held: i32 },
    /// A range that is not in the text. The document keeps the edits before it.
    OutOfRange,
}

impl Documents {
    pub(crate) fn open(&mut self, uri: Uri, document: Document) {
        self.forget_parse(&uri);
        self.open.insert(uri, document);
    }

    pub(crate) fn close(&mut self, uri: &Uri) -> Option<Document> {
        self.forget_parse(uri);
        self.open.remove(uri)
    }

    pub(crate) fn get(&self, uri: &Uri) -> Option<&Document> {
        self.open.get(uri)
    }

    /// The document at `uri`, and the parse to check it with: its own, if it was the last one
    /// checked, and a fresh one that takes the place of the last one's otherwise.
    pub(crate) fn parsed_for(&mut self, uri: &Uri) -> Option<(&Document, &mut Parsed)> {
        let document = self.open.get(uri)?;
        if self.parsed.as_ref().is_none_or(|(kept, _)| kept != uri) {
            self.parsed = Some((uri.clone(), Parsed::default()));
        }
        let (_, parsed) = self.parsed.as_mut()?;
        Some((document, parsed))
    }

    fn forget_parse(&mut self, uri: &Uri) {
        if self.parsed.as_ref().is_some_and(|(kept, _)| kept == uri) {
            self.parsed = None;
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&Uri, &Document)> {
        self.open.iter()
    }

    /// Applies the client's changes, in order: a whole new text, or a range replaced. The
    /// document's parse, if it is kept, is told of each one before it lands.
    pub(crate) fn change(
        &mut self,
        uri: &Uri,
        version: i32,
        changes: Vec<TextDocumentContentChangeEvent>,
        encoding: Encoding,
    ) -> Result<(), Refused> {
        let document = self.open.get_mut(uri).ok_or(Refused::NotOpen)?;
        if version < document.version {
            return Err(Refused::Stale {
                held: document.version,
            });
        }
        document.version = version;
        let mut parsed = match &mut self.parsed {
            Some((kept, parsed)) if kept == uri => Some(parsed),
            _ => None,
        };
        lspkit_text::lsp::apply_each(&mut document.text, changes, encoding, |edit| {
            let Some(parsed) = parsed.as_deref_mut() else {
                return;
            };
            match edit {
                Edit::Range {
                    before,
                    range,
                    inserted,
                } => parsed.edit(before, range, inserted),
                Edit::Whole => parsed.clear(),
            }
        })
        .map_err(|_| Refused::OutOfRange)
    }
}
