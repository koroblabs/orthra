//! What a project and its user say about checking it: which languages, which words are words, and
//! which files are not prose.
//!
//! The layers (`docs/01-architecture.md` §9), most general first:
//!
//! - built in: English, either spelling (en-US and en-GB);
//! - the user's: `~/.config/orthra/config.toml` and `words.txt`;
//! - the project's: `orthra.toml` and `.orthra/words.txt`;
//! - the editor's, or the command line's: the languages it asks for.
//!
//! Languages and words are unions across the layers, except that a language a layer names replaces
//! the built-in variants of its own: `languages = ["en-US"]` is American only, `["uk-UA"]` keeps
//! both spellings of English. A file that cannot be read as it should is a problem said out loud,
//! never a layer quietly left out.

use std::{
    fs,
    path::{Path, PathBuf},
};

use lspkit_walk::{Override, OverrideBuilder};
use orthra_engine::Engine;

#[cfg(test)]
mod tests;

/// The project's settings file, at its root.
pub const PROJECT_FILE: &str = "orthra.toml";
/// The project's words, one per line, sorted, so that two branches adding a word merge.
pub const PROJECT_WORDS: &str = ".orthra/words.txt";

/// The languages a project is checked in until a layer names a variant of one of them: English,
/// either spelling. A project that says nothing has not said which, and `colour` in it is no typo.
/// On the projects beside orthra, a sixth of the words reported with en-US alone were British
/// (755 of 4 631, 2026-11-08).
const BUILT_IN: &[&str] = &["en-US", "en-GB"];

/// Directories of test data in every language, ignored before any layer's patterns: what a tool
/// printed, captured for a test (`fixtures/`, `api-fixtures/`), Jest's `__fixtures__/`, Go's
/// `testdata/`, and the `tests/data/` of Rust's and Python's projects. None of it is anybody's
/// prose, and some of it is misspelled on purpose. Whole files, not only their strings: captured
/// output in a `.txt` has no strings, it is all data. On the projects beside orthra, such
/// directories held 1 330 of the 3 876 words reported (2026-11-08); ripgrep's `tests/data/`, 93 of
/// its 230 on the zero corpus, all of them a Sherlock Holmes story searched in its tests
/// (2026-11-09). In the cargo registry, 28 crates' `tests/data/` hold keys, blobs and tables.
const BUILT_IN_IGNORE: &[&str] = &[
    "fixtures/",
    "*-fixtures/",
    "__fixtures__/",
    "testdata/",
    "**/tests/data/",
    "**/test/data/",
];

#[derive(Debug, Clone)]
pub struct Config {
    root: PathBuf,
    languages: Vec<String>,
    /// The built-in languages no layer has named a variant of yet: the ones a named variant
    /// replaces.
    defaults: Vec<String>,
    /// Where installed packs are looked for: [`Config::packs_dir`], unless set otherwise.
    packs: Option<PathBuf>,
    words: Vec<String>,
    ignore: Override,
    /// How strongly a finding is shown: `error`, `warning`, `information` or `hint`. The most
    /// specific layer that says wins.
    severity: Option<String>,
    /// Whether a language whose pack is installed rather than built in is fetched and installed
    /// when it is enabled and missing: `auto-install = false` keeps a machine off the network.
    auto_install: bool,
    problems: Vec<String>,
}

/// The severities a finding can be shown with.
pub const SEVERITIES: &[&str] = &["error", "warning", "information", "hint"];

impl Config {
    /// Built in only: English in either spelling, no words, and only test data ignored.
    pub fn builtin(root: &Path) -> Config {
        let defaults: Vec<String> = BUILT_IN.iter().map(|&tag| tag.to_owned()).collect();
        let mut config = Config {
            root: root.to_path_buf(),
            languages: defaults.clone(),
            defaults,
            packs: Config::packs_dir(),
            words: Vec::new(),
            ignore: Override::empty(),
            severity: None,
            auto_install: true,
            problems: Vec::new(),
        };
        config.set_ignore(&[]);
        config
    }

    /// Every layer for the project at `root`: built in, the user's in `user` when one is given,
    /// and the project's.
    pub fn load(root: &Path, user: Option<&Path>) -> Config {
        let mut config = Config::builtin(root);
        let mut patterns = Vec::new();
        if let Some(user) = user {
            config.read_settings(&user.join("config.toml"), &mut patterns);
            config.read_words(&user.join("words.txt"));
        }
        config.read_settings(&root.join(PROJECT_FILE), &mut patterns);
        config.read_words(&root.join(PROJECT_WORDS));
        config.set_ignore(&patterns);
        config
    }

