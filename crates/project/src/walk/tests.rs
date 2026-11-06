use std::fs;

use expect_test::expect;

use super::files;

#[test]
fn what_the_project_leaves_out_of_git_is_left_out() {
    let root = std::env::temp_dir().join(format!("orthra-walk-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for (path, text) in [
        (".gitignore", "/vendor/\n*.log\n"),
        ("src/lib.rs", ""),
        ("src/debug.log", ""),
        ("vendor/acme/lib.php", ""),
        ("docs/.ignore", "drafts/\n"),
        ("docs/guide.md", ""),
        ("docs/drafts/idea.md", ""),
        (".hidden/notes.md", ""),
        ("target/debug/out.rs", ""),
        ("web/node_modules/left-pad/index.ts", ""),
    ] {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let mut out = Vec::new();
    files(&root, &lspkit_walk::Override::empty(), &mut out);
    let listed: String = out
        .iter()
        .map(|path| format!("{}\n", path.strip_prefix(&root).unwrap().display()))
        .collect();
    let _ = fs::remove_dir_all(&root);
    expect![[r#"
        docs/guide.md
        src/lib.rs
    "#]]
    .assert_eq(&listed);
}
