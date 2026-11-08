//! Comment lines, and lines of prose such as a `LICENSE`'s, that name people: `# Author: Jens Getreu <getreu at web.de>`. A license header or
//! a theme's credits is a list of names, and on helix's TOML such lines were 121 of 262 findings.
//! Which labels say so is each pack's data (`name-labels`, and `name-lines` for a word that needs no
//! colon, `Copyright`), because `Author` is English. A line of a few words and an email in angle
//! brackets is a name whatever the language: ` *  arcizan <ghostrevery@gmail.com>`. On redis,
//! copyright lines' names were a fifth of what was reported (2026-11-08).
//!
//! A handle in a code comment's conventions is a name too: `TODO(nweiz)`, and the code review
//! comments of OCaml's shops, `CR rgrinberg:` and `CR-someday amokhov:`. On dune they were 114 of
//! 347 findings, and `TODO(username)` is Dart's and Go's style (2026-11-09).

use orthra_hosts::SpanKind;
use orthra_segment::Word;

#[cfg(test)]
mod tests;

/// The labels every enabled pack gives, lower-cased: `author`, `original author`.
#[derive(Debug, Clone, Default)]
pub(crate) struct NameLabels {
    labels: Vec<Box<str>>,
    /// Words that start a line of names without a colon: `copyright`, `created by`.
    lines: Vec<Box<str>>,
    /// The first word of each label: a line can only be labeled if its first word is one.
    firsts: Vec<Box<str>>,
}

impl NameLabels {
    pub(crate) fn new(labels: Vec<Box<str>>, lines: Vec<Box<str>>) -> NameLabels {
        let mut firsts: Vec<Box<str>> = labels
            .iter()
            .filter_map(|label| label.split(' ').next())
            .map(Box::from)
            .collect();
        firsts.sort_unstable();
        firsts.dedup();
        NameLabels {
            labels,
            lines,
            firsts,
        }
    }

    /// Drops the words of every comment line that names people: one that starts with a label and a
    /// colon, or a line word, or that is a few words and an email. The walk reads the text between
    /// words once, so it costs nothing per word beyond a look at that gap, and a line is read only
    /// from its first word.
    pub(crate) fn drop_named_lines(
        &self,
        text: &str,
        words: &mut Vec<Word>,
        places: &mut Vec<SpanKind>,
    ) {
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
                    && Self::is_handle(text, words, i, gap)
                || matches!(
                    place,
                    SpanKind::Comment | SpanKind::DocComment | SpanKind::Prose
                ) && (Self::first_on_line(gap, i) || Self::after_copyright_sign(gap, i))
                    && {
                        let line_end = text[start..].find('\n').map_or(text.len(), |at| start + at);
                        let line = &text[start..line_end];
                        let first = &text[word.range];
                        let names = Self::after_copyright_sign(gap, i)
                            || self.begins_a_label(first) && self.labeled(line)
                            || self.begins_a_name_line(first, line)
                            || Self::name_and_email(line);
                        if names {
                            named_until = line_end;
                        }
                        names
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

    /// Whether `word` is the first word of a label, ignoring case.
    fn begins_a_label(&self, word: &str) -> bool {
        Self::is_one_of(word, &self.firsts)
    }

    /// Whether `word` is one of `words`, lower-cased, ignoring its case.
    fn is_one_of(word: &str, words: &[Box<str>]) -> bool {
        words.iter().any(|one| Self::same(word, one))
    }

    /// Whether `word` is `lowered`, ignoring its case.
    fn same(word: &str, lowered: &str) -> bool {
        word.chars()
            .flat_map(char::to_lowercase)
            .eq(lowered.chars())
    }

    /// Whether `line`, whose first word is `first`, starts with the words of a name line:
    /// `Copyright (c) 2009`, `Created by Olivier Poitrey on 9/21/09.` The first word is the
    /// segmenter's, so a colon or a bracket after it does not matter; the rest are read word by
    /// word.
    fn begins_a_name_line(&self, first: &str, line: &str) -> bool {
        self.lines.iter().any(|entry| {
            let mut wanted = entry.split(' ');
            let mut written = line.split_whitespace().skip(1);
            wanted.next().is_some_and(|word| Self::same(first, word))
                && wanted.all(|word| written.next().is_some_and(|w| Self::same(w, word)))
        })
    }

    /// Whether word `i`, after `gap`, the text since the word before it, is a handle: `nweiz` in
    /// `TODO(nweiz)`, `rgrinberg` in `CR rgrinberg:` and `amokhov` in `CR-someday amokhov:`. The
    /// markers are acronyms, which the segmenter makes no word of, so they are read in the gap.
    fn is_handle(text: &str, words: &[Word], i: usize, gap: &str) -> bool {
        const MARKERS: &[&str] = &["TODO", "FIXME", "XXX", "HACK", "NOTE", "BUG"];
        const REVIEWS: &[&str] = &["CR", "XCR"];
        // Whether `before` ends with one of `marks`, as a whole word.
        let ends_with = |before: &str, marks: &[&str]| {
            marks.iter().any(|mark| {
                before.strip_suffix(mark).is_some_and(|head| {
                    head.chars()
                        .next_back()
                        .is_none_or(|c| !c.is_alphanumeric())
                })
            })
        };
        let after = &text[usize::from(words[i].range.end())..];
        if let Some(before) = gap.strip_suffix('(') {
            return ends_with(before, MARKERS) && after.starts_with(')');
        }
        if !after.starts_with(':') {
            return false;
        }
        if gap
            .strip_suffix(' ')
            .is_some_and(|before| ends_with(before, REVIEWS))
        {
            return true;
        }
        // `CR-someday amokhov:`: the word before is the review's kind, after `CR-`.
        gap == " " && i > 0 && {
            let kind_start = usize::from(words[i - 1].range.start());
            let since = i
                .checked_sub(2)
                .map_or(0, |at| usize::from(words[at].range.end()));
            text[since..kind_start]
                .strip_suffix('-')
                .is_some_and(|before| ends_with(before, REVIEWS))
        }
    }

    /// Whether the line so far, given the text since the word before, is markers and a copyright
    /// sign, `(c)` or `©`, which says what follows is the holders' names in every language:
    /// ` * (c) Olivier Poitrey <rs@dailymotion.com>`. `i` is the word's index: the first word's gap
    /// is the line so far even with no line break in it.
    fn after_copyright_sign(gap: &str, i: usize) -> bool {
        let head = match gap.rfind('\n') {
            Some(at) => &gap[at..],
            None if i == 0 => gap,
            None => return false,
        };
        let signless = head.replace("(c)", "").replace("(C)", "").replace('©', "");
        signless.len() < head.len() && !signless.chars().any(char::is_alphanumeric)
    }

    /// Whether `line`, from its first word on, is one to four words of a name and an email in
    /// angle brackets, and no word after it: `Jane Doe <jane@example.com>`,
    /// `antirez <antirez at gmail dot com>`. A sentence that goes on past an email is prose.
    fn name_and_email(line: &str) -> bool {
        let Some(open) = line.find('<') else {
            return false;
        };
        let Some(close) = line[open..].find('>') else {
            return false;
        };
        let (name, email) = (&line[..open], &line[open + 1..open + close]);
        let after = &line[open + close + 1..];
        (email.contains('@') || email.contains(" at "))
            && !after.chars().any(char::is_alphabetic)
            && (1..=4).contains(&name.split_whitespace().count())
            && name
                .chars()
                .all(|c| c.is_alphabetic() || c.is_whitespace() || ".-'".contains(c))
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
