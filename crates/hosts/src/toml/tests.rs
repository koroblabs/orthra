use expect_test::expect;

use crate::tests::check;

#[test]
fn toml_prose_is_comments_strings_and_keys() {
    check(
        "/Cargo.toml",
        r#"# The greeter's manifest.
[package]
name = "greeter"
description = "Says hello\nto whoever asks"
license = 'MIT OR Apache-2.0'
keywords = ["greeting", "hello"]

[dependencies]
serde = { version = "1", features = ["derive"] } # serialisation
"tree-sitter" = "=0.27.1"

[profile.release]
notes = """
A multi-line
basic string."""
"#,
        expect![[r#"
            toml
            > # The greeter's manifest.
              ^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > [package]
               ^^^^^^^ declaration
            > name = "greeter"
              ^^^^ declaration
                     ^^^^^^^^^ value
            > description = "Says hello\nto whoever asks"
              ^^^^^^^^^^^ declaration
                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ string
            > license = 'MIT OR Apache-2.0'
              ^^^^^^^ declaration
                        ^^^^^^^^^^^^^^^^^^^ string
            > keywords = ["greeting", "hello"]
              ^^^^^^^^ declaration
                          ^^^^^^^^^^ value
                                      ^^^^^^^ value
            >
            > [dependencies]
               ^^^^^^^^^^^^ declaration
            > serde = { version = "1", features = ["derive"] } # serialisation
              ^^^^^ declaration
                        ^^^^^^^ declaration
                                  ^^^ value
                                       ^^^^^^^^ declaration
                                                   ^^^^^^^^ value
                                                               ^^^^^^^^^^^^^^^ comment
            > "tree-sitter" = "=0.27.1"
               ^^^^^^^^^^^^ declaration
                              ^^^^^^^^^ value
            >
            > [profile.release]
               ^^^^^^^ declaration
                       ^^^^^^^ declaration
            > notes = """
              ^^^^^ declaration
                      ^^^ string (+2 lines)
            > A multi-line
            > basic string."""
        "#]],
    );
}
