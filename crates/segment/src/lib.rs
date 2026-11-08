//! Spans to words: what in a range of prose is a word to check, and what never is.
//!
//! A word is reported to the packs only if a dictionary could judge it. Every rule below exists
//! because a case in `corpora/incumbent/` or a finding on the zero corpus showed the alternative
//! to be noise:
//!
//! - **Whole chunks that are never words**: URLs, email addresses, paths, hashes, UUIDs, colors,
//!   command-line flags, qualified names (`os.path.join`, `Vec::new`), and code between backticks.
//! - **Escapes end words.** In `"Hello\nworld"` the words are `Hello` and `world`, never `nworld`.
//! - **Letters glued to a digit are a token, not a word**: `100ms`, `utf8`, `x86`, `k8s`, `v2`.
//! - **Identifiers split** at `_` and at case changes, keeping an acronym's plural whole:
//!   `getURLsForIDs` is `get`, `URLs`, `For`, `IDs`.
//! - **`'s` is dropped**, so `laravel's` is judged as `laravel` and an unknown name is reported once,
//!   not twice (13 % of codebook's findings on phrost).
//! - **Acronyms of up to five capitals and words of fewer than three letters are not checked.**

use orthra_text::{TextRange, TextSize, script::Script};

#[cfg(test)]
mod tests;

/// A word to check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word {
    pub range: TextRange,
    pub script: Script,
    pub shape: Shape,
    /// Code rather than prose: a part of an identifier (`recieve` in `recieve_message`), or a word
    /// between backticks in prose (`` `qzxRender()` ``). Code is checked for known typos only
    /// (`docs/03-roadmap.md` open question 7): an unknown word in code is usually a name somebody
    /// chose, and a known typo is a typo wherever it is written.
    pub code: bool,
    /// The run of letters the word was cut from: `PostgreSQL` for `Postgre`. Equal to `range` when
    /// nothing was cut. A run a pack accepts whole is accepted in all its parts.
    pub run: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Lower,
    Title,
    Upper,
    Mixed,
}

impl Shape {
    fn of(word: &str) -> Shape {
        let mut chars = word.chars().filter(|c| c.is_alphabetic());
        let first_upper = chars.next().is_some_and(char::is_uppercase);
        let (mut upper, mut lower) = (0, 0);
        for c in chars {
            if c.is_uppercase() {
                upper += 1;
            } else {
                lower += 1;
            }
        }
        match (first_upper, upper, lower) {
            (false, 0, _) => Shape::Lower,
            (true, 0, _) => Shape::Title,
            (true, _, 0) => Shape::Upper,
            _ => Shape::Mixed,
        }
    }
}

/// What a host and the enabled packs say about how words are written.
#[derive(Debug, Clone, Copy)]
pub struct Rules<'a> {
    /// Characters that stay inside a word when letters of the script beside each stand on both
    /// sides: apostrophes, and a hyphen in Ukrainian.
    pub word_internal: &'a [(char, Script)],
    /// Whether `\n`, `\t` and `\u{e9}` are escapes in this span.
    pub backslash_escapes: bool,
    /// Whether `\\n` is an escape as well as `\n`: the host's strings are read twice, by the shell
    /// and then by printf.
    pub doubled_escapes: bool,
}

impl Rules<'_> {
    /// Whether `c` is inside a word after the letter `before`: a pack of `before`'s script says so.
    fn joins(&self, c: char, before: char) -> bool {
        let Some(script) = Script::of_char(before) else {
            return false;
        };
        self.word_internal
            .iter()
            .any(|(internal, of)| *internal == c && *of == script)
    }
}

