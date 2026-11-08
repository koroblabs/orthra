use std::{fs, path::PathBuf};

use expect_test::expect;

use crate::{Config, Project};

/// A project whose settings ignore a rooted pattern, walked from a relative path one directory
/// above it: the pattern still ignores, and every file is written as the path was given.
#[test]
fn a_project_walked_from_a_relative_path_keeps_its_rooted_ignores() {
    let base = std::env::temp_dir().join(format!("orthra-relative-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let root = base.join("proj");
    for (path, text) in [
        ("orthra.toml", "ignore = [\"crates/*/words.txt\"]\n"),
        ("crates/hosts/words.txt", "qzxv\n"),
        ("crates/hosts/notes.md", "Notes.\n"),
        (".git/HEAD", "ref: refs/heads/main\n"),
    ] {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    // The same directory, spelled relative to the working directory the test runs in.
    let cwd = std::env::current_dir().unwrap();
    let up: PathBuf = cwd.components().skip(1).map(|_| "..").collect();
    let relative = up.join(root.strip_prefix("/").unwrap());
    let config = Config::load(&Project::root_of(&relative), None);
    let engine = config.engine().unwrap();
    let project = Project::discover(std::slice::from_ref(&relative), &engine, &config);
    let found: Vec<String> = project
        .documents()
        .iter()
        .map(|(path, language)| {
            let shown = path.strip_prefix(&up).unwrap_or(path).to_string_lossy();
            format!(
                "{language:<10}{}",
                shown.replace(base.to_string_lossy().trim_start_matches('/'), "<base>")
            )
        })
        .collect();
    let _ = fs::remove_dir_all(&base);
    expect![[r#"
        markdown  <base>/proj/crates/hosts/notes.md
        toml      <base>/proj/orthra.toml
    "#]]
    .assert_eq(&(found.join("\n") + "\n"));
}
