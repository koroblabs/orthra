//! Where prose lives in a file: comments, strings, declarations, markup text.
//!
//! A host knows one programming or markup language, and says which ranges of a file are prose and
//! of which kind. It knows nothing about natural languages; that is a pack's job, and the two are
//! orthogonal (`docs/01-architecture.md` §3). Which host reads a file is a table, decided once from
//! its language id or its path. A file no host claims is not checked: silence, not a guess.

use std::fmt;

use orthra_text::TextRange;

use crate::parsed::Parsed;

mod c;
mod cmake;
mod cpp;
mod csharp;
mod css;
mod dart;
mod dockerfile;
mod elixir;
mod elm;
mod erlang;
mod fsharp;
pub mod git_commit;
mod go;
mod graphql;
mod groovy;
mod haskell;
mod html;
mod java;
mod javascript;
mod json;
mod julia;
mod kotlin;
mod less;
mod lua;
mod make;
pub mod markdown;
mod objc;
mod ocaml;
pub mod parsed;
mod php;
pub mod plain_text;
mod powershell;
mod proto;
mod python;
mod r;
mod rst;
mod ruby;
mod rust;
mod scala;
mod scss;
mod shell;
mod sql;
mod svelte;
mod swift;
pub mod test_files;
mod toml;
mod tree_sitter_host;
mod tsx;
mod typescript;
mod vue;
mod xml;
mod yaml;
mod zig;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub range: TextRange,
    pub kind: SpanKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// Text meant to be read: a paragraph, a commit message.
    Prose,
    Comment,
    DocComment,
    String,
    /// A name being declared: the one place an identifier is the author's to fix.
    Declaration,
    /// A value that is code, not text, and so is held to known typos: a string of one token in a
    /// format whose strings are values (`"derive"`, `"source.rust"`), a CI step's shell command.
    Value,
}

/// Facts about a host's text, not behavior.
#[derive(Debug, Clone, Copy, Default)]
pub struct Conventions {
    /// Whether `\n`, `\t` and `\u{e9}` are escapes rather than letters.
    pub backslash_escapes: bool,
    /// Whether a doubled backslash before a letter is an escape too: a shell script writes
    /// `"\\nexport"` for printf and `echo -e`, which read it as a newline and `export`. Elsewhere
    /// `"C:\\network"` is a path, and the `n` is the word's.
    pub doubled_escapes: bool,
    /// What starts a comment that runs to the end of the line: `//`, `#`. `None` for prose, and for
    /// a format that has none (JSON).
    pub line_comment: Option<&'static str>,
    /// What opens and closes a comment, for a language with no line comment: Markdown's `<!--`
    /// and `-->`.
    pub block_comment: Option<(&'static str, &'static str)>,
}