/// The words of every span in `text`, in order.
///
/// All of a document's spans go through one call, so the buffers below are allocated once per
/// document: segmentation is most of what a check costs, and an allocation per span or per chunk
/// was most of segmentation (measured 2026-11-05: 102 ms to under half of that on the zero corpus).
pub fn words(
    text: &str,
    spans: impl IntoIterator<Item = TextRange>,
    rules: &Rules<'_>,
    out: &mut Vec<Word>,
) {
    let mut scratch = Scratch::default();
    for span in spans {
        let base = usize::from(span.start());
        let slice = &text[span];
        if !slice.contains('`') {
            scratch.chunks(slice, base, rules, out);
            continue;
        }
        let mut from = 0;
        let mut search = 0;
        // A pair of backticks on one line holds code, not prose: `qzxRender()`.
        while let Some(open) = slice[search..].find('`').map(|i| search + i) {
            let line_end = slice[open..].find('\n').map_or(slice.len(), |i| open + i);
            match slice[open + 1..line_end].find('`') {
                Some(close) => {
                    scratch.chunks(&slice[from..open], base + from, rules, out);
                    // Code between a pair of backticks is segmented, and marked as code.
                    let first = out.len();
                    scratch.chunks(
                        &slice[open + 1..open + 1 + close],
                        base + open + 1,
                        rules,
                        out,
                    );
                    for word in &mut out[first..] {
                        word.code = true;
                    }
                    from = open + 1 + close + 1;
                    search = from;
                }
                None => search = open + 1,
            }
        }
        scratch.chunks(&slice[from..], base + from, rules, out);
    }
}

/// Buffers every chunk of a document shares.
#[derive(Debug, Default)]
struct Scratch {
    chars: Vec<(usize, char)>,
    cuts: Vec<usize>,
}

impl Scratch {
    /// The words of the whitespace-separated chunks of `slice`, which starts at `base`.
    fn chunks(&mut self, slice: &str, base: usize, rules: &Rules<'_>, out: &mut Vec<Word>) {
        let mut start = None;
        for (i, c) in slice.char_indices().chain([(slice.len(), ' ')]) {
            match (start, c.is_whitespace()) {
                (None, false) => start = Some(i),
                (Some(s), true) => {
                    start = None;
                    let (lead, core) = trim_punctuation(&slice[s..i]);
                    // A chunk that one dot starts is a file's name or its extension:
                    // `.gitattributes`, `(.rgignore)`, `` `.gir` ``. An ellipsis is not one.
                    let dotted =
                        slice[s..s + lead].ends_with('.') && !slice[s..s + lead].ends_with("..");
                    if core.is_empty() || dotted || never_a_word(core) {
                        continue;
                    }
                    Splitter {
                        text: core,
                        base: base + s + lead,
                        rules,
                        out,
                        scratch: self,
                    }
                    .run();
                }
                _ => {}
            }
        }
    }
}

/// The chunk without the punctuation around it, and how many bytes were cut from its front.
fn trim_punctuation(chunk: &str) -> (usize, &str) {
    let is_edge = |c: char| "()[]{}<>\"'`,.;:!?*_~“”‘’«»".contains(c);
    let core = chunk.trim_start_matches(is_edge);
    let lead = chunk.len() - core.len();
    (lead, core.trim_end_matches(is_edge))
}

/// Chunks that are never words, whatever letters they hold.
fn never_a_word(chunk: &str) -> bool {
    // Most chunks are plain words, and a chunk of letters alone is none of the shapes below.
    if chunk.bytes().all(|b| b.is_ascii_alphabetic() || b == b'\'') {
        return false;
    }
    let hex = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit());
    // A glob is a pattern, not words: `*.rs`, `[Mm]akefile`, `src/**`, `*.{c,h}`.
    let glob = chunk.contains('*')
        || chunk.contains('?')
            && chunk.chars().any(|c| c.is_alphanumeric())
            && !chunk.ends_with('?')
        || chunk
            .find('[')
            .is_some_and(|open| chunk[open..].contains(']'));
    glob || chunk.contains("://")
        || chunk.starts_with("www.")
        || (chunk.contains('@')
            && chunk
                .rsplit('@')
                .next()
                .is_some_and(|host| host.contains('.')))
        // A path, and also words a slash joins: reading `Inflate/Deflate` as two words was
        // measured (2026-11-06) and found 5 to 8 of the typo sample's 1 614 non-word typos for 31
        // more findings on the zero corpus, almost all `owner/repo` names (`nikic/php-parser`,
        // `sigstore/fulcio`), which prose writes the same way.
        || chunk.contains('/')
        || qualified(chunk)
        || chunk.contains(":\\")
        || locale(chunk)
        || chunk.starts_with('-') && chunk.len() > 1
        || chunk.starts_with('#') && hex(&chunk[1..])
        || chunk.len() >= 7 && hex(chunk) && chunk.chars().any(|c| c.is_ascii_digit())
        || chunk.split('-').all(hex) && chunk.matches('-').count() == 4
}

