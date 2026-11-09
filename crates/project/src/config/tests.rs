use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

use expect_test::{Expect, expect};

use super::Config;

/// The configuration read from a project and a user directory of `files`, rendered with what it
/// ignores among `paths`.
fn check(files: &[(&str, &str)], paths: &[&str], expect: Expect) {
    // A directory of its own for each call: the tests run on threads of one process.
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "orthra-config-{}-{}",
        std::process::id(),
        CALLS.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    for (path, text) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let project = root.join("project");
    fs::create_dir_all(&project).unwrap();
    let config = Config::load(&project, Some(&root.join("user")));
    let ignored: Vec<String> = paths
        .iter()
        .map(|path| {
            let verdict = if config.is_ignored(&project.join(path)) {
                "ignored"
            } else {
                "read"
            };
            format!("{verdict:<8}{path}")
        })
        .collect();
    let rendered = format!(
        "languages {:?}\nwords {:?}\nproblems {:?}\n{}\n",
        config.languages(),
        config.words(),
        config
            .problems()
            .iter()
            .map(|problem| problem.replace(&*root.to_string_lossy(), "<root>"))
            .collect::<Vec<_>>(),
        ignored.join("\n")
    );
    let _ = fs::remove_dir_all(&root);
    expect.assert_eq(&rendered);
}

#[test]
fn languages_and_words_are_unions_of_every_layer() {
    check(
        &[
            ("user/config.toml", "languages = [\"en-GB\"]\n"),
            ("user/words.txt", "# mine\nqzxvuser\n"),
            (
                "project/orthra.toml",
                "languages = [\"en-GB\", \"uk-UA\"]\nignore = [\"fixtures/\", \"*/vocabulary.txt\"]\n",
            ),
            (
                "project/.orthra/words.txt",
                "orthra\nphrost  # the sibling\n\n",
            ),
        ],
        &["fixtures/a.md", "hosts/vocabulary.txt", "src/lib.rs"],
        expect![[r#"
            languages ["en-GB", "uk-UA"]
            words ["qzxvuser", "orthra", "phrost"]
            problems []
            ignored fixtures/a.md
            ignored hosts/vocabulary.txt
            read    src/lib.rs
        "#]],
    );
}

#[test]
fn english_is_either_spelling_until_a_layer_names_one() {
    check(
        &[],
        &[],
        expect![[r#"
            languages ["en-US", "en-GB"]
            words []
            problems []

        "#]],
    );
    check(
        &[("project/orthra.toml", "languages = [\"uk-UA\"]\n")],
        &[],
        expect![[r#"
            languages ["en-US", "en-GB", "uk-UA"]
            words []
            problems []

        "#]],
    );
    check(
        &[("project/orthra.toml", "languages = [\"en-US\"]\n")],
        &[],
        expect![[r#"
            languages ["en-US"]
            words []
            problems []

        "#]],
    );
    // A later layer still adds: the user's British and the project's American are both asked.
    check(
        &[
            ("user/config.toml", "languages = [\"en-GB\"]\n"),
            ("project/orthra.toml", "languages = [\"en-US\"]\n"),
        ],
        &[],
        expect![[r#"
            languages ["en-GB", "en-US"]
            words []
            problems []

        "#]],
    );
}

#[test]
fn test_data_is_ignored_in_every_language() {
    check(
        &[],
        &[
            "crates/zfs/fixtures/zpool-status.txt",
            "web/api-fixtures/capabilities.json",
            "src/__fixtures__/page.md",
            "cmd/parse/testdata/input.go",
            "src/fixtures.rs",
            "tests/fixture_test.rs",
        ],
        expect![[r#"
            languages ["en-US", "en-GB"]
            words []
            problems []
            ignored crates/zfs/fixtures/zpool-status.txt
            ignored web/api-fixtures/capabilities.json
            ignored src/__fixtures__/page.md
            ignored cmd/parse/testdata/input.go
            read    src/fixtures.rs
            read    tests/fixture_test.rs
        "#]],
    );
}

#[test]
fn a_broken_settings_file_is_a_problem_said_out_loud() {
    check(
        &[(
            "project/orthra.toml",
            "languages = \"en-GB\"\nspelling = [\"strict\"]\nignore = [\n",
        )],
        &[],
        expect![[r#"
            languages ["en-US", "en-GB"]
            words []
            problems ["<root>/project/orthra.toml: TOML parse error at line 3, column 11\n  |\n3 | ignore = [\n  |           ^\nunclosed array, expected `]`\n"]

        "#]],
    );
    check(
        &[(
            "project/orthra.toml",
            "languages = \"en-GB\"\nspelling = [\"strict\"]\n",
        )],
        &["src/lib.rs"],
        expect![[r#"
            languages ["en-US", "en-GB"]
            words []
            problems ["<root>/project/orthra.toml: languages is not a list of strings", "<root>/project/orthra.toml: spelling is not a setting"]
            read    src/lib.rs
        "#]],
    );
}

#[test]
fn a_word_is_added_in_order_and_once() {
    let dir = std::env::temp_dir().join(format!("orthra-words-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let file = dir.join(".orthra/words.txt");
    for word in ["qzxv", "Phrost", "abcq", "QZXV", "orthra"] {
        super::add_word(&file, word).unwrap();
    }
    fs::write(
        &file,
        format!(
            "# the project's words\n{}",
            fs::read_to_string(&file).unwrap()
        ),
    )
    .unwrap();
    super::add_word(&file, "middle").unwrap();
    let written = fs::read_to_string(&file).unwrap();
    let _ = fs::remove_dir_all(&dir);
    expect![[r#"
        # the project's words
        abcq
        middle
        orthra
        Phrost
        qzxv
    "#]]
    .assert_eq(&written);
}

/// A words file an editor saved with a byte-order mark keeps its first word.
#[test]
fn a_byte_order_mark_is_not_part_of_the_first_word() {
    check(
        &[(
            "project/.orthra/words.txt",
            "\u{feff}qzxvfirst\nqzxvsecond\n",
        )],
        &[],
        expect![[r#"
            languages ["en-US", "en-GB"]
            words ["qzxvfirst", "qzxvsecond"]
            problems []

        "#]],
    );
}

/// A word is added as the check reads the file: past a byte-order mark, and once, whatever comment
/// follows a word already there.
#[test]
fn a_word_is_added_as_the_file_is_read() {
    let dir = std::env::temp_dir().join(format!("orthra-words-read-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let file = dir.join(".orthra/words.txt");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        "\u{feff}# the project's words\nphrost  # the sibling\nqzxv\n",
    )
    .unwrap();
    for word in ["phrost", "abcq"] {
        super::add_word(&file, word).unwrap();
    }
    let written = fs::read_to_string(&file).unwrap();
    let _ = fs::remove_dir_all(&dir);
    expect![[r#"
        # the project's words
        abcq
        phrost  # the sibling
        qzxv
    "#]]
    .assert_eq(&written);
}
