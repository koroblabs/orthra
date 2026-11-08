use expect_test::expect;

use crate::{Hosts, tests::check};

#[test]
fn ini_prose_is_comments_values_and_the_names_a_file_declares() {
    check(
        "/setup.cfg",
        r#"; The packaging metadata.
# Seperates teh sections.
[metadata]
name = greeter
description = A greeter that occured once
license = MIT

[tox:tox]
envlist = py312
commands = pytest --recieve {posargs}
"#,
        expect![[r#"
            ini
            > ; The packaging metadata.
              ^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > # Seperates teh sections.
              ^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > [metadata]
               ^^^^^^^^ declaration
            > name = greeter
              ^^^^ declaration
                     ^^^^^^^ value
            > description = A greeter that occured once
              ^^^^^^^^^^^ declaration
                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^ string
            > license = MIT
              ^^^^^^^ declaration
                        ^^^ value
            >
            > [tox:tox]
               ^^^^^^^ declaration
            > envlist = py312
              ^^^^^^^ declaration
                        ^^^^^ value
            > commands = pytest --recieve {posargs}
              ^^^^^^^^ declaration
                         ^^^^^^^^^^^^^^^^^^^^^^^^^^ value
        "#]],
    );
}

#[test]
fn well_known_file_names_are_ini() {
    let hosts = Hosts::builtin();
    let claimed: String = [
        "/.editorconfig",
        "/.gitconfig",
        "/.gitmodules",
        "/.flake8",
        "/.coveragerc",
        "/.pylintrc",
        "/pylintrc",
        "/tox.ini",
        "/pytest.ini",
        "/app/setup.cfg",
        "/app/settings.cfg",
        "/app/setup.py",
    ]
    .iter()
    .map(|path| {
        format!(
            "{path} {}\n",
            hosts.for_path(path).map_or("-", |host| host.id())
        )
    })
    .collect();
    expect![[r#"
        /.editorconfig ini
        /.gitconfig ini
        /.gitmodules ini
        /.flake8 ini
        /.coveragerc ini
        /.pylintrc ini
        /pylintrc ini
        /tox.ini ini
        /pytest.ini ini
        /app/setup.cfg ini
        /app/settings.cfg ini
        /app/setup.py python
    "#]]
    .assert_eq(&claimed);
}