    /// What is ignored: the built-in test data, then `patterns`, the layers'.
    fn set_ignore(&mut self, patterns: &[String]) {
        let mut ignore = OverrideBuilder::new(&self.root);
        let patterns = BUILT_IN_IGNORE
            .iter()
            .copied()
            .chain(patterns.iter().map(String::as_str));
        for pattern in patterns {
            // An override is a whitelist; `!` makes it an ignore.
            if let Err(err) = ignore.add(&format!("!{pattern}")) {
                self.problems.push(format!("ignore = {pattern:?}: {err}"));
            }
        }
        match ignore.build() {
            Ok(ignore) => self.ignore = ignore,
            Err(err) => self.problems.push(format!("ignore: {err}")),
        }
    }

    /// The project's words file: where "add to the project's words" writes.
    pub fn project_words(&self) -> PathBuf {
        self.root.join(PROJECT_WORDS)
    }

    /// The user's words file in `user`: where "add to your words" writes.
    pub fn user_words(user: &Path) -> PathBuf {
        user.join("words.txt")
    }

    /// The files a server watches, because a change to any of them changes the findings.
    pub fn watched(&self, user: Option<&Path>) -> Vec<PathBuf> {
        let mut files = vec![self.root.join(PROJECT_FILE), self.project_words()];
        if let Some(user) = user {
            files.push(user.join("config.toml"));
            files.push(Config::user_words(user));
        }
        files
    }

    /// The user's directory: `$XDG_CONFIG_HOME/orthra`, or `~/.config/orthra`.
    pub fn user_dir() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|dir| dir.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(base.join("orthra"))
    }

    /// Where installed packs live: `$XDG_DATA_HOME/orthra/packs`, or `~/.local/share/orthra/packs`.
    pub fn packs_dir() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|dir| dir.is_absolute())
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })?;
        Some(base.join("orthra").join("packs"))
    }

    /// Where installed models live: `$XDG_DATA_HOME/orthra/models`, beside the packs.
    pub fn models_dir() -> Option<PathBuf> {
        Some(Config::packs_dir()?.parent()?.join("models"))
    }

    /// Whether a missing pack is fetched and installed: `auto-install`, true unless a settings file
    /// says otherwise.
    pub fn auto_install(&self) -> bool {
        self.auto_install
    }

    /// Looks for installed packs in `packs` instead: a test's own directory, never the person's.
    pub fn set_packs(&mut self, packs: Option<PathBuf>) {
        self.packs = packs;
    }

    /// Where installed packs are looked for.
    pub fn packs(&self) -> Option<&Path> {
        self.packs.as_deref()
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The languages to check with, built in first, each once.
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// Adds languages a later layer asks for: the editor's, or `--lang`. A tag replaces the built-in
    /// variants of its language that no layer has named yet, so `--lang en-US` checks as en-US
    /// alone, and `--lang uk-UA` adds Ukrainian to both spellings of English.
    pub fn add_languages(&mut self, tags: impl IntoIterator<Item = String>) {
        for tag in tags {
            let (replaced, kept): (Vec<String>, Vec<String>) = std::mem::take(&mut self.defaults)
                .into_iter()
                .partition(|default| same_language(default, &tag));
            self.defaults = kept;
            self.languages.retain(|known| !replaced.contains(known));
            if !self.languages.contains(&tag) {
                self.languages.push(tag);
            }
        }
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// How strongly a finding is shown, `warning` when no layer says: Zed lists a warning among
    /// the project's problems, and shows an information only where it is written.
    pub fn severity(&self) -> &str {
        self.severity.as_deref().unwrap_or("warning")
    }

    /// The severity a later layer asks for: the editor's.
    pub fn set_severity(&mut self, severity: &str) {
        if SEVERITIES.contains(&severity) {
            self.severity = Some(severity.to_owned());
        } else {
            self.problems.push(format!(
                "severity {severity:?} is not one of {SEVERITIES:?}"
            ));
        }
    }

    /// What went wrong reading the layers, one sentence each, naming the file.
    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    /// Whether `path` is one of the files the project says are not prose: named itself, or inside
    /// a directory that is. The walk prunes such a directory; an editor opens the file anyway.
    pub fn is_ignored(&self, path: &Path) -> bool {
        if self.ignore.matched(path, path.is_dir()).is_ignore() {
            return true;
        }
        path.ancestors()
            .skip(1)
            .take_while(|dir| dir.starts_with(&self.root) && *dir != self.root)
            .any(|dir| self.ignore.matched(dir, true).is_ignore())
    }

    /// The matcher [`Self::is_ignored`] uses, for a walk to prune with.
    pub(crate) fn ignore(&self) -> &Override {
        &self.ignore
    }

    /// An engine for these languages, knowing these words.
    pub fn engine(&self) -> Result<Engine, String> {
        let tags: Vec<&str> = self.languages.iter().map(String::as_str).collect();
        let mut engine =
            Engine::with_installed(&tags, self.packs.as_deref()).map_err(|err| err.0)?;
        engine.set_words(self.words.iter().map(String::as_str));
        Ok(engine)
    }

    /// `languages = [...]` and `ignore = [...]` from a settings file, if there is one.
    fn read_settings(&mut self, file: &Path, patterns: &mut Vec<String>) {
        let Ok(text) = fs::read_to_string(file) else {
            return;
        };
        let shown = file.display();
        let table: toml::Table = match text.parse() {
            Ok(table) => table,
            Err(err) => {
                self.problems.push(format!("{shown}: {err}"));
                return;
            }
        };
        for (key, value) in &table {
            if key == "auto-install" {
                match value.as_bool() {
                    Some(auto) => self.auto_install = auto,
                    None => self
                        .problems
                        .push(format!("{shown}: auto-install is true or false")),
                }
                continue;
            }
            if key == "severity" {
                match value.as_str() {
                    Some(severity) if SEVERITIES.contains(&severity) => {
                        self.severity = Some(severity.to_owned());
                    }
                    _ => self
                        .problems
                        .push(format!("{shown}: severity is one of {SEVERITIES:?}")),
                }
                continue;
            }
            let Some(list) = value.as_array().and_then(|list| {
                list.iter()
                    .map(|item| item.as_str().map(str::to_owned))
                    .collect::<Option<Vec<String>>>()
            }) else {
                self.problems
                    .push(format!("{shown}: {key} is not a list of strings"));
                continue;
            };
            match key.as_str() {
                "languages" => self.add_languages(list),
                "ignore" => patterns.extend(list),
                _ => self
                    .problems
                    .push(format!("{shown}: {key} is not a setting")),
            }
        }
    }

    /// A words file's words: one per line, `#` starting a comment.
    fn read_words(&mut self, file: &Path) {
        let Some(text) = words_text(file) else {
            return;
        };
        self.words.extend(
            text.lines()
                .map(entry)
                .filter(|word| !word.is_empty())
                .map(str::to_owned),
        );
    }
}

/// A words file's text, without the byte-order mark some editors write first: read with it, the
/// first word was `\u{feff}word`, and matched nothing.
fn words_text(file: &Path) -> Option<String> {
    let text = fs::read_to_string(file).ok()?;
    Some(match text.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_owned(),
        None => text,
    })
}

