//! The writing system a word is written in.
//!
//! A word goes to the packs for its script (`docs/01-architecture.md` §4), so the script is decided
//! once, here, from the Unicode Script property of the word's letters. Marks, digits and
//! punctuation (`Common`, `Inherited`) say nothing about a word's script and are passed over.

use unicode_script::{Script as Unicode, UnicodeScript};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Script {
    Latin,
    Cyrillic,
    Greek,
    /// Any other script. No pack speaks one yet, so such a word is not checked.
    Other,
    /// Letters from two scripts in one word: `сount` with a Cyrillic `с`. Its own finding.
    Mixed,
    /// No letters at all.
    None,
}

impl Script {
    pub fn of(word: &str) -> Script {
        // Most words are ASCII, and an ASCII letter is Latin without a table lookup.
        if word.is_ascii() {
            return if word.bytes().any(|b| b.is_ascii_alphabetic()) {
                Script::Latin
            } else {
                Script::None
            };
        }
        let mut found = None;
        for c in word.chars() {
            let Some(script) = Script::of_char(c) else {
                continue;
            };
            match found {
                None => found = Some(script),
                Some(seen) if seen == script => {}
                Some(_) => return Script::Mixed,
            }
        }
        found.unwrap_or(Script::None)
    }

    /// The script of one character, or `None` for a mark, a digit or punctuation, which belong
    /// to every script.
    pub fn of_char(c: char) -> Option<Script> {
        if c.is_ascii() {
            return c.is_ascii_alphabetic().then_some(Script::Latin);
        }
        match c.script() {
            Unicode::Latin => Some(Script::Latin),
            Unicode::Cyrillic => Some(Script::Cyrillic),
            Unicode::Greek => Some(Script::Greek),
            Unicode::Common | Unicode::Inherited | Unicode::Unknown => None,
            _ => Some(Script::Other),
        }
    }

    /// Whether a character's script has letters that look like another's: Latin, Cyrillic and
    /// Greek share `a`, `о`, `ο`. Letters of these stay in one word when they meet, so that
    /// `mixed-script` sees `сount`; any other script ends a word where it meets them, as Chinese
    /// and Japanese are written against Latin with no space: `preview窗口`. `None` for a character
    /// with no script.
    pub fn is_confusable(c: char) -> Option<bool> {
        Script::of_char(c).map(|script| script != Script::Other)
    }

    /// The ISO 15924 code a pack names its script with: `Latn`, `Cyrl`, `Grek`.
    pub fn from_code(code: &str) -> Option<Script> {
        match code {
            "Latn" => Some(Script::Latin),
            "Cyrl" => Some(Script::Cyrillic),
            "Grek" => Some(Script::Greek),
            _ => None,
        }
    }
}
