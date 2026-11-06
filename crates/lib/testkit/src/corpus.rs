//! A corpus case: a fixture whose subject is written as the rendering it should produce, under a
//! few lines saying where the case came from.
//!
//! ```text
//! hypothesis: H1
//! source: probe written for the hypotheses in corpora/incumbent/README.md
//! incumbent: not measured yet
//! why: an escape glues letters onto a word
//!
//! //- /src/greet.rs  expect
//! > fn greet() -> String { format!("Hello\nworld") }
//! ```
//!
//! Lines before the first `//- ` header are `key: value` pairs, and `#` starts a comment there.
//! The subject must carry `expect`, so the case reads exactly as a failing test will print it.

use std::{collections::BTreeMap, fs, path::Path};

use crate::{annotate::Annotation, fixture::Fixture};

#[cfg(test)]
mod tests;

/// Every case says what it tests, where it came from, what the incumbent does with it, and why
/// orthra's answer is the right one. A case missing one of these cannot be argued about later.
const REQUIRED: &[&str] = &["hypothesis", "source", "incumbent", "why"];

/// `H1`..`H5` are the hypotheses in `corpora/incumbent/README.md`; `reported` is a case somebody
/// sent.
const HYPOTHESES: &[&str] = &["H1", "H2", "H3", "H4", "H5", "reported"];

#[derive(Debug)]
pub struct Case {
    pub name: String,
    pub meta: BTreeMap<String, String>,
    pub fixture: Fixture,
}

impl Case {
    pub fn parse(name: &str, source: &str) -> Case {
        let header = source
            .lines()
            .position(|line| line.starts_with("//- "))
            .unwrap_or_else(|| panic!("{name}: a case with no fixture"));
        let mut meta = BTreeMap::new();
        for line in source.lines().take(header) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once(": ")
                .unwrap_or_else(|| panic!("{name}: metadata is `key: value`, not {line:?}"));
            assert!(
                REQUIRED.contains(&key),
                "{name}: unknown metadata key {key:?}; the keys are {REQUIRED:?}"
            );
            let previous = meta.insert(key.to_owned(), value.trim().to_owned());
            assert!(previous.is_none(), "{name}: `{key}` given twice");
        }
        for key in REQUIRED {
            assert!(meta.contains_key(*key), "{name}: missing `{key}`");
        }
        let hypothesis = &meta["hypothesis"];
        assert!(
            hypothesis.split(", ").all(|one| HYPOTHESES.contains(&one)),
            "{name}: hypothesis {hypothesis:?} is not among {HYPOTHESES:?}"
        );

        let rest: String = source
            .lines()
            .skip(header)
            .map(|line| format!("{line}\n"))
            .collect();
        let fixture = Fixture::parse(&rest);
        assert!(
            fixture.subject().expected.is_some(),
            "{name}: the subject must be written with `expect`"
        );
        Case {
            name: name.to_owned(),
            meta,
            fixture,
        }
    }

    pub fn expected(&self) -> &[Annotation] {
        self.fixture
            .subject()
            .expected
            .as_deref()
            .expect("parse refuses a case without `expect`")
    }

    /// Every `*.case` under `dir`, by name.
    pub fn load_dir(dir: &Path) -> Vec<Case> {
        let mut paths: Vec<_> = fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("{}: {err}", dir.display()))
            .map(|entry| entry.expect("a readable directory entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "case"))
            .collect();
        paths.sort();
        paths
            .iter()
            .map(|path| {
                let name = path.file_stem().expect("a file name").to_string_lossy();
                let text = fs::read_to_string(path)
                    .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
                Case::parse(&name, &text)
            })
            .collect()
    }
}