/// The word a words file's line holds, without its comment: `phrost  # the sibling` is `phrost`.
fn entry(line: &str) -> &str {
    line.split('#').next().unwrap_or_default().trim()
}

/// Whether two language tags name the same language, whatever their region or script: their
/// first subtags, `en` in `en-US` and `en_GB`.
fn same_language(a: &str, b: &str) -> bool {
    fn language(tag: &str) -> &str {
        tag.split(['-', '_']).next().unwrap_or_default()
    }
    language(a).eq_ignore_ascii_case(language(b))
}

/// Adds `word` to the words file `file`, which is created if it is not there: one word per line,
/// sorted ignoring case, each once, so that two branches that each add a word merge without a
/// conflict. Comment lines stay at the top. A word already there, in any case, is left as it is.
pub fn add_word(file: &Path, word: &str) -> std::io::Result<()> {
    let text = words_text(file).unwrap_or_default();
    let (mut comments, mut words) = (Vec::new(), Vec::new());
    for line in text.lines() {
        if line.trim_start().starts_with('#') {
            comments.push(line.to_owned());
        } else if !line.trim().is_empty() {
            words.push(line.trim().to_owned());
        }
    }
    // A line is the word it holds, as the check reads it: `phrost  # the sibling` is `phrost`.
    let lower = word.to_lowercase();
    if words
        .iter()
        .any(|known| entry(known).to_lowercase() == lower)
    {
        return Ok(());
    }
    words.push(word.to_owned());
    words.sort_by_key(|line| entry(line).to_lowercase());
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut out = String::new();
    for line in comments.iter().chain(&words) {
        out.push_str(line);
        out.push('\n');
    }
    fs::write(file, out)
}