/// A name qualified by its module, package or type, which refers to code and is not prose: three or
/// more names joined by dots (`os.path.join`, `google.longrunning.Operations`), or two or more by
/// `::` (`Vec::new`, `std::io::Error`). Two joined by a dot are not, since a sentence missing its
/// space reads the same (`done.Then`). A comment that names somebody else's API is not where it is
/// misspelled (`CLAUDE.md` §5).
fn qualified(chunk: &str) -> bool {
    let name = |part: &str| {
        part.starts_with(|c: char| c.is_alphabetic() || c == '_')
            && part.chars().all(|c| c.is_alphanumeric() || c == '_')
    };
    chunk.matches('.').count() >= 2 && chunk.split('.').all(name)
        || chunk.contains("::") && chunk.split("::").all(name)
}

/// A language or locale tag, which names a language and is not a word in it: `uk-UA`, `en_US`,
/// `es-419`, `zh-Hant-TW`. A language of two or three lower-case letters and a region (`UA`, `419`),
/// a script (`Hant`) between them if any, joined by `-` or `_`. `uk-UA` read as words is `uk`,
/// which no dictionary has in lower case. A script alone after a word is prose, not a tag:
/// `non-Java`.
fn locale(chunk: &str) -> bool {
    let parts: Vec<&str> = chunk.split(['-', '_']).collect();
    let language =
        |part: &str| (2..=3).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_lowercase());
    let script = |part: &str| {
        let bytes = part.as_bytes();
        bytes.len() == 4
            && bytes[0].is_ascii_uppercase()
            && bytes[1..].iter().all(u8::is_ascii_lowercase)
    };
    let region = |part: &str| {
        let bytes = part.as_bytes();
        bytes.len() == 2 && bytes.iter().all(u8::is_ascii_uppercase)
            || bytes.len() == 3 && bytes.iter().all(u8::is_ascii_digit)
    };
    match parts[..] {
        [first, last] => language(first) && region(last),
        [first, middle, last] => language(first) && script(middle) && region(last),
        _ => false,
    }
}

/// Splits one chunk into words.
struct Splitter<'t, 'o, 'r, 's> {
    text: &'t str,
    base: usize,
    rules: &'r Rules<'r>,
    out: &'o mut Vec<Word>,
    scratch: &'s mut Scratch,
}

impl Splitter<'_, '_, '_, '_> {
    fn run(mut self) {
        let identifier =
            self.text.contains('_') || self.text.contains("::") || self.text.contains("()");
        let mut chars = std::mem::take(&mut self.scratch.chars);
        // The common chunk is one plain word of ASCII letters: one run, no digits, no escapes, and
        // nothing to decode. It skips the loop below and gives the same words.
        if self.text.bytes().all(|b| b.is_ascii_alphabetic()) {
            let mut cuts = std::mem::take(&mut self.scratch.cuts);
            camel_cuts(self.text, &mut cuts);
            let code = cuts.len() > 2;
            for window in cuts.windows(2) {
                self.push(window[0], window[1], (0, self.text.len()), code);
            }
            self.scratch.cuts = cuts;
            self.scratch.chars = chars;
            return;
        }
        chars.clear();
        chars.extend(self.text.char_indices());
        let mut i = 0;
        while i < chars.len() {
            let (start, c) = chars[i];
            if c == '\\' && self.rules.backslash_escapes {
                // The escape's letter is part of the escape: skip it with the backslash, and with
                // the doubled backslash before it where the host's strings are read twice.
                let doubled = self.rules.doubled_escapes
                    && chars.get(i + 1).is_some_and(|(_, next)| *next == '\\')
                    && chars
                        .get(i + 2)
                        .is_some_and(|(_, letter)| letter.is_alphabetic());
                i += if doubled { 3 } else { 2 };
                continue;
            }
            if !c.is_alphanumeric() {
                i += 1;
                continue;
            }
            // A run of letters and digits, keeping a word-internal character between two letters,
            // and ending where a script that looks like Latin meets one that does not.
            let mut confusable = Script::is_confusable(c);
            let mut j = i + 1;
            while j < chars.len() {
                let c = chars[j].1;
                // An escaped apostrophe between two letters is an apostrophe: `didn\'t` in a
                // single-quoted string. The engine reads the word without the backslash.
                let escaped = c == '\\'
                    && chars[j - 1].1.is_alphabetic()
                    && chars
                        .get(j + 1)
                        .is_some_and(|(_, next)| self.rules.joins(*next, chars[j - 1].1))
                    && chars
                        .get(j + 2)
                        .is_some_and(|(_, after)| after.is_alphabetic());
                if escaped {
                    j += 2;
                    continue;
                }
                let internal = self.rules.joins(c, chars[j - 1].1)
                    && chars[j - 1].1.is_alphabetic()
                    && chars
                        .get(j + 1)
                        .is_some_and(|(_, next)| next.is_alphabetic());
                if !(c.is_alphanumeric() || internal) {
                    break;
                }
                if !c.is_ascii() || confusable == Some(false) {
                    let here = Script::is_confusable(c);
                    if here.is_some() && confusable.is_some() && here != confusable {
                        break;
                    }
                    confusable = confusable.or(here);
                }
                j += 1;
            }
            let end = chars.get(j).map_or(self.text.len(), |(at, _)| *at);
            let run = &self.text[start..end];
            // A short run right after a dot that follows a name is a file extension or a field:
            // `services.yml`, `notes.txt`, `self.name`. Not a word to check.
            let extension = i >= 2
                && chars[i - 1].1 == '.'
                && chars[i - 2].1.is_alphanumeric()
                && run.chars().count() <= 4;
            // A run glued to `@` is a handle, a decorator or a marker, never prose: `@user`,
            // `@Override`, the `fi@name@rst` of a test fixture.
            let at_sign =
                (i >= 1 && chars[i - 1].1 == '@') || chars.get(j).is_some_and(|(_, c)| *c == '@');
            if !extension && !at_sign && !run.bytes().any(|b| b.is_ascii_digit()) {
                let mut cuts = std::mem::take(&mut self.scratch.cuts);
                camel_cuts(run, &mut cuts);
                let code = identifier || cuts.len() > 2;
                for window in cuts.windows(2) {
                    self.push(start + window[0], start + window[1], (start, end), code);
                }
                self.scratch.cuts = cuts;
            }
            i = j;
        }
        self.scratch.chars = chars;
    }

