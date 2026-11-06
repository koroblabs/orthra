use expect_test::expect;

use crate::tests::check;

#[test]
fn yaml_prose_is_comments_strings_and_keys() {
    check(
        "/.github/workflows/ci.yml",
        r#"# Builds and tests on every push.
name: Continuous integration
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - name: "Run the tests"
        run: cargo test --workspace
      - name: Explain
        run: |
          echo "Hello\nworld"
          just test
description: >
  A folded block
  of prose.
script:
  - cargo build --relase
command: ["sh", "-c", "echo qzxv"]
"#,
        expect![[r#"
            yaml
            > # Builds and tests on every push.
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > name: Continuous integration
              ^^^^ declaration
                    ^^^^^^^^^^^^^^^^^^^^^^ string
            > on: [push, pull_request]
              ^^ declaration
                   ^^^^ value
                         ^^^^^^^^^^^^ value
            > jobs:
              ^^^^ declaration
            >   test:
                ^^^^ declaration
            >     runs-on: ubuntu-latest
                  ^^^^^^^ declaration
                           ^^^^^^^^^^^^^ value
            >     timeout-minutes: 30
                  ^^^^^^^^^^^^^^^ declaration
            >     steps:
                  ^^^^^ declaration
            >       - uses: actions/checkout@v4
                      ^^^^ declaration
                            ^^^^^^^^^^^^^^^^^^^ value
            >       - name: "Run the tests"
                      ^^^^ declaration
                            ^^^^^^^^^^^^^^^ string
            >         run: cargo test --workspace
                      ^^^ declaration
                           ^^^^^^^^^^^^^^^^^^^^^^ value
            >       - name: Explain
                      ^^^^ declaration
                            ^^^^^^^ value
            >         run: |
                      ^^^ declaration
                           ^ value (+2 lines)
            >           echo "Hello\nworld"
            >           just test
            > description: >
              ^^^^^^^^^^^ declaration
                           ^ string (+2 lines)
            >   A folded block
            >   of prose.
            > script:
              ^^^^^^ declaration
            >   - cargo build --relase
                ^^^^^^^^^^^^^^^^^^^^^^ value
            > command: ["sh", "-c", "echo qzxv"]
              ^^^^^^^ declaration
                       ^^^^^^^^^^^^^^^^^^^^^^^^^ value
        "#]],
    );
}
