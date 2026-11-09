//! `wrong-layout`: a word typed with the keyboard still in another language's layout.
//! `руддщ цщкдв` is `hello world` typed on a Ukrainian layout,
//! and `ghbdsn` is `привіт` typed on an English one.
//! The fix is the same keys in the layout that makes a word: what was meant, letter for letter.
//!
//! A word is asked about when its own script's packs do not accept it: unknown, or in a script no
//! enabled pack speaks, since an English writer's slip is not checked as Ukrainian and is still a
//! slip worth saying. Words are judged a run at a time ([`WrongLayout::run`]), and a word is
//! offered only when a pack of the other layout's script accepts it, so a typo is not turned into
//! a guess. Which key types which letter is data, `wrong_layout/layouts.tsv`.

use std::sync::OnceLock;

use orthra_lexicon::Verdict;
use orthra_packs::Packs;
use orthra_text::{TextRange, TextSize, script::Script};

use crate::{Check, CheckCode, DocFacts, Finding};

#[cfg(test)]
mod tests;

/// A keyboard layout: its script, and the letter each key types, in the table's key order.
struct Layout {
    script: Script,
    keys: Vec<char>,
}

fn layouts() -> &'static [Layout] {
    static LAYOUTS: OnceLock<Vec<Layout>> = OnceLock::new();
    LAYOUTS.get_or_init(|| {
        let mut lines = include_str!("wrong_layout/layouts.tsv")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'));
        let mut layouts: Vec<Layout> = lines
            .next()
            .expect("layouts.tsv names its layouts")
            .split('\t')
            .map(|head| Layout {
                script: head
                    .split_once('/')
                    .and_then(|(_, code)| Script::from_code(code))
                    .unwrap_or_else(|| panic!("layouts.tsv: {head:?} is not `name/Script`")),
                keys: Vec::new(),
            })
            .collect();
        for line in lines {
            for (layout, cell) in layouts.iter_mut().zip(line.split('\t')) {
                let mut chars = cell.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => layout.keys.push(c),
                    _ => panic!("layouts.tsv: {cell:?} is not one character"),
                }
            }
        }
        layouts
    })
}

/// `word` typed again in each other layout, the first spelling a pack of that layout's script
/// accepts: `руддщ` is `hello`, `ghbdsn` is `привіт`. Case is kept letter by letter. `None` when no
/// layout makes a word.
pub fn retyped(word: &str, packs: &Packs) -> Option<String> {
    let script = Script::of(word);
    for from in layouts().iter().filter(|layout| layout.script == script) {
        for to in layouts().iter().filter(|layout| layout.script != script) {
            let Some(typed) = retype(word, from, to) else {
                continue;
            };
            if Script::of(&typed) == to.script
                && packs.for_script(to.script).any(|pack| pack.accepts(&typed))
            {
                return Some(typed);
            }
        }
    }
    None
}

/// `range` of `text` with the characters around it that are keys of a layout of its script and not
/// letters there: `lzre.` is `дякую`, its `ю` on the full stop's key, which segmentation leaves out
/// of the word. A slip with such a key inside the word (`,f,ecz`, `бабуся`) is cut into short
/// words there, and is not found.
fn with_keys_around(text: &str, range: TextRange, script: Script) -> TextRange {
    let key = |c: char| {
        !c.is_alphabetic()
            && layouts()
                .iter()
                .any(|layout| layout.script == script && layout.keys.contains(&c))
    };
    let (mut start, mut end) = (usize::from(range.start()), usize::from(range.end()));
    while let Some(c) = text[..start].chars().next_back().filter(|c| key(*c)) {
        start -= c.len_utf8();
    }
    while let Some(c) = text[end..].chars().next().filter(|c| key(*c)) {
        end += c.len_utf8();
    }
    let at = |offset: usize| TextSize::try_from(offset).unwrap_or_default();
    TextRange::new(at(start), at(end))
}