    fn push(&mut self, from: usize, to: usize, run: (usize, usize), code: bool) {
        let mut word = &self.text[from..to];
        for possessive in ["\\'s", "\\’s", "'s", "’s", "'S"] {
            if let Some(base) = word.strip_suffix(possessive) {
                word = base;
            }
        }
        let shape = Shape::of(word);
        let letters = word.chars().filter(|c| c.is_alphabetic()).count();
        let acronym = matches!(shape, Shape::Upper) && letters <= 5
            || matches!(shape, Shape::Mixed)
                && word.ends_with('s')
                && word[..word.len() - 1].chars().all(char::is_uppercase)
                && letters <= 6;
        if letters < 3 || acronym {
            return;
        }
        let size = |at: usize| TextSize::try_from(self.base + at).expect("a document under 4 GiB");
        self.out.push(Word {
            range: TextRange::new(size(from), size(from + word.len())),
            script: Script::of(word),
            shape,
            code,
            run: TextRange::new(size(run.0), size(run.1)),
        });
    }
}

/// Where a camelCase or PascalCase run is cut, as byte offsets from 0 to its length; the parts are
/// the windows between cuts. An upper-case run followed by a lower-case letter ends one letter early
/// (`HTTPServer` is `HTTP` + `Server`), unless the lower-case letter is a lone plural `s` (`URLs`).
fn camel_cuts(run: &str, cuts: &mut Vec<usize>) {
    cuts.clear();
    cuts.push(0);
    // Most runs are lower or title case, and have no cut to find.
    if !run.chars().skip(1).any(char::is_uppercase) {
        cuts.push(run.len());
        return;
    }
    let mut chars = run.char_indices().peekable();
    let mut prev: Option<char> = None;
    while let Some((at, c)) = chars.next() {
        if let Some(prev) = prev {
            let mut ahead = chars.clone();
            let next = ahead.next().map(|(_, n)| n);
            let after_next = ahead.next().map(|(_, n)| n);
            let lower_to_upper = prev.is_lowercase() && c.is_uppercase();
            let acronym_to_word = prev.is_uppercase()
                && c.is_uppercase()
                && next.is_some_and(char::is_lowercase)
                && !(next == Some('s') && after_next.is_none_or(char::is_uppercase));
            if lower_to_upper || acronym_to_word {
                cuts.push(at);
            }
        }
        prev = Some(c);
    }
    cuts.push(run.len());
    cuts.dedup();
}
