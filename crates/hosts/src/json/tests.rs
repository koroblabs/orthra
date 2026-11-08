use expect_test::expect;

use crate::tests::check;

#[test]
fn json_prose_is_strings_keys_and_jsonc_comments() {
    check(
        "/package.json",
        r#"{
  // JSONC allows a comment.
  "name": "greeter",
  "description": "Says hello\nto whoever asks",
  "keywords": ["greeting", "hello world"],
  "private": true,
  "version": "1.0.0",
  "scripts": { "test": "node --test" },
  "motto": "Hello\nworld",
  "path": "C:\\Users\\qzxv"
}
"#,
        expect![[r#"
            json
            > {
            >   // JSONC allows a comment.
                ^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            >   "name": "greeter",
                 ^^^^^ declaration
                        ^^^^^^^^^ value
            >   "description": "Says hello\nto whoever asks",
                 ^^^^^^^^^^^^ declaration
                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ string
            >   "keywords": ["greeting", "hello world"],
                 ^^^^^^^^^ declaration
                             ^^^^^^^^^^ value
                                         ^^^^^^^^^^^^^ string
            >   "private": true,
                 ^^^^^^^^ declaration
            >   "version": "1.0.0",
                 ^^^^^^^^ declaration
                           ^^^^^^^ value
            >   "scripts": { "test": "node --test" },
                 ^^^^^^^^ declaration
                           ^^^^^^^^^^^^^^^^^^^^^^^^^ value
            >   "motto": "Hello\nworld",
                 ^^^^^^ declaration
                         ^^^^^^^^^^^^^^ string
            >   "path": "C:\\Users\\qzxv"
                 ^^^^^ declaration
                        ^^^^^^^^^^^^^^^^^ value
            > }
        "#]],
    );
}
