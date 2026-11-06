//! Comment lines that name people: `# Author: Jens Getreu <getreu at web.de>`. A license header or
//! a theme's credits is a list of names, and on helix's TOML such lines were 121 of 262 findings.
//! Which labels say so is each pack's data (`name-labels`), because `Author` is English.

use orthra_hosts::SpanKind;
use orthra_segment::Word;

#[cfg(test)]
mod tests;

/// The labels every enabled pack gives, lower-cased: `author`, `original author`.
#[derive(Debug, Clone, Default)]
pub(crate) struct NameLabels {
    labels: Vec<Box<str>>,
    /// The first word of each label: a line can only be labeled if its first word is one.
    firsts: Vec<Box<str>>,
}

impl NameLabels {
    pub(crate) fn new(labels: Vec<Box<str>>) -> NameLabels {
        let mut firsts: Vec<Box<str>> = labels
            .iter()
            .filter_map(|label| label.split(' ').next())
            .map(Box::from)
            .collect();
        firsts.sort_unstable();
        firsts.dedup();
        NameLabels { labels, firsts }
    }

    /// Drops the words of every comment line that starts with a label and a colon. The walk reads
    /// the text between words once, so it costs nothing per word beyond a look at that gap.
    pub(crate) fn drop_named_lines(
        &self,
        text: &str,
        words: &mut Vec<Word>,
        places: &mut Vec<SpanKind>,
    ) {
        if self.labels.is_empty() {
            return;
        }
        // Empty until a word is dropped, which most documents never do.
        let mut dropped: Vec<bool> = Vec::new();
        let mut previous_end = 0;
        // Words that start before this offset are on a named line.
        let mut named_until = 0;
        for (i, (word, place)) in words.iter().zip(places.iter()).enumerate() {
            let start = usize::from(word.range.start());
            let gap = &text[previous_end..start];
            previous_end = usize::from(word.range.end());
            let named = start < named_until
                || matches!(place, SpanKind::Comment | SpanKind::DocComment)
                    && self.begins_a_label(&text[word.range])
                    && Self::first_on_line(gap, i)
                    && {
                        let line_end = text[start..].find('\n').map_or(text.len(), |at| start + at);
                        let labeled = self.labeled(&text[start..line_end]);
                        if labeled {
                            named_until = line_end;
                        }
                        labeled
                    };
            if named {
                dropped.resize(words.len(), false);
                dropped[i] = true;
            }
        }
        if dropped.is_empty() {
            return;
        }
        let mut flags = dropped.iter();
        words.retain(|_| !flags.next().is_some_and(|dropped| *dropped));
        let mut flags = dropped.iter();
        places.retain(|_| !flags.next().is_some_and(|dropped| *dropped));
    }

    /// Whether `word` is the first word of a label, ignoring case. Most comment lines start with
    /// a word that is not, and are passed over without reading the rest of the line.
    fn begins_a_label(&self, word: &str) -> bool {
        self.firsts.iter().any(|first| {
            first.len() == word.len() && word.chars().flat_map(char::to_lowercase).eq(first.chars())
        })
    }

    /// Whether a word is the first on its line, given the text since the word before it (word
    /// `i`): nothing but markers, `#`, `//` or ` * `, since the line began.
    fn first_on_line(gap: &str, i: usize) -> bool {
        match gap.rfind('\n') {
            Some(at) => !gap[at..].chars().any(char::is_alphanumeric),
            None => i == 0 && !gap.chars().any(char::is_alphanumeric),
        }
    }

    /// Whether `line` starts with a label, then a colon: `Author:`, `Original author :`.
    fn labeled(&self, line: &str) -> bool {
        let Some((lead, _)) = line.split_once(':') else {
            return false;
        };
        if !lead
            .chars()
            .all(|c| c.is_alphabetic() || c == '-' || c == ' ')
        {
            return false;
        }
        self.labels.iter().any(|label| {
            let mut written = lead.split_whitespace();
            label.split(' ').all(|part| {
                written
                    .next()
                    .is_some_and(|w| w.chars().flat_map(char::to_lowercase).eq(part.chars()))
            }) && written.next().is_none()
        })
    }
}
