//! What a Hunspell `.aff` says, as far as a list of word forms can hold it, read the way spellbook
//! reads it: the forms a compiled dictionary must list, and how a word is converted before it is
//! looked up.
//!
//! A directive that changes what is accepted in a way a list of forms cannot say is refused, with
//! its name: compounding, prefixes, a suffix that takes further affixes, and the flags that make a
//! form need an affix, forbid it, or fix its case. The pack keeps `hunspell` then. What only
//! shapes spellbook's suggestions (`TRY`, `REP`, `MAP`, `KEY`) is read by no one here, since a
//! compiled dictionary suggests by edit distance.

use std::{borrow::Cow, collections::HashMap};

use crate::DictError;

/// Directives a compiled dictionary refuses, because the forms it lists would accept what
/// spellbook does not, or miss what it accepts.
const REFUSED: &[&str] = &[
    "PFX",
    "AF",
    "COMPOUNDFLAG",
    "COMPOUNDBEGIN",
    "COMPOUNDMIDDLE",
    "COMPOUNDEND",
    "COMPOUNDLAST",
    "COMPOUNDRULE",
    "ONLYINCOMPOUND",
    "NEEDAFFIX",
    "PSEUDOROOT",
    "CIRCUMFIX",
    "FORBIDDENWORD",
    "KEEPCASE",
    "CHECKSHARPS",
    "NOSUGGEST",
    "FORBIDWARN",
    "OCONV",
];

/// How a word is changed before it is looked up: what spellbook does to it, and what the forms
/// were built for.
#[derive(Debug, Default)]
pub(crate) struct Lookup {
    /// `IGNORE`: removed from every word, the dictionary's and the one checked.
    pub(crate) ignore: Vec<char>,
    /// `ICONV`: replaced in the word checked, the longest pattern first at each place. dict_uk
    /// writes `ICONV a 0`, which spellbook reads as the character `0`, not as nothing: a Latin
    /// letter inside a Cyrillic word then makes it no word at all.
    pub(crate) conversions: Vec<(String, String)>,
    /// `BREAK`: where a word not found whole is split and each part checked. `^-` breaks at the
    /// start, `-$` at the end, anything else in the middle.
    pub(crate) breaks: Breaks,
}

#[derive(Debug)]
pub(crate) struct Breaks {
    pub(crate) start: Vec<String>,
    pub(crate) middle: Vec<String>,
    pub(crate) end: Vec<String>,
}

impl Breaks {
    fn new(patterns: &[&str]) -> Breaks {
        let mut breaks = Breaks {
            start: Vec::new(),
            middle: Vec::new(),
            end: Vec::new(),
        };
        for pattern in patterns {
            if let Some(start) = pattern.strip_prefix('^') {
                breaks.start.push(start.to_owned());
            } else if let Some(end) = pattern.strip_suffix('$') {
                breaks.end.push(end.to_owned());
            } else {
                breaks.middle.push((*pattern).to_owned());
            }
        }
        breaks
    }
}

impl Default for Breaks {
    /// Hunspell's, when the `.aff` names none.
    fn default() -> Breaks {
        Breaks::new(&["^-", "-", "-$"])
    }
}

/// An `.aff` read whole: the lookup, and the suffixes the forms are made with.
pub(crate) struct Aff {
    pub(crate) lookup: Lookup,
    flags: Flags,
    suffixes: HashMap<u64, Flagged>,
}

/// The suffixes of one flag.
#[derive(Default)]
struct Flagged {
    groups: Vec<Suffixes>,
    /// The most bytes any of them strips.
    longest_strip: usize,
}

/// How flags are written, read byte for byte as spellbook and Hunspell read them.
#[derive(Debug, Clone, Copy)]
enum Flags {
    /// One byte per flag: Hunspell's default. A character of two bytes is two flags on a stem,
    /// and its first byte alone in an affix's header: dict_uk writes flags up to U+00BF, so each
    /// of `\u{80}` to `\u{bf}` names the affixes of all of them, in spellbook as in Hunspell.
    Short,
    /// `FLAG UTF-8`: one UTF-16 unit per flag.
    Utf8,
    /// `FLAG long`: two bytes per flag.
    Long,
    /// `FLAG num`: numbers, comma-separated.
    Num,
}