/// `word`'s keys in `from`, typed in `to`; `None` when a letter is on no key of `from`.
fn retype(word: &str, from: &Layout, to: &Layout) -> Option<String> {
    let mut typed = String::with_capacity(word.len());
    for c in word.chars() {
        let lower = c.to_lowercase().next()?;
        let key = from.keys.iter().position(|k| *k == lower)?;
        let other = to.keys[key];
        if c.is_uppercase() {
            typed.extend(other.to_uppercase());
        } else {
            typed.push(other);
        }
    }
    Some(typed)
}

#[derive(Debug)]
pub struct WrongLayout;

impl WrongLayout {
    /// Whether word `i` may be a slip: a word its own script's packs do not accept -- unknown, or
    /// in a script no enabled pack speaks, short ones included -- in prose that is not test data.
    ///
    /// A word in a script no pack speaks is asked only in a document that has a word in a script a
    /// pack does: a slip is made in the middle of writing that language, and a Ukrainian file
    /// with no Latin word is Ukrainian. So such a file does not read the English pack either.
    fn candidate(facts: &DocFacts<'_>, i: usize, spoken_elsewhere: bool) -> bool {
        let word = &facts.words[i];
        let unaccepted = match facts.verdicts[i] {
            Verdict::Unknown => true,
            Verdict::NotChecked => {
                spoken_elsewhere
                    && !matches!(word.script, Script::Mixed | Script::None | Script::Other)
                    && facts.packs.for_script(word.script).next().is_none()
            }
            Verdict::Correct | Verdict::KnownTypo => false,
        };
        unaccepted && !facts.is_code(i) && !facts.is_fixture(i)
    }
}

impl Check for WrongLayout {
    fn code(&self) -> CheckCode {
        CheckCode("wrong-layout")
    }

    /// A run of neighboring candidates in one script -- on one line, nothing accepted between
    /// them -- is a slip when every word of it is another layout's word, and then each is that
    /// finding. One word alone is weak evidence: on all of UA-GEC's Ukrainian text, with only an
    /// English pack, a word at a time found 1 341 "slips", `часу` as `xfce` and `руки` as `herb`
    /// (2026-11-06). A Ukrainian sentence always has a word that makes nothing typed again, and a
    /// sentence typed with the wrong layout has none.
    fn run(&self, facts: &DocFacts<'_>, out: &mut Vec<Finding>) {
        let spoken = facts
            .words
            .iter()
            .any(|word| facts.packs.for_script(word.script).next().is_some());
        let candidate = |i| Self::candidate(facts, i, spoken);
        let mut i = 0;
        while i < facts.words.len() {
            if !candidate(i) {
                i += 1;
                continue;
            }
            let script = facts.words[i].script;
            let mut end = i + 1;
            while end < facts.words.len()
                && facts.words[end].script == script
                && candidate(end)
                && !facts.text
                    [facts.words[end - 1].range.end().into()..facts.words[end].range.start().into()]
                    .contains('\n')
            {
                end += 1;
            }
            self.run_of(facts, i..end, out);
            i = end;
        }
    }
}

impl WrongLayout {
    /// The findings for the run of candidates `run`: every word of it typed again, or none.
    fn run_of(&self, facts: &DocFacts<'_>, run: std::ops::Range<usize>, out: &mut Vec<Finding>) {
        let letters: usize = run
            .clone()
            .map(|i| {
                facts.text[facts.words[i].range]
                    .chars()
                    .filter(|c| c.is_alphabetic())
                    .count()
            })
            .sum();
        // As unknown-word: fewer than four letters is an abbreviation more often than a slip.
        if letters < 4 {
            return;
        }
        let mut found = Vec::with_capacity(run.len());
        for i in run {
            let word = &facts.words[i];
            // The word with the keys around it first, then the word alone.
            let around = with_keys_around(facts.text, word.range, word.script);
            let typed = [around, word.range]
                .into_iter()
                .take(if around == word.range { 1 } else { 2 })
                .find_map(|range| Some((range, retyped(&facts.text[range], facts.packs)?)));
            match typed {
                Some((range, fix)) => found.push(Finding {
                    range,
                    run: range,
                    code: self.code(),
                    fixes: vec![fix],
                }),
                None => return,
            }
        }
        out.extend(found);
    }
}