pub trait Host: Send + Sync + fmt::Debug {
    fn id(&self) -> &'static str;
    /// LSP language ids this host answers for.
    fn language_ids(&self) -> &'static [&'static str];
    /// File names and extensions this host claims when there is no language id: `COMMIT_EDITMSG`, `md`.
    /// A name may hold `*` for any run of characters: `LICENSE-*`.
    fn file_names(&self) -> &'static [&'static str];
    fn extensions(&self) -> &'static [&'static str];
    fn conventions(&self) -> Conventions;
    fn spans(&self, text: &str, out: &mut Vec<Span>);

    /// [`Host::spans`], parsing with what `parsed` kept of the document's last check, and keeping
    /// in it what the next check can use. The spans are always the ones `spans` gives for the same
    /// text: what is kept saves work and never changes an answer (`CLAUDE.md` §1.2).
    fn spans_reusing(&self, text: &str, parsed: &mut Parsed, out: &mut Vec<Span>) {
        parsed.clear();
        self.spans(text, out);
    }

    /// Whether this host can read its language: `Err` says why not. A host that cannot yields no
    /// spans, and fails alone.
    /// Words of this language that are words in its files, whatever a dictionary says: `async`,
    /// `usize`, `serde` in Rust. One per line; `#` starts a comment.
    fn vocabulary(&self) -> &'static str {
        ""
    }

    /// Where this language keeps its tests, beyond the `tests` and `test` directories every
    /// language uses: a pattern ending in `/` is a directory anywhere in the path, and any other is
    /// a file name, where `*` stands for any run of characters. See [`test_files`].
    fn test_files(&self) -> &'static [&'static str] {
        &[]
    }

    /// File names this host's language uses for files a tool writes, where `*` stands for any run
    /// of characters: `package-lock.json`, `*.min.js`, `*.pb.go`. Nobody fixes a typo in one, and
    /// its words are other people's names or a minifier's, so no host checks it. See
    /// [`test_files::is_generated`].
    fn generated_files(&self) -> &'static [&'static str] {
        &[]
    }

    fn health(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Hosts {
    hosts: Vec<Box<dyn Host>>,
}

impl Hosts {
    pub fn builtin() -> Hosts {
        Hosts {
            hosts: vec![
                Box::new(plain_text::PlainText),
                Box::new(markdown::Markdown),
                Box::new(git_commit::GitCommit),
                Box::new(&rust::HOST),
                Box::new(&python::HOST),
                Box::new(&go::HOST),
                Box::new(&shell::HOST),
                Box::new(&php::HOST),
                Box::new(&toml::HOST),
                Box::new(&yaml::HOST),
                Box::new(&sql::HOST),
                Box::new(&json::HOST),
                Box::new(&css::HOST),
                Box::new(&html::HOST),
                Box::new(&typescript::HOST),
                Box::new(&tsx::HOST),
                Box::new(&javascript::HOST),
                Box::new(&lua::HOST),
                Box::new(&scss::HOST),
                Box::new(&less::HOST),
                Box::new(&kotlin::HOST),
                Box::new(&java::HOST),
                Box::new(&swift::HOST),
                Box::new(&c::HOST),
                Box::new(&cpp::HOST),
                Box::new(&ruby::HOST),
                Box::new(&vue::HOST),
                Box::new(&svelte::HOST),
                Box::new(&scala::HOST),
                Box::new(&groovy::HOST),
                Box::new(&csharp::HOST),
                Box::new(&fsharp::HOST),
                Box::new(&dart::HOST),
                Box::new(&zig::HOST),
                Box::new(&objc::HOST),
                Box::new(&haskell::HOST),
                Box::new(&ocaml::HOST),
                Box::new(&ocaml::INTERFACE),
                Box::new(&elm::HOST),
                Box::new(&elixir::HOST),
                Box::new(&erlang::HOST),
                Box::new(&r::HOST),
                Box::new(&julia::HOST),
                Box::new(&powershell::HOST),
                Box::new(&make::HOST),
                Box::new(&cmake::HOST),
                Box::new(&dockerfile::HOST),
                Box::new(&graphql::HOST),
                Box::new(&proto::HOST),
                Box::new(&xml::HOST),
                Box::new(&rst::HOST),
            ],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Host> {
        self.hosts.iter().map(|host| &**host)
    }

    pub fn for_language(&self, id: &str) -> Option<&dyn Host> {
        self.hosts
            .iter()
            .find(|host| host.language_ids().contains(&id))
            .map(|host| &**host)
    }

    /// The host for a path, by its file name, then by its extension. A file the host names as
    /// generated has none.
    pub fn for_path(&self, path: &str) -> Option<&dyn Host> {
        let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
        let extension = name
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase());
        self.hosts
            .iter()
            .find(|host| test_files::names_match(host.file_names(), name))
            .or_else(|| {
                let extension = extension.as_deref()?;
                self.hosts
                    .iter()
                    .find(|host| host.extensions().contains(&extension))
            })
            .map(|host| &**host)
            .filter(|host| !test_files::is_generated(*host, path))
    }
}
