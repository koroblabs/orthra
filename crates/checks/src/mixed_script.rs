//! `mixed-script`: letters from two scripts in one word. `сount` with a Cyrillic `с` passes neither
//! dictionary, or looks right on screen; codebook reports it as an ordinary typo, `сount` → `count`,
//! which reads as nonsense to the person looking at it (`corpora/incumbent/08-mixed-script.case`). The fix is the
//! same word in one script ([`unmixed`]), from `mixed_script/homoglyphs.tsv`.

use std::sync::OnceLock;

use orthra_packs::Packs;
use orthra_text::script::Script;

use crate::{Check, CheckCode, DocFacts, Finding};

#[cfg(test)]
mod tests;

/// The scripts a word can be unmixed into, in the table's column order.
const SCRIPTS: [Script; 3] = [Script::Latin, Script::Cyrillic, Script::Greek];

/// Letters that look alike, a row per set and a column per script of [`SCRIPTS`].
fn homoglyphs() -> &'static [[Option<char>; 3]] {
    static TABLE: OnceLock<Vec<[Option<char>; 3]>> = OnceLock::new();
    TABLE.get_or_init(|| {
        include_str!("mixed_script/homoglyphs.tsv")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                let mut cells = line.split('\t').map(|cell| {
                    let mut chars = cell.chars();
                    match (chars.next(), chars.next()) {
                        (Some('-'), None) | (None, _) => None,
                        (Some(c), None) => Some(c),
                        _ => panic!("homoglyphs.tsv: {cell:?} is not one letter"),
                    }
                });
                [(); 3].map(|()| cells.next().flatten())
            })
            .collect()
    })
}

/// `word` in one script, as a fix: every spelling a pack of its script accepts, or, when none is
/// accepted, the one in the script most of the word is already in. `сount` with a Cyrillic `с` is
/// `count`, and not also `соunt` in Cyrillic, which no one meant. The letters look the same, so the
/// fix changes nothing anybody can see but the scripts.
pub fn unmixed(word: &str, packs: &Packs) -> Vec<String> {
    let mut found: Vec<(bool, usize, usize, String)> = Vec::new();
    for (column, script) in SCRIPTS.iter().enumerate() {
        let mut own = 0;
        let mut written = String::with_capacity(word.len());
        let complete = word.chars().all(|c| match Script::of_char(c) {
            None => {
                written.push(c);
                true
            }
            Some(of) if of == *script => {
                own += 1;
                written.push(c);
                true
            }
            Some(_) => match homoglyphs()
                .iter()
                .find(|row| row.contains(&Some(c)))
                .and_then(|row| row[column])
            {
                Some(stand_in) => {
                    written.push(stand_in);
                    true
                }
                None => false,
            },
        });
        if complete && own > 0 {
            let accepted = packs.for_script(*script).any(|pack| pack.accepts(&written));
            found.push((accepted, own, column, written));
        }
    }
    found.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
    let any_accepted = found.first().is_some_and(|(accepted, ..)| *accepted);
    found
        .into_iter()
        .take_while(|(accepted, ..)| *accepted == any_accepted)
        .take(if any_accepted { SCRIPTS.len() } else { 1 })
        .map(|(_, _, _, word)| word)
        .collect()
}

#[derive(Debug)]
pub struct MixedScript;

impl Check for MixedScript {
    fn code(&self) -> CheckCode {
        CheckCode("mixed-script")
    }

    fn run(&self, facts: &DocFacts<'_>, out: &mut Vec<Finding>) {
        for word in facts
            .words
            .iter()
            .filter(|word| word.script == Script::Mixed)
        {
            out.push(Finding {
                range: word.range,
                run: word.run,
                code: self.code(),
                fixes: unmixed(&facts.text[word.range], facts.packs),
            });
        }
    }
}