/// Suffixes of one flag that strip the same text under the same condition, so a stem is tested
/// once for all of them: dict_uk writes `ів еви [^жшщ]ів`, `ів евих [^жшщ]ів` and so on, and
/// testing each of its 5 802 suffixes on its own was most of what compiling it cost.
struct Suffixes {
    strip: String,
    condition: Condition,
    /// The condition as written, to know a group from another.
    written: String,
    adds: Vec<String>,
}

/// An affix's condition: elements matched against the end of the stem, in order.
struct Condition(Vec<Element>);

enum Element {
    Any,
    Char(char),
    In(Vec<char>),
    NotIn(Vec<char>),
}

impl Aff {
    /// Reads `text`, refusing what a compiled dictionary cannot hold.
    pub(crate) fn parse(text: &str) -> Result<Aff, DictError> {
        let fail = |line: usize, why: String| DictError(format!(".aff line {}: {why}", line + 1));
        let mut lookup = Lookup::default();
        let mut breaks: Option<Vec<&str>> = None;
        let mut flags = Flags::Short;
        // Suffix lines in the order written, read once the flag type is known.
        let mut suffix_lines: Vec<(usize, Vec<&str>)> = Vec::new();
        let mut lines = text.lines().enumerate().peekable();
        while let Some((at, line)) = lines.next() {
            let words: Vec<&str> = line.split_whitespace().collect();
            let Some(&directive) = words.first() else {
                continue;
            };
            if directive.starts_with('#') {
                continue;
            }
            if REFUSED.contains(&directive) {
                return Err(fail(
                    at,
                    format!(
                        "{directive} is not something a compiled dictionary can hold; \
                         use `dictionary.hunspell`"
                    ),
                ));
            }
            match directive {
                "SET" => {
                    let set = words.get(1).copied().unwrap_or_default();
                    if !set.eq_ignore_ascii_case("UTF-8") {
                        return Err(fail(at, format!("SET {set}: only UTF-8 is read")));
                    }
                }
                "FLAG" => {
                    flags = match words.get(1).copied().unwrap_or_default() {
                        "UTF-8" => Flags::Utf8,
                        "long" => Flags::Long,
                        "num" => Flags::Num,
                        other => return Err(fail(at, format!("FLAG {other} is not a flag type"))),
                    }
                }
                "LANG" => {
                    // spellbook cases Turkic i and ı its own way, and this crate does not.
                    let lang = words.get(1).copied().unwrap_or_default();
                    if ["tr", "az", "crh"]
                        .iter()
                        .any(|turkic| lang.starts_with(turkic))
                    {
                        return Err(fail(
                            at,
                            format!("LANG {lang}: Turkic casing is not compiled"),
                        ));
                    }
                }
                "IGNORE" => {
                    lookup.ignore = words.get(1).copied().unwrap_or_default().chars().collect();
                }
                "ICONV" | "BREAK" => {
                    let count: usize = words
                        .get(1)
                        .and_then(|count| count.parse().ok())
                        .ok_or_else(|| fail(at, format!("{directive} with no row count")))?;
                    let mut rows = Vec::with_capacity(count);
                    for _ in 0..count {
                        let (at, row) = lines
                            .next()
                            .ok_or_else(|| fail(at, format!("{directive} has too few rows")))?;
                        let row: Vec<&str> = row.split_whitespace().collect();
                        if row.first() != Some(&directive) || row.len() < 2 {
                            return Err(fail(at, format!("not a {directive} row")));
                        }
                        rows.push(row);
                    }
                    if directive == "BREAK" {
                        breaks = Some(rows.iter().map(|row| row[1]).collect());
                    } else {
                        for row in rows {
                            let to = row
                                .get(2)
                                .ok_or_else(|| fail(at, "an ICONV row with one side".to_owned()))?;
                            lookup
                                .conversions
                                .push((row[1].to_owned(), (*to).to_owned()));
                        }
                    }
                }
                "SFX" => {
                    let count: usize = words
                        .get(3)
                        .and_then(|count| count.parse().ok())
                        .ok_or_else(|| fail(at, "an SFX header with no row count".to_owned()))?;
                    for _ in 0..count {
                        let (at, row) = lines
                            .next()
                            .ok_or_else(|| fail(at, "SFX has too few rows".to_owned()))?;
                        let row: Vec<&str> = row.split_whitespace().collect();
                        if row.first() != Some(&"SFX") || row.get(1) != words.get(1) {
                            return Err(fail(at, "not a row of the SFX above it".to_owned()));
                        }
                        suffix_lines.push((at, row));
                    }
                }
                _ => {}
            }
        }
        // The longest pattern is tried first at each place, as spellbook does.
        lookup
            .conversions
            .sort_by_key(|(from, _)| std::cmp::Reverse(from.len()));
        if let Some(breaks) = breaks {
            lookup.breaks = Breaks::new(&breaks);
        }

        let mut suffixes: HashMap<u64, Flagged> = HashMap::new();
        for (at, row) in suffix_lines {
            let [_, flag, strip, add, ..] = row[..] else {
                return Err(fail(
                    at,
                    "an SFX row with fewer than three fields".to_owned(),
                ));
            };
            let flag = parse_flag(flags, flag).map_err(|why| fail(at, why))?;
            if add.contains('/') {
                return Err(fail(
                    at,
                    format!(
                        "SFX {add}: a suffix that takes further affixes is not compiled; \
                         use `dictionary.hunspell`"
                    ),
                ));
            }
            let strip = if strip == "0" { "" } else { strip };
            let add: String = if add == "0" { "" } else { add }
                .chars()
                .filter(|c| !lookup.ignore.contains(c))
                .collect();
            let written = match row.get(4) {
                Some(&".") | None => "",
                Some(condition) => condition,
            };
            let flagged = suffixes.entry(flag).or_default();
            flagged.longest_strip = flagged.longest_strip.max(strip.len());
            match flagged
                .groups
                .iter_mut()
                .find(|group| group.strip == strip && group.written == written)
            {
                Some(group) => group.adds.push(add),
                None => flagged.groups.push(Suffixes {
                    strip: strip.to_owned(),
                    condition: Condition::parse(written).map_err(|why| fail(at, why))?,
                    written: written.to_owned(),
                    adds: vec![add],
                }),
            }
        }
        Ok(Aff {
            lookup,
            flags,
            suffixes,
        })
    }

