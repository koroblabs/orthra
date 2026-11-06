//! A workspace written as one string, in lspkit's fixture format, which phrost writes too.
//!
//! ```text
//! //- /orthra.toml
//! languages = ["en-US"]
//!
//! //- /src/lib.rs
//! /// Retruns the fi$0rst item.
//! ```
//!
//! The format is `lspkit_fixture`'s: `//- /path flags` headers with the slash, `$0` and `@name@`
//! markers cut from the text, a backslash before either making it text, indentation trimmed, each
//! file ending in one newline, and **the subject is the last file**. Malformed input is a panic
//! there, with the same messages for both servers.
//!
//! What the flags mean is orthra's:
//! - `host=<id>` checks the file with that host instead of the one its path implies;
//! - `raw` (the kit's) leaves markers in the text, for a host whose own syntax collides with them;
//! - `expect` says the file is written as an annotated rendering (`annotate`), so its text and its
//!   expected findings are one artifact. Its text is left as written, since cutting a marker would
//!   move every caret after it.
//!
//! Any other flag is a panic: a typo has to stop the test, not quietly build another workspace.

use lspkit_fixture::Markers;

use crate::annotate::Annotation;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Fixture {
    files: Vec<FixtureFile>,
    markers: Markers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureFile {
    pub path: String,
    pub text: String,
    pub host: Option<String>,
    pub raw: bool,
    /// `expect`: the file was written as an annotated rendering, and these are its annotations;
    /// `text` is the text it renders.
    pub expected: Option<Vec<Annotation>>,
}

impl Fixture {
    pub fn parse(source: &str) -> Fixture {
        let (files, markers) =
            lspkit_fixture::Fixture::parse_with(source, &["expect"]).into_parts();
        Fixture {
            files: files.iter().map(FixtureFile::new).collect(),
            markers,
        }
    }

    pub fn files(&self) -> &[FixtureFile] {
        &self.files
    }

    /// The file every check reads: the last one.
    pub fn subject(&self) -> &FixtureFile {
        self.files
            .last()
            .expect("parsing refuses a fixture with no files")
    }

    /// The file at `path`, spelled as its header spells it: from the root, with the slash.
    pub fn file(&self, path: &str) -> &FixtureFile {
        assert!(
            path.starts_with('/'),
            "fixture paths start at the root, with the slash: {path:?}"
        );
        self.files
            .iter()
            .find(|file| file.path == path)
            .unwrap_or_else(|| panic!("fixture has no file at {path}"))
    }

    /// The markers cut from the files: `cursor()` for `$0`, `position(name)` for `@name@`, and
    /// `assert_all_used()` once a test has asked about every one it wrote.
    pub fn markers(&self) -> &Markers {
        &self.markers
    }
}

impl FixtureFile {
    fn new(file: &lspkit_fixture::File) -> FixtureFile {
        let path = file.path();
        let mut host = None;
        let mut expect = false;
        for flag in file.flags() {
            if flag == "expect" {
                expect = true;
            } else if let Some(id) = flag.strip_prefix("host=") {
                host = Some(id.to_owned());
            } else {
                panic!("unknown fixture flag {flag:?} on {path}");
            }
        }
        let (text, expected) = if expect {
            let (text, annotations) = Annotation::parse(file.text());
            (text, Some(annotations))
        } else {
            (file.text().to_owned(), None)
        };
        FixtureFile {
            path: path.to_owned(),
            text,
            host,
            raw: file.is_verbatim() && !expect,
            expected,
        }
    }
}
