//! What the engine keeps of an open document between two checks of it, so that a keystroke is
//! parsed and segmented again only where it changed.
//!
//! Two things are kept, and each is told of every edit before the next check:
//! - the host's parse ([`orthra_hosts::parsed::Parsed`]), which tree-sitter reuses wherever the
//!   edits did not reach;
//! - the words of every span of the last check. Segmentation reads a span's text and the rules
//!   and nothing else, so a span that comes back at the same range, of the same kind, untouched by
//!   any edit since, has the same text and the same words. An edit drops every span it touches,
//!   at its edges too, and moves the ones after it.
//!
//! Nothing kept changes an answer: `incremental == from-scratch` holds every fixture's edited
//! findings to a fresh check's.

use std::ops::Range;

use orthra_hosts::{Span, SpanKind};
use orthra_segment::{Rules, Word};
use orthra_text::{TextRange, TextSize, script::Script};

/// The rules a document's words were segmented under: its host, whether backslashes escape, and
/// each word-internal character with its script.
type SegmentedBy = (&'static str, bool, Vec<(char, Script)>);

/// What a check of a document kept for the next one. Empty until the first check.
#[derive(Debug, Default)]
pub struct Parsed {
    pub(crate) host: orthra_hosts::parsed::Parsed,
    words: KeptWords,
}

impl Parsed {
    /// `range` of `before`, the text as it is now, is about to become `inserted`.
    pub fn edit(&mut self, before: &str, range: TextRange, inserted: &str) {
        self.host.edit(before, range, inserted);
        self.words.edit(range, TextSize::of(inserted));
    }

    /// The text is about to be replaced whole: nothing kept is worth keeping.
    pub fn clear(&mut self) {
        self.host.clear();
        self.words = KeptWords::default();
    }

    /// The words of `spans` and the kind of span each is in, segmented under `rules` by the host
    /// `host`: the kept words of every span the edits left alone, and the rest segmented now.
    pub(crate) fn words(
        &mut self,
        text: &str,
        spans: &[Span],
        host: &'static str,
        rules: &Rules<'_>,
    ) -> (Vec<Word>, Vec<SpanKind>) {
        self.words.segment(text, spans, host, rules)
    }
}

#[derive(Debug, Default)]
struct KeptWords {
    /// The rules the words were segmented under: the host, its escapes, and the word-internal
    /// characters of the packs. Words segmented under other rules are not reused.
    rules: Option<SegmentedBy>,
    spans: Vec<KeptSpan>,
    /// Every kept span's words, each relative to its span's start.
    words: Vec<Word>,
}

#[derive(Debug)]
struct KeptSpan {
    range: TextRange,
    kind: SpanKind,
    words: Range<usize>,
}

impl KeptWords {
    fn edit(&mut self, range: TextRange, inserted: TextSize) {
        let (start, end) = (range.start(), range.end());
        self.spans.retain_mut(|kept| {
            if kept.range.end() < start {
                true
            } else if kept.range.start() > end {
                // After the edit: `kept.range.start() > end >= end - start`, so this cannot wrap.
                kept.range = TextRange::at(
                    kept.range.start() - (end - start) + inserted,
                    kept.range.len(),
                );
                true
            } else {
                // Touched, its edges included: text written at a span's end may be part of it.
                false
            }
        });
    }

    fn segment(
        &mut self,
        text: &str,
        spans: &[Span],
        host: &'static str,
        rules: &Rules<'_>,
    ) -> (Vec<Word>, Vec<SpanKind>) {
        let segmented_by = (host, rules.backslash_escapes, rules.word_internal.to_vec());
        if self.rules.as_ref() != Some(&segmented_by) {
            self.spans.clear();
            self.rules = Some(segmented_by);
        }
        // Spans and kept spans are both in order, so one walk pairs each span with its old self.
        let mut next = 0;
        let reused: Vec<Option<Range<usize>>> = spans
            .iter()
            .map(|span| {
                while self
                    .spans
                    .get(next)
                    .is_some_and(|kept| kept.range.start() < span.range.start())
                {
                    next += 1;
                }
                self.spans
                    .get(next)
                    .filter(|kept| kept.range == span.range && kept.kind == span.kind)
                    .map(|kept| kept.words.clone())
            })
            .collect();

        // Everything else in one call, as segmentation shares its buffers across a document.
        let mut fresh = Vec::new();
        orthra_segment::words(
            text,
            spans
                .iter()
                .zip(&reused)
                .filter(|(_, reused)| reused.is_none())
                .map(|(span, _)| span.range),
            rules,
            &mut fresh,
        );

        let mut words = Vec::with_capacity(fresh.len() + self.words.len());
        let mut places = Vec::with_capacity(words.capacity());
        let mut kept_spans = Vec::with_capacity(spans.len());
        let mut kept_words = Vec::with_capacity(words.capacity());
        let mut fresh = fresh.into_iter().peekable();
        for (span, reused) in spans.iter().zip(reused) {
            let first = words.len();
            let base = span.range.start();
            match reused {
                Some(old) => {
                    words.extend(self.words[old].iter().map(|word| moved(word, base, true)))
                }
                None => {
                    while let Some(word) =
                        fresh.next_if(|word| word.range.start() < span.range.end())
                    {
                        words.push(word);
                    }
                }
            }
            places.extend(std::iter::repeat_n(span.kind, words.len() - first));
            let at = kept_words.len();
            kept_words.extend(words[first..].iter().map(|word| moved(word, base, false)));
            kept_spans.push(KeptSpan {
                range: span.range,
                kind: span.kind,
                words: at..kept_words.len(),
            });
        }
        self.spans = kept_spans;
        self.words = kept_words;
        (words, places)
    }
}

/// `word` moved to start from `base` (`to`), or from the start of the text to `base` (`!to`).
fn moved(word: &Word, base: TextSize, to: bool) -> Word {
    let shift = |range: TextRange| {
        if to { range + base } else { range - base }
    };
    Word {
        range: shift(word.range),
        run: shift(word.run),
        ..*word
    }
}