    /// Calls `each` with every form `dic` makes from the stems `stem` keeps: each stem, and each
    /// stem with each suffix its flags allow, in the dictionary's order and with repeats. `stem` is
    /// given the most bytes a suffix of the stem's may strip: a stem longer than that by `n` bytes
    /// begins every form it makes with its own first `n` bytes.
    pub(crate) fn forms(
        &self,
        dic: &str,
        mut stem: impl FnMut(&str, usize) -> bool,
        mut each: impl FnMut(&str),
    ) -> Result<(), DictError> {
        let mut form = String::new();
        let mut flags = Vec::new();
        // The first line is the entry count.
        for (at, line) in dic.lines().enumerate().skip(1) {
            let (word, written) =
                entry(line).map_err(|why| DictError(format!(".dic line {}: {why}", at + 1)))?;
            let word: Cow<'_, str> = if word.contains(self.lookup.ignore.as_slice()) {
                Cow::Owned(
                    word.chars()
                        .filter(|c| !self.lookup.ignore.contains(c))
                        .collect(),
                )
            } else {
                Cow::Borrowed(word)
            };
            if word.is_empty() {
                continue;
            }
            flags.clear();
            parse_flags(self.flags, written, &mut flags)
                .map_err(|why| DictError(format!(".dic line {}: {why}", at + 1)))?;
            let reach = flags
                .iter()
                .filter_map(|flag| self.suffixes.get(flag))
                .map(|flagged| flagged.longest_strip)
                .max()
                .unwrap_or(0);
            if !stem(&word, reach) {
                continue;
            }
            each(&word);
            for flagged in flags.iter().filter_map(|flag| self.suffixes.get(flag)) {
                for group in &flagged.groups {
                    let Some(kept) = word.strip_suffix(group.strip.as_str()) else {
                        continue;
                    };
                    if !group.condition.matches_end(&word) {
                        continue;
                    }
                    for add in &group.adds {
                        form.clear();
                        form.push_str(kept);
                        form.push_str(add);
                        if !form.is_empty() {
                            each(&form);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// A `.dic` line's word and flags, the way spellbook splits the lines it reads quickly: the word
/// ends at the first `/` (its flags follow) or tab (nothing follows). A line spellbook reads the
/// slow way -- an escaped slash, a space, a word that starts with `/` -- is refused, because
/// reading it differently would accept other words.
fn entry(line: &str) -> Result<(&str, &str), String> {
    let line = line.trim_end_matches('\r');
    match line.find(['/', '\\', ' ', '\t']) {
        None => Ok((line, "")),
        Some(at) => match line.as_bytes()[at] {
            b'/' if at != 0 => {
                let rest = &line[at + 1..];
                Ok((
                    &line[..at],
                    rest.split_whitespace().next().unwrap_or_default(),
                ))
            }
            b'\t' => Ok((&line[..at], "")),
            _ => Err(format!(
                "{line:?} has a space, a backslash or a leading slash, which is not compiled"
            )),
        },
    }
}

/// An affix header's flag: the first of the flags `text` would be on a stem.
fn parse_flag(kind: Flags, text: &str) -> Result<u64, String> {
    match kind {
        Flags::Num => text
            .parse::<u16>()
            .map(u64::from)
            .map_err(|err| format!("flag {text:?}: {err}")),
        _ => {
            let mut flags = Vec::new();
            parse_flags(kind, text, &mut flags)?;
            flags
                .first()
                .copied()
                .ok_or_else(|| format!("flag {text:?} is too short"))
        }
    }
}

/// A stem's flags, into `out`.
fn parse_flags(kind: Flags, text: &str, out: &mut Vec<u64>) -> Result<(), String> {
    match kind {
        Flags::Short => out.extend(text.bytes().map(u64::from)),
        Flags::Utf8 => out.extend(text.encode_utf16().map(u64::from)),
        Flags::Long => {
            let bytes = text.as_bytes();
            if !bytes.len().is_multiple_of(2) {
                return Err(format!("{text:?} is not pairs of bytes"));
            }
            out.extend(
                bytes
                    .chunks(2)
                    .map(|pair| (u64::from(pair[0]) << 8) | u64::from(pair[1])),
            );
        }
        Flags::Num => {
            for flag in text.split(',').filter(|flag| !flag.is_empty()) {
                out.push(
                    flag.parse::<u16>()
                        .map(u64::from)
                        .map_err(|err| format!("flag {flag:?}: {err}"))?,
                );
            }
        }
    }
    Ok(())
}

impl Condition {
    fn parse(text: &str) -> Result<Condition, String> {
        let mut elements = Vec::new();
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            elements.push(match c {
                '.' => Element::Any,
                '[' => {
                    let mut set: Vec<char> = Vec::new();
                    let mut closed = false;
                    for c in chars.by_ref() {
                        if c == ']' {
                            closed = true;
                            break;
                        }
                        set.push(c);
                    }
                    if !closed {
                        return Err(format!("condition {text:?} has an open bracket"));
                    }
                    match set.split_first() {
                        Some(('^', rest)) => Element::NotIn(rest.to_vec()),
                        _ => Element::In(set),
                    }
                }
                c => Element::Char(c),
            });
        }
        Ok(Condition(elements))
    }

    /// Whether the end of `stem` matches: its last characters, one per element, in order.
    fn matches_end(&self, stem: &str) -> bool {
        let wanted = self.0.len();
        let start = match stem.char_indices().rev().nth(wanted.wrapping_sub(1)) {
            _ if wanted == 0 => return true,
            Some((at, _)) => at,
            None => return false,
        };
        stem[start..]
            .chars()
            .zip(&self.0)
            .all(|(c, element)| match element {
                Element::Any => true,
                Element::Char(wanted) => c == *wanted,
                Element::In(set) => set.contains(&c),
                Element::NotIn(set) => !set.contains(&c),
            })
    }
}
