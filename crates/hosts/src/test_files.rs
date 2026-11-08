//! Which files are tests, and which a tool writes. A test's strings are test data, not text, and
//! are left unchecked; a generated file is not checked at all.
//!
//! Every language keeps tests in a `tests` or `test` directory. Beyond that each has its own
//! conventions (`*_test.go`, `test_*.py`, `*Test.php`), and those are its host's data
//! ([`Host::test_files`]), never a branch here.

use crate::Host;

/// Directories that hold tests whatever the language.
const EVERY_LANGUAGE: &[&str] = &["test/", "tests/"];

/// Whether `path` is one of `host`'s test files.
pub fn is_test_path(host: &dyn Host, path: &str) -> bool {
    let path = path.replace('\\', "/");
    let (directories, name) = path.rsplit_once('/').unwrap_or(("", &path));
    EVERY_LANGUAGE
        .iter()
        .chain(host.test_files())
        .any(|pattern| match pattern.strip_suffix('/') {
            Some(directory) => directories.split('/').any(|part| part == directory),
            None => matches(pattern, name),
        })
}

/// Whether `path` is a file `host`'s language has a tool write: its name matches one of
/// [`Host::generated_files`].
pub fn is_generated(host: &dyn Host, path: &str) -> bool {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    names_match(host.generated_files(), name)
}

/// Whether the file name `name` matches one of `patterns`, where each `*` stands for any run of
/// characters: a host's generated files, and the names it claims.
pub(crate) fn names_match(patterns: &[&str], name: &str) -> bool {
    patterns.iter().any(|pattern| matches(pattern, name))
}

/// Whether `name` matches `pattern`, where each `*` stands for any run of characters, empty
/// included.
fn matches(pattern: &str, name: &str) -> bool {
    let mut pieces = pattern.split('*');
    let Some(mut rest) = name.strip_prefix(pieces.next().unwrap_or_default()) else {
        return false;
    };
    let Some(mut last) = pieces.next() else {
        return rest.is_empty();
    };
    for piece in pieces {
        let Some(at) = rest.find(last) else {
            return false;
        };
        rest = &rest[at + last.len()..];
        last = piece;
    }
    rest.ends_with(last)
}

#[cfg(test)]
mod tests;
