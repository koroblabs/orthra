//! BERT's uncased WordPiece, the tokenizer ELECTRA was trained with: text to vocabulary ids.
//!
//! Written here rather than read from Hugging Face's `tokenizer.json` through the `tokenizers`
//! crate, which candle depends on anyway: the vocabulary then travels inside the model file, and
//! the tokenizer is a hundred lines to read. Scoring the typo sample through it put the right fix
//! first 71.4 % of the time, against 71.1 % through `tokenizers` (2026-11-06). It follows Hugging
//! Face's `BertTokenizer` with `do_lower_case`:
//! - drop control characters and turn whitespace into spaces;
//! - set CJK ideographs apart;
//! - lower-case, and strip accents (NFD, then no combining marks);
//! - split at punctuation;
//! - match each word against the vocabulary, longest piece first, its later pieces as `##piece`.

use std::collections::HashMap;

use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

#[cfg(test)]
mod tests;

/// A word longer than this, in characters, is one unknown token, as in BERT.
const LONGEST_WORD: usize = 100;

#[derive(Debug)]
pub(crate) struct WordPiece {
    ids: HashMap<String, u32>,
    pub(crate) unknown: u32,
    pub(crate) mask: u32,
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) pad: u32,
}

impl WordPiece {
    /// The vocabulary, a token per line, its id the line's number.
    pub(crate) fn new(tokens: impl IntoIterator<Item = String>) -> Result<WordPiece, String> {
        let ids: HashMap<String, u32> = tokens
            .into_iter()
            .enumerate()
            .map(|(id, token)| (token, u32::try_from(id).unwrap_or(u32::MAX)))
            .collect();
        let special = |token: &str| {
            ids.get(token)
                .copied()
                .ok_or_else(|| format!("the vocabulary has no {token}"))
        };
        Ok(WordPiece {
            unknown: special("[UNK]")?,
            mask: special("[MASK]")?,
            start: special("[CLS]")?,
            end: special("[SEP]")?,
            pad: special("[PAD]")?,
            ids,
        })
    }

    /// The ids of `text`, with no `[CLS]` or `[SEP]` around them.
    pub(crate) fn ids(&self, text: &str, out: &mut Vec<u32>) {
        for word in words(text) {
            self.pieces(&word, out);
        }
    }

    fn pieces(&self, word: &str, out: &mut Vec<u32>) {
        if word.chars().count() > LONGEST_WORD {
            out.push(self.unknown);
            return;
        }
        let before = out.len();
        let mut start = 0;
        let mut piece = String::new();
        while start < word.len() {
            // The longest piece of the vocabulary the rest of the word starts with.
            let mut end = word.len();
            let found = loop {
                piece.clear();
                if start > 0 {
                    piece.push_str("##");
                }
                piece.push_str(&word[start..end]);
                if let Some(id) = self.ids.get(&piece) {
                    break Some(*id);
                }
                match word[start..end].char_indices().next_back() {
                    Some((last, _)) if last > 0 => end = start + last,
                    _ => break None,
                }
            };
            match found {
                Some(id) => {
                    out.push(id);
                    start = end;
                }
                None => {
                    // A word with a piece that is not in the vocabulary is one unknown token.
                    out.truncate(before);
                    out.push(self.unknown);
                    return;
                }
            }
        }
    }
}

/// The words BERT's basic tokenizer makes of `text`: cleaned, lower-cased, accents stripped, and
/// split at whitespace and punctuation, each punctuation character a word of its own.
fn words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let end = |word: &mut String, words: &mut Vec<String>| {
        if !word.is_empty() {
            words.push(std::mem::take(word));
        }
    };
    for c in text.chars() {
        if c == '\0' || c == '\u{fffd}' || (c.is_control() && !c.is_whitespace()) {
            continue;
        }
        if c.is_whitespace() {
            end(&mut word, &mut words);
            continue;
        }
        if is_cjk(c) || is_punctuation(c) {
            end(&mut word, &mut words);
            words.push(c.to_string());
            continue;
        }
        for lower in c.to_lowercase() {
            word.extend(lower.to_string().nfd().filter(|c| !is_combining_mark(*c)));
        }
    }
    end(&mut word, &mut words);
    words
}

/// BERT's punctuation: ASCII's non-alphanumeric printable characters, and Unicode's punctuation.
/// Unicode's categories are not in the standard library; the blocks below hold the punctuation
/// prose uses, and a symbol outside them stays in its word, as an unknown piece would anyway.
fn is_punctuation(c: char) -> bool {
    c.is_ascii_punctuation()
        || matches!(c,
            '¡' | '§' | '«' | '¶' | '·' | '»' | '¿' | '\u{37e}' | '\u{387}'
            | '\u{2010}'..='\u{2027}'
            | '\u{2030}'..='\u{205e}'
            | '\u{3001}'..='\u{3003}'
            | '\u{3008}'..='\u{3011}'
            | '\u{fe50}'..='\u{fe6b}'
            | '\u{ff01}'..='\u{ff0f}'
        )
}

/// The CJK ideographs BERT sets apart, as its `_is_chinese_char` lists them.
fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4e00}'..='\u{9fff}'
        | '\u{3400}'..='\u{4dbf}'
        | '\u{20000}'..='\u{2a6df}'
        | '\u{2a700}'..='\u{2b73f}'
        | '\u{2b740}'..='\u{2b81f}'
        | '\u{2b820}'..='\u{2ceaf}'
        | '\u{f900}'..='\u{faff}'
        | '\u{2f800}'..='\u{2fa1f}'
    )
}
