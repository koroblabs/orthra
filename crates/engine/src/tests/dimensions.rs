use expect_test::{Expect, expect};
use orthra_harness::{
    dimensions::check_all,
    subject::{Edit, Observed},
};
use orthra_testkit::fixture::Fixture;

use super::subject::Orthra;

fn check(source: &str, edits: &[(&str, &str)], expect: Expect) {
    let fixture = Fixture::parse(source);
    let mut text = fixture.subject().text.clone();
    let edits: Vec<Edit> = edits
        .iter()
        .map(|(old, new)| {
            let edit = Edit::replace(&text, old, new);
            edit.apply(&mut text);
            edit
        })
        .collect();
    let findings = check_all(&Orthra, &fixture, &edits);
    expect.assert_eq(&Observed::render(&fixture.subject().text, &findings));
}

#[test]
fn plain_text_typos_known_and_unknown() {
    check(
        r#"
        //- /notes.txt
        Seperate the recieved values; then count what occured.
        A qwzxv is not a word, and neither is "frobnicatz".
        "#,
        &[("recieved", "received"), ("qwzxv", "word")],
        expect![[r#"
            > Seperate the recieved values; then count what occured.
              ^^^^^^^^ known-typo -> Separate
                           ^^^^^^^^ known-typo -> received
                                                            ^^^^^^^ known-typo -> occurred
            > A qwzxv is not a word, and neither is "frobnicatz".
                ^^^^^ unknown-word
                                                     ^^^^^^^^^^ unknown-word
        "#]],
    );
}

#[test]
fn markdown_checks_prose_and_not_code() {
    check(
        r#"
        //- /README.md
        # Using the `fooBarBaz` helper

        Call `qzxRender()` before anything else; it is definately required.

        ```rust
        let xyzzy = qzx_render(plugh);
        ```

        See [the docs](https://example.com/qzx-render) for more.
        "#,
        &[("definately", "definitely")],
        expect![[r#"
            > # Using the `fooBarBaz` helper
            >
            > Call `qzxRender()` before anything else; it is definately required.
                                                             ^^^^^^^^^^ known-typo -> definitely
            >
            > ```rust
            > let xyzzy = qzx_render(plugh);
            > ```
            >
            > See [the docs](https://example.com/qzx-render) for more.
        "#]],
    );
}

#[test]
fn a_mixed_script_word_and_cyrillic_without_its_pack() {
    check(
        "//- /notes.txt\nThе total, in bytes. Перевірити кеш.\n",
        &[],
        expect![[r#"
            > Thе total, in bytes. Перевірити кеш.
              ^^^ mixed-script -> The
        "#]],
    );
}

#[test]
fn words_typed_in_the_wrong_layout_are_the_words_meant() {
    // An English writer with the Ukrainian layout still on: no Cyrillic pack, and still a slip.
    // `вщтєе` is `don't`, the apostrophe on `є`'s key. A Ukrainian word typed so stays unchecked.
    check(
        "//- /notes.txt\nРуддщ цщкдв, I вщтєе know. Перевірити кеш.\n",
        &[],
        expect![[r#"
            > Руддщ цщкдв, I вщтєе know. Перевірити кеш.
              ^^^^^ wrong-layout -> Hello
                    ^^^^^ wrong-layout -> world
                             ^^^^^ wrong-layout -> don't
        "#]],
    );
}

#[test]
fn rust_through_every_dimension() {
    check(
        r#"
        //- /src/lib.rs
        /// Seperate the recieved values.
        pub fn recieve_message(count: usize) -> String {
            // A qwzxv is not a word; neither is frobnicatz in prose.
            let message = format!("Hello\nworld {count}");
            message
        }
        "#,
        &[("Seperate", "Separate"), ("qwzxv", "word")],
        expect![[r#"
            > /// Seperate the recieved values.
                  ^^^^^^^^ known-typo -> Separate
                               ^^^^^^^^ known-typo -> received
            > pub fn recieve_message(count: usize) -> String {
                     ^^^^^^^ known-typo -> receive
            >     // A qwzxv is not a word; neither is frobnicatz in prose.
                       ^^^^^ unknown-word
                                                       ^^^^^^^^^^ unknown-word
            >     let message = format!("Hello\nworld {count}");
            >     message
            > }
        "#]],
    );
}

#[test]
fn typescript_through_every_dimension() {
    check(
        r#"
        //- /src/api.ts
        /** Fetches the definately current user. */
        export function getURLsForIDs(ids: string[]): string[] {
          const recieveTimeoutMs = 3000;
          return ids.map((id) => `user ${id} occured`);
        }
        "#,
        &[("definately", "definitely")],
        expect![[r#"
            > /** Fetches the definately current user. */
                              ^^^^^^^^^^ known-typo -> definitely
            > export function getURLsForIDs(ids: string[]): string[] {
            >   const recieveTimeoutMs = 3000;
                      ^^^^^^^ known-typo -> receive
            >   return ids.map((id) => `user ${id} occured`);
                                                   ^^^^^^^ known-typo -> occurred
            > }
        "#]],
    );
}

#[test]
fn javascript_through_every_dimension() {
    check(
        r#"
        //- /src/api.js
        /** Fetches the definately current user. */
        export function getURLsForIDs(ids) {
          const recieveTimeoutMs = 3000;
          return ids.map((id) => `user ${id} occured`);
        }
        "#,
        &[("definately", "definitely")],
        expect![[r#"
            > /** Fetches the definately current user. */
                              ^^^^^^^^^^ known-typo -> definitely
            > export function getURLsForIDs(ids) {
            >   const recieveTimeoutMs = 3000;
                      ^^^^^^^ known-typo -> receive
            >   return ids.map((id) => `user ${id} occured`);
                                                   ^^^^^^^ known-typo -> occurred
            > }
        "#]],
    );
}

#[test]
fn python_through_every_dimension() {
    check(
        r#"
        //- /src/greet.py
        """Seperate the greeting helpers."""


        def recieve(person, *args):
            # Build teh greeting.
            message = f"Hello\nworld, {person}!"
            return message
        "#,
        &[("teh", "the")],
        expect![[r#"
            > """Seperate the greeting helpers."""
                 ^^^^^^^^ known-typo -> Separate
            >
            >
            > def recieve(person, *args):
                  ^^^^^^^ known-typo -> receive
            >     # Build teh greeting.
                          ^^^ known-typo -> the
            >     message = f"Hello\nworld, {person}!"
            >     return message
        "#]],
    );
}

#[test]
fn go_through_every_dimension() {
    check(
        r#"
        //- /cmd/greet/main.go
        // Package main seperates teh greeting.
        package main

        import "fmt"

        func recieve(person string) string {
        	message := "Hello\nworld, occured"
        	fmt.Println(message)
        	return person
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Package main seperates teh greeting.
                              ^^^^^^^^^ known-typo -> separates
                                        ^^^ known-typo -> the
            > package main
            >
            > import "fmt"
            >
            > func recieve(person string) string {
                   ^^^^^^^ known-typo -> receive
            > 	message := "Hello\nworld, occured"
              	                          ^^^^^^^ known-typo -> occurred
            > 	fmt.Println(message)
            > 	return person
            > }
        "#]],
    );
}

#[test]
fn css_through_every_dimension() {
    check(
        r#"
        //- /src/styles.css
        /* Seperates teh header. */
        :root {
          --recieve-color: #336;
        }

        .card::before {
          color: var(--recieve-color);
          widht: 1px;
          content: "Hello\a occured";
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > /* Seperates teh header. */
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > :root {
            >   --recieve-color: #336;
                  ^^^^^^^ known-typo -> receive
            > }
            >
            > .card::before {
            >   color: var(--recieve-color);
            >   widht: 1px;
                ^^^^^ known-typo -> width
            >   content: "Hello\a occured";
                                  ^^^^^^^ known-typo -> occurred
            > }
        "#]],
    );
}

#[test]
fn php_through_every_dimension() {
    check(
        r#"
        //- /src/Greeter.php
        <?php
        /**
         * Seperates teh greeting.
         */
        function recieve(string $person): string
        {
            $message = "Hello\nworld, occured";
            return $person . $message;
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > <?php
            > /**
            >  * Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            >  */
            > function recieve(string $person): string
                       ^^^^^^^ known-typo -> receive
            > {
            >     $message = "Hello\nworld, occured";
                                            ^^^^^^^ known-typo -> occurred
            >     return $person . $message;
            > }
        "#]],
    );
}

#[test]
fn toml_through_every_dimension() {
    check(
        r#"
        //- /Cargo.toml
        # Seperates teh greeting.
        [package]
        name = "greeter"
        description = "Says hello, occured"
        keywords = ["recieve", "qzxv"]

        [dependancies]
        "#,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > [package]
            > name = "greeter"
            > description = "Says hello, occured"
                                         ^^^^^^^ known-typo -> occurred
            > keywords = ["recieve", "qzxv"]
                           ^^^^^^^ known-typo -> receive
            >
            > [dependancies]
               ^^^^^^^^^^^^ known-typo -> dependencies
        "#]],
    );
}

#[test]
fn yaml_through_every_dimension() {
    check(
        r#"
        //- /.github/workflows/ci.yml
        # Seperates teh greeting.
        name: Says hello, occured
        on: [push, recieve]
        jobs:
          qzxv:
            runs-on: ubuntu-latest
            steps:
              - run: cargo biuld --qzxv
        "#,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > name: Says hello, occured
                                ^^^^^^^ known-typo -> occurred
            > on: [push, recieve]
                         ^^^^^^^ known-typo -> receive
            > jobs:
            >   qzxv:
            >     runs-on: ubuntu-latest
            >     steps:
            >       - run: cargo biuld --qzxv
                                 ^^^^^ known-typo -> build
        "#]],
    );
}

#[test]
fn json_through_every_dimension() {
    check(
        r#"
        //- /package.json
        {
          "description": "Seperates teh greeting, occured",
          "keywords": ["recieve", "qzxv"],
          "dependancies": { "qzxv": "^1.0" }
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > {
            >   "description": "Seperates teh greeting, occured",
                                ^^^^^^^^^ known-typo -> Separates
                                          ^^^ known-typo -> the
                                                        ^^^^^^^ known-typo -> occurred
            >   "keywords": ["recieve", "qzxv"],
                              ^^^^^^^ known-typo -> receive
            >   "dependancies": { "qzxv": "^1.0" }
                 ^^^^^^^^^^^^ known-typo -> dependencies
            > }
        "#]],
    );
}

/// An edit that keeps a span's range and kind but changes where its words are: the span's kept
/// words are not reused, because the edit touched it. Live, `quick brown` became one word.
#[test]
fn an_edit_inside_a_span_that_moves_its_word_boundaries() {
    check(
        r#"
        //- /src/lib.rs
        // The quick brown fox.
        fn quick() {}
        "#,
        &[(" brown", "xbrown")],
        expect![[r#"
            > // The quick brown fox.
            > fn quick() {}
        "#]],
    );
}

/// Test code, found by patterns over two siblings, an attribute and the item after it: its strings
/// are test data, and not reported, however the tree is cut for a keystroke.
#[test]
fn rust_test_code_through_every_dimension() {
    check(
        r#"
        //- /src/lib.rs
        /// Sends the mesage.
        pub fn send() -> &'static str {
            "the qzxv is shown"
        }

        #[cfg(test)]
        mod tests {
            #[test]
            fn sends() {
                assert_eq!(super::send(), "the wrld is fixture data");
            }
        }

        #[test]
        fn alone() {
            let _ = "frobnicatz is test data too";
        }
        "#,
        &[("mesage", "message"), ("qzxv", "word")],
        expect![[r#"
            > /// Sends the mesage.
                            ^^^^^^ known-typo -> message
            > pub fn send() -> &'static str {
            >     "the qzxv is shown"
                       ^^^^ unknown-word
            > }
            >
            > #[cfg(test)]
            > mod tests {
            >     #[test]
            >     fn sends() {
            >         assert_eq!(super::send(), "the wrld is fixture data");
            >     }
            > }
            >
            > #[test]
            > fn alone() {
            >     let _ = "frobnicatz is test data too";
            > }
        "#]],
    );
}

#[test]
fn shell_through_every_dimension() {
    check(
        r#"
        //- /bin/greet.sh
        #!/usr/bin/env bash
        # Seperates teh greeting.
        recieve() {
          local message="Hello\nworld, occured"
          echo "$message"
        }

        recieve
        "#,
        &[("teh", "the")],
        expect![[r#"
            > #!/usr/bin/env bash
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > recieve() {
              ^^^^^^^ known-typo -> receive
            >   local message="Hello\nworld, occured"
                                             ^^^^^^^ known-typo -> occurred
            >   echo "$message"
            > }
            >
            > recieve
        "#]],
    );
}
