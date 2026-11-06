//! The names a project's manifests declare: its package, every dependency, and its authors.
//!
//! On the zero corpus a project's own name and its dependencies' were a large share of every
//! checker's noise (`phrost` 980 times, `ripgrep` 789), and its authors' names, in the header of
//! every file, were most of composer's (1 814 of 2 235). They are declared vocabulary: words in
//! this project, whatever a dictionary says.

use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(test)]
mod tests;

/// The manifest names for every path checked: the nearest `Cargo.toml`, `package.json` or
/// `composer.json` at or above each, and every name it gives.
pub(crate) fn names(paths: &[PathBuf]) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = Vec::new();
    for path in paths {
        let Ok(start) = path.canonicalize() else {
            continue;
        };
        let start = if start.is_dir() {
            start
        } else {
            start.parent().map(Path::to_path_buf).unwrap_or(start)
        };
        for dir in start.ancestors() {
            let mut found = false;
            for file in ["Cargo.toml", "package.json", "composer.json"] {
                let manifest = dir.join(file);
                if !manifest.is_file() || seen.contains(&manifest) {
                    continue;
                }
                found = true;
                if let Ok(text) = fs::read_to_string(&manifest) {
                    names.extend(read(file, &text));
                    // A Cargo workspace's members have manifests of their own, and their
                    // dependencies are the project's too: ripgrep's `aho-corasick` is a member's.
                    for member in workspace_members(dir, file, &text) {
                        let member = member.join("Cargo.toml");
                        if let Ok(text) = fs::read_to_string(&member) {
                            names.extend(read(file, &text));
                        }
                    }
                }
                seen.push(manifest);
            }
            if found {
                break;
            }
        }
    }
    names
}

fn read(kind: &str, text: &str) -> Vec<String> {
    let mut names = Vec::new();
    if kind == "Cargo.toml" {
        let Ok(table) = text.parse::<toml::Table>() else {
            return names;
        };
        let package = table.get("package");
        if let Some(name) = package
            .and_then(|p| p.get("name"))
            .and_then(toml::Value::as_str)
        {
            names.push(name.to_owned());
        }
        // `authors = ["Name <email>"]`, in the package or shared by the workspace.
        let shared = table.get("workspace").and_then(|w| w.get("package"));
        for authors in [package, shared]
            .into_iter()
            .flatten()
            .filter_map(|p| p.get("authors")?.as_array())
        {
            names.extend(authors.iter().filter_map(toml::Value::as_str).map(person));
        }
        let workspace = table.get("workspace").and_then(|w| w.get("dependencies"));
        for deps in ["dependencies", "dev-dependencies", "build-dependencies"]
            .iter()
            .filter_map(|section| table.get(*section))
            .chain(workspace)
            .filter_map(toml::Value::as_table)
        {
            names.extend(deps.keys().cloned());
        }
        return names;
    }
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return names;
    };
    if let Some(name) = json.get("name").and_then(serde_json::Value::as_str) {
        names.push(name.to_owned());
    }
    for section in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "require",
        "require-dev",
    ] {
        if let Some(deps) = json.get(section).and_then(serde_json::Value::as_object) {
            names.extend(deps.keys().cloned());
        }
    }
    // npm's `author` and `contributors` are each a string, `Name <email> (url)`, or an object
    // with a `name`; composer's `authors` are objects.
    let people = json.get("author").into_iter().chain(
        ["contributors", "authors"]
            .iter()
            .filter_map(|key| json.get(*key)?.as_array())
            .flatten(),
    );
    for someone in people {
        let name = someone.as_str().or_else(|| someone.get("name")?.as_str());
        names.extend(name.map(person));
    }
    names
}

/// A person's name, without the `<email>` and `(url)` that may follow it.
fn person(written: &str) -> String {
    let end = written.find(['<', '(']).unwrap_or(written.len());
    written[..end].trim().to_owned()
}

/// The member directories a Cargo workspace at `dir` lists; `crates/*` is expanded by listing
/// `dir/crates`.
fn workspace_members(dir: &Path, kind: &str, text: &str) -> Vec<std::path::PathBuf> {
    if kind != "Cargo.toml" {
        return Vec::new();
    }
    let Ok(table) = text.parse::<toml::Table>() else {
        return Vec::new();
    };
    let members = table
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str);
    let mut out = Vec::new();
    for member in members {
        match member.strip_suffix("/*") {
            Some(parent) => {
                if let Ok(entries) = fs::read_dir(dir.join(parent)) {
                    out.extend(entries.flatten().map(|entry| entry.path()));
                }
            }
            None => out.push(dir.join(member)),
        }
    }
    out
}
