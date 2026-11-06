//! The sentence a rewrite is asked for: the one the cursor is in, found by punctuation, inside a
//! passage of text meant to be read that the engine found (`orthra_engine::Engine::sentence_at`).
//!
//! Deliberately simple, so that what a person is offered is predictable:
//! - a sentence ends at `.`, `!`, `?` or `…`, with the quotes, brackets and emphasis that close it,
//!   followed by whitespace or the passage's end; and at a blank line;
//! - its edges are its first and last word, with at most two quotes, brackets or emphasis marks
//!   that open or close them where they stand apart from what is before or after: `**Bold** text`
//!   and `("quoted")`, but not a comment's `//`, a block comment's `*/` or a docstring's `"""`,
//!   which stay where they are, out of the model's reach;
//! - the cursor is in it, edges included;
//! - it is never longer than [`SENTENCE_BYTES`], and so never a whole file.
//!
//! `e.g. this` ends a sentence after `e.g.`: an abbreviation list is a language's data, and a
//! shorter sentence than meant costs a rewrite less than a longer one.

use orthra_text::{TextRange, TextSize};

#[cfg(test)]
mod tests;

/// The longest sentence offered to be rewritten, in bytes. A sentence a person writes is shorter;
/// a longer stretch with no sentence end is a list, a table or a log, and not one sentence.
pub const SENTENCE_BYTES: usize = 1_000;

/// What ends a sentence.
const TERMINATORS: &[char] = &['.', '!', '?', '…'];
/// What may open a sentence's first word: quotes, brackets, emphasis, inline code.
const OPENERS: &[char] = &[
    '"', '\'', '“', '‘', '„', '«', '(', '[', '*', '_', '`', '¿', '¡',
];
/// What may close its last word, after its terminator or without one.
const CLOSERS: &[char] = &['"', '\'', '”', '’', '»', ')', ']', '*', '_', '`'];
/// The most opening or closing marks a sentence's edge takes: `**`, `("`. A docstring's `"""` is
/// three, and stays out.
const MARKS: usize = 2;

/// The sentence around `at` in `passage` of `text`, or `None`: no word there, the cursor outside
/// it, or longer than [`SENTENCE_BYTES`].
pub fn around(text: &str, passage: TextRange, at: TextSize) -> Option<TextRange> {
    let (lo, hi, at) = (
        usize::from(passage.start()),
        usize::from(passage.end()),
        usize::from(at),
    );
    if hi > text.len() || !(lo <= at && at <= hi) || !text.is_char_boundary(at) {
        return None;
    }
    // Only as far as the longest sentence reaches either way: a passage can be a whole text file.
    let mut from = lo.max(at.saturating_sub(SENTENCE_BYTES));
    while !text.is_char_boundary(from) {
        from += 1;
    }
    let mut to = hi.min(at + SENTENCE_BYTES);
    while !text.is_char_boundary(to) {
        to -= 1;
    }
    let ends = ends(text, from, to, hi);
    // A sentence that starts or ends past the reach is longer than any offered.
    let start = match ends.iter().rev().find(|&&end| end < at) {
        Some(&end) => end,
        None if from == lo => lo,
        None => return None,
    };
    let end = match ends.iter().find(|&&end| end >= at) {
        Some(&end) => end,
        None if to == hi => hi,
        None => return None,
    };
    let start = first_word(text, start, end)?;
    let end = last_word(text, start, end)?;
    if !(start <= at && at <= end) || end - start > SENTENCE_BYTES {
        return None;
    }
    Some(TextRange::new(size(start), size(end)))
}

fn size(at: usize) -> TextSize {
    TextSize::try_from(at).expect("a document under 4 GiB")
}

/// Where sentences end between `from` and `to`, looking as far as `hi` to tell: after a
/// terminator and what closes it, followed by whitespace or `hi`; and at a blank line.
fn ends(text: &str, from: usize, to: usize, hi: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut chars = text[from..to].char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let i = from + i;
        if TERMINATORS.contains(&c) {
            let mut end = i + c.len_utf8();
            while let Some(&(_, next)) = chars.peek() {
                if !TERMINATORS.contains(&next) && !CLOSERS.contains(&next) {
                    break;
                }
                end += next.len_utf8();
                chars.next();
            }
            if text[end..hi].chars().next().is_none_or(char::is_whitespace) {
                out.push(end);
            }
        } else if c == '\n' {
            let rest = &text[i + 1..hi];
            if rest
                .find('\n')
                .is_some_and(|line| rest[..line].trim().is_empty())
            {
                out.push(i);
            }
        }
    }
    out
}

/// Where the sentence whose text starts at `start` begins: its first word, with what opens it.
fn first_word(text: &str, start: usize, end: usize) -> Option<usize> {
    let (word, _) = text[start..end]
        .char_indices()
        .find(|(_, c)| c.is_alphanumeric())?;
    let word = start + word;
    let mut first = word;
    for c in text[start..word].chars().rev().take(MARKS) {
        if !OPENERS.contains(&c) {
            break;
        }
        first -= c.len_utf8();
    }
    // Marks open the word only where they stand apart from what is before them.
    let before = text[start..first].chars().next_back();
    Some(if before.is_some_and(|c| !c.is_whitespace()) {
        word
    } else {
        first
    })
}

/// Where the sentence ends, before `end`: its last word, with its terminator and what closes it.
fn last_word(text: &str, start: usize, end: usize) -> Option<usize> {
    let (word, c) = text[start..end]
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_alphanumeric())?;
    let word = start + word + c.len_utf8();
    let after_word = &text[word..end];
    let terminated = word
        + after_word
            .chars()
            .take_while(|c| TERMINATORS.contains(c))
            .map(char::len_utf8)
            .sum::<usize>();
    let (mut last, mut closers) = (word, 0);
    for c in after_word.chars() {
        let closes = !TERMINATORS.contains(&c);
        if closes && !(CLOSERS.contains(&c) && closers < MARKS) {
            break;
        }
        closers += usize::from(closes);
        last += c.len_utf8();
    }
    // As at the start: the marks close the word only where they stand apart from what follows,
    // and the terminator right after it ends it either way: `here."""` ends at `here.`.
    let after = text[last..end].chars().next();
    Some(if after.is_some_and(|c| !c.is_whitespace()) {
        terminated
    } else {
        last
    })
}
