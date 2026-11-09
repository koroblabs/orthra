use expect_test::{Expect, expect};

use super::read;

fn check(kind: &str, text: &str, expect: Expect) {
    expect.assert_eq(&read(kind, text).join("\n"));
}

#[test]
fn cargo_declares_its_package_dependencies_and_authors() {
    check(
        "Cargo.toml",
        r#"
[package]
name = "greeter"
authors = ["Ada Lovelace <ada@example.com>"]

[dependencies]
serde = "1"

[workspace.package]
authors = ["Grace Hopper"]
"#,
        expect![[r#"
            greeter
            Ada Lovelace
            Grace Hopper
            serde"#]],
    );
}

/// A target is named apart from its package, and a binary is the name people type.
#[test]
fn cargo_declares_the_names_of_its_targets() {
    check(
        "Cargo.toml",
        r#"
[package]
name = "greeter-daemon"

[lib]
name = "greetlib"

[[bin]]
name = "greetd"
path = "src/main.rs"

[[example]]
name = "greetdemo"
"#,
        expect![[r#"
            greeter-daemon
            greetd
            greetdemo
            greetlib"#]],
    );
}

/// npm's commands are named apart from the package too.
#[test]
fn npm_declares_its_commands() {
    check(
        "package.json",
        r#"{ "name": "greeter", "bin": { "greetd": "cli.js", "greetctl": "ctl.js" } }"#,
        expect![[r#"
            greeter
            greetctl
            greetd"#]],
    );
}

#[test]
fn npm_declares_its_author_as_a_string_or_an_object() {
    check(
        "package.json",
        r#"{
  "name": "greeter",
  "author": "Ada Lovelace <ada@example.com> (https://example.com)",
  "contributors": [{ "name": "Grace Hopper", "email": "grace@example.com" }, "Alan Turing"],
  "dependencies": { "left-pad": "1" }
}"#,
        expect![[r#"
            greeter
            left-pad
            Ada Lovelace
            Grace Hopper
            Alan Turing"#]],
    );
}

#[test]
fn composer_declares_its_authors() {
    check(
        "composer.json",
        r#"{
  "name": "composer/composer",
  "authors": [
    { "name": "Nils Adermann", "email": "naderman@naderman.de" },
    { "name": "Jordi Boggiano", "email": "j.boggiano@seld.be" }
  ],
  "require": { "php": "^7.2.5 || ^8.0", "composer/semver": "^3.3" }
}"#,
        expect![[r#"
            composer/composer
            composer/semver
            php
            Nils Adermann
            Jordi Boggiano"#]],
    );
}
