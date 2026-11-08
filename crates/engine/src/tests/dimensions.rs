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
fn html_through_every_dimension() {
    check(
        r##"
        //- /src/index.html
        <!-- Seperates teh header. -->
        <h1 id="recieve" title="Hello occured">Welcome&nbsp;Seperates</h1>
        <a href="#recieve">teh link</a>
        <p>{% if recieve %}{{ recieve.occured }} occured{% endif %}</p>
        <script>var recieve = 1;</script>
        "##,
        &[
            ("teh link", "the link"),
            ("{{ recieve.occured }}", "{{ seperates }}"),
        ],
        expect![[r##"
            > <!-- Seperates teh header. -->
                   ^^^^^^^^^ known-typo -> Separates
                             ^^^ known-typo -> the
            > <h1 id="recieve" title="Hello occured">Welcome&nbsp;Seperates</h1>
                                            ^^^^^^^ known-typo -> occurred
                                                                  ^^^^^^^^^ known-typo -> Separates
            > <a href="#recieve">teh link</a>
                                 ^^^ known-typo -> the
            > <p>{% if recieve %}{{ recieve.occured }} occured{% endif %}</p>
                                                       ^^^^^^^ known-typo -> occurred
            > <script>var recieve = 1;</script>
        "##]],
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
fn sql_through_every_dimension() {
    check(
        r#"
        //- /migrations/0001_users.sql
        -- Seperates teh greeting.
        CREATE TABLE recieve (
          person_id uuid,
          note text DEFAULT 'Hello, occured'
        );
        CREATE VIEW people AS SELECT person_id FROM recieve;
        "#,
        &[("teh", "the")],
        expect![[r#"
            > -- Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > CREATE TABLE recieve (
                           ^^^^^^^ known-typo -> receive
            >   person_id uuid,
            >   note text DEFAULT 'Hello, occured'
                                          ^^^^^^^ known-typo -> occurred
            > );
            > CREATE VIEW people AS SELECT person_id FROM recieve;
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

#[test]
fn lua_through_every_dimension() {
    check(
        r#"
        //- /lua/greet.lua
        --- Seperates teh greeting.
        ---@param person string
        local function recieve(person)
          local message = "Hello\nworld, occured"
          return recieve(message)
        end
        "#,
        &[("teh", "the")],
        expect![[r#"
            > --- Seperates teh greeting.
                  ^^^^^^^^^ known-typo -> Separates
                            ^^^ known-typo -> the
            > ---@param person string
            > local function recieve(person)
                             ^^^^^^^ known-typo -> receive
            >   local message = "Hello\nworld, occured"
                                               ^^^^^^^ known-typo -> occurred
            >   return recieve(message)
            > end
        "#]],
    );
}

#[test]
fn scss_through_every_dimension() {
    check(
        r#"
        //- /src/styles.scss
        // Seperates teh header.
        $recieve-color: #336;

        @mixin occured-mixin($recieve-arg) {
          color: $recieve-color;
        }

        .card::before {
          @include occured-mixin($recieve-color);
          widht: 1px;
          content: "Hello\a occured";
          width: calc(100% - #{$recieve-color});
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh header.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > $recieve-color: #336;
               ^^^^^^^ known-typo -> receive
            >
            > @mixin occured-mixin($recieve-arg) {
                     ^^^^^^^ known-typo -> occurred
                                    ^^^^^^^ known-typo -> receive
            >   color: $recieve-color;
            > }
            >
            > .card::before {
            >   @include occured-mixin($recieve-color);
            >   widht: 1px;
                ^^^^^ known-typo -> width
            >   content: "Hello\a occured";
                                  ^^^^^^^ known-typo -> occurred
            >   width: calc(100% - #{$recieve-color});
            > }
        "#]],
    );
}

#[test]
fn less_through_every_dimension() {
    check(
        r#"
        //- /src/styles.less
        // Seperates teh header.
        @recieve-color: #336;

        .occured-mixin(@recieve-arg) {
          color: @recieve-color;
        }

        .card::before {
          .occured-mixin(@recieve-color);
          widht: 1px;
          content: "Hello\a occured";
          width: ~"calc(100% - @{recieve-color})";
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh header.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > @recieve-color: #336;
               ^^^^^^^ known-typo -> receive
            >
            > .occured-mixin(@recieve-arg) {
               ^^^^^^^ known-typo -> occurred
                              ^^^^^^^ known-typo -> receive
            >   color: @recieve-color;
            > }
            >
            > .card::before {
            >   .occured-mixin(@recieve-color);
            >   widht: 1px;
                ^^^^^ known-typo -> width
            >   content: "Hello\a occured";
                                  ^^^^^^^ known-typo -> occurred
            >   width: ~"calc(100% - @{recieve-color})";
            > }
        "#]],
    );
}

#[test]
fn kotlin_through_every_dimension() {
    check(
        r#"
        //- /src/main/kotlin/Greeter.kt
        // Package seperates teh greeting.
        package com.example

        /** Says teh name. */
        fun recieve(person: String): String {
            val message = "Hello\n$person, occured"
            println(message)
            return recieve(person)
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Package seperates teh greeting.
                         ^^^^^^^^^ known-typo -> separates
                                   ^^^ known-typo -> the
            > package com.example
            >
            > /** Says teh name. */
                       ^^^ known-typo -> the
            > fun recieve(person: String): String {
                  ^^^^^^^ known-typo -> receive
            >     val message = "Hello\n$person, occured"
                                                 ^^^^^^^ known-typo -> occurred
            >     println(message)
            >     return recieve(person)
            > }
        "#]],
    );
}

#[test]
fn java_through_every_dimension() {
    check(
        r#"
        //- /src/main/java/org/example/Greeter.java
        // Greeter seperates teh greeting.
        package org.example;

        public class Greeter {
            /** Says it once. */
            String recieve(String person) {
                String message = "Hello\nworld, occured";
                return recieve(message) + person;
            }
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Greeter seperates teh greeting.
                         ^^^^^^^^^ known-typo -> separates
                                   ^^^ known-typo -> the
            > package org.example;
            >
            > public class Greeter {
            >     /** Says it once. */
            >     String recieve(String person) {
                         ^^^^^^^ known-typo -> receive
            >         String message = "Hello\nworld, occured";
                                                      ^^^^^^^ known-typo -> occurred
            >         return recieve(message) + person;
            >     }
            > }
        "#]],
    );
}

#[test]
fn swift_through_every_dimension() {
    check(
        r##"
        //- /Sources/App/Greeter.swift
        // Seperates teh header.
        struct Greeter {
            var recieve: Int = 0

            func occured(person: String) -> String {
                let message = "Hello\n occured, \(person)"
                return message + String(recieve)
            }
        }
        "##,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh header.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > struct Greeter {
            >     var recieve: Int = 0
                      ^^^^^^^ known-typo -> receive
            >
            >     func occured(person: String) -> String {
                       ^^^^^^^ known-typo -> occurred
            >         let message = "Hello\n occured, \(person)"
                                             ^^^^^^^ known-typo -> occurred
            >         return message + String(recieve)
            >     }
            > }
        "#]],
    );
}

#[test]
fn c_through_every_dimension() {
    check(
        r#"
        //- /src/greet.c
        /** Seperates teh greeting. */
        #include <stdio.h>

        int recieve(const char *person) {
        	printf("Hello\n%s, occured", person);
        	return recieve(person);
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > /** Seperates teh greeting. */
                  ^^^^^^^^^ known-typo -> Separates
                            ^^^ known-typo -> the
            > #include <stdio.h>
            >
            > int recieve(const char *person) {
                  ^^^^^^^ known-typo -> receive
            > 	printf("Hello\n%s, occured", person);
              	                   ^^^^^^^ known-typo -> occurred
            > 	return recieve(person);
            > }
        "#]],
    );
}

#[test]
fn cpp_through_every_dimension() {
    check(
        r#"
        //- /src/greet.cpp
        /// Seperates teh greeting.
        #include <string>

        std::string recieve(const std::string &person) {
        	auto note = R"(Hello occured)";
        	return recieve(person) + note;
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > /// Seperates teh greeting.
                  ^^^^^^^^^ known-typo -> Separates
                            ^^^ known-typo -> the
            > #include <string>
            >
            > std::string recieve(const std::string &person) {
                          ^^^^^^^ known-typo -> receive
            > 	auto note = R"(Hello occured)";
              	                     ^^^^^^^ known-typo -> occurred
            > 	return recieve(person) + note;
            > }
        "#]],
    );
}

#[test]
fn ruby_through_every_dimension() {
    check(
        r##"
        //- /lib/greeter.rb
        # Seperates teh greeting.
        # @param person [String] who occured
        def recieve(person)
          message = "Hello\nworld, occured #{person}"
          puts message
          recieve(person)
        end
        "##,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > # @param person [String] who occured
                                           ^^^^^^^ known-typo -> occurred
            > def recieve(person)
                  ^^^^^^^ known-typo -> receive
            >   message = "Hello\nworld, occured #{person}"
                                         ^^^^^^^ known-typo -> occurred
            >   puts message
            >   recieve(person)
            > end
        "#]],
    );
}

#[test]
fn vue_through_every_dimension() {
    check(
        r##"
        //- /src/Card.vue
        <template>
          <!-- Seperates teh header. -->
          <h1 :id="recieve" title="Hello occured">Welcome&nbsp;Seperates</h1>
          <p>{{ recieve.occured }} occured <a :href="recieve">teh link</a></p>
        </template>
        <script setup>
        const recieve = 1;
        </script>
        "##,
        &[
            ("teh link", "the link"),
            ("{{ recieve.occured }}", "{{ seperates }}"),
        ],
        expect![[r#"
            > <template>
            >   <!-- Seperates teh header. -->
                     ^^^^^^^^^ known-typo -> Separates
                               ^^^ known-typo -> the
            >   <h1 :id="recieve" title="Hello occured">Welcome&nbsp;Seperates</h1>
                                               ^^^^^^^ known-typo -> occurred
                                                                     ^^^^^^^^^ known-typo -> Separates
            >   <p>{{ recieve.occured }} occured <a :href="recieve">teh link</a></p>
                                         ^^^^^^^ known-typo -> occurred
                                                                    ^^^ known-typo -> the
            > </template>
            > <script setup>
            > const recieve = 1;
            > </script>
        "#]],
    );
}

#[test]
fn svelte_through_every_dimension() {
    check(
        r##"
        //- /src/Card.svelte
        <script>
          let recieve = 1;
        </script>
        <!-- Seperates teh header. -->
        <h1 id={recieve} title="Hello occured">Welcome&nbsp;Seperates</h1>
        {#if recieve}
          <p>{recieve.occured} occured <a href={recieve}>teh link</a></p>
        {/if}
        "##,
        &[
            ("teh link", "the link"),
            ("{recieve.occured}", "{seperates}"),
        ],
        expect![[r#"
            > <script>
            >   let recieve = 1;
            > </script>
            > <!-- Seperates teh header. -->
                   ^^^^^^^^^ known-typo -> Separates
                             ^^^ known-typo -> the
            > <h1 id={recieve} title="Hello occured">Welcome&nbsp;Seperates</h1>
                                            ^^^^^^^ known-typo -> occurred
                                                                  ^^^^^^^^^ known-typo -> Separates
            > {#if recieve}
            >   <p>{recieve.occured} occured <a href={recieve}>teh link</a></p>
                                     ^^^^^^^ known-typo -> occurred
                                                               ^^^ known-typo -> the
            > {/if}
        "#]],
    );
}

#[test]
fn scala_through_every_dimension() {
    check(
        r#"
        //- /src/main/scala/Greeter.scala
        // Package seperates teh greeting.
        package com.example

        /** Says teh name. */
        object Greeter {
          def recieve(person: String): String = {
            val message = s"Hello\n$person, occured"
            println(message)
            recieve(person)
          }
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Package seperates teh greeting.
                         ^^^^^^^^^ known-typo -> separates
                                   ^^^ known-typo -> the
            > package com.example
            >
            > /** Says teh name. */
                       ^^^ known-typo -> the
            > object Greeter {
            >   def recieve(person: String): String = {
                    ^^^^^^^ known-typo -> receive
            >     val message = s"Hello\n$person, occured"
                                                  ^^^^^^^ known-typo -> occurred
            >     println(message)
            >     recieve(person)
            >   }
            > }
        "#]],
    );
}

#[test]
fn groovy_through_every_dimension() {
    check(
        r#"
        //- /src/main/groovy/Greeter.groovy
        // Package seperates teh greeting.
        package com.example

        /** Says teh name. */
        class Greeter {
            String recieve(String person) {
                def message = "Hello\n${person}, occured"
                println message
                return recieve(person)
            }
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Package seperates teh greeting.
                         ^^^^^^^^^ known-typo -> separates
                                   ^^^ known-typo -> the
            > package com.example
            >
            > /** Says teh name. */
                       ^^^ known-typo -> the
            > class Greeter {
            >     String recieve(String person) {
                         ^^^^^^^ known-typo -> receive
            >         def message = "Hello\n${person}, occured"
                                                       ^^^^^^^ known-typo -> occurred
            >         println message
            >         return recieve(person)
            >     }
            > }
        "#]],
    );
}

#[test]
fn csharp_through_every_dimension() {
    check(
        r#"
        //- /src/Greeter.cs
        // Seperates teh greeting.
        namespace Demo;

        public class Greeter
        {
            public string Recieve(string person)
            {
                var message = "Hello\nworld, occured";
                return Console.WriteLine(message + Recieve(person));
            }
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > namespace Demo;
            >
            > public class Greeter
            > {
            >     public string Recieve(string person)
                                ^^^^^^^ known-typo -> Receive
            >     {
            >         var message = "Hello\nworld, occured";
                                                   ^^^^^^^ known-typo -> occurred
            >         return Console.WriteLine(message + Recieve(person));
            >     }
            > }
        "#]],
    );
}

#[test]
fn fsharp_through_every_dimension() {
    check(
        r#"
        //- /src/Greeter.fs
        // Seperates teh greeting.
        module Demo

        let recieve (person: string) =
            let message = "Hello\nworld, occured"
            printfn "%s" message
            recieve person
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > module Demo
            >
            > let recieve (person: string) =
                  ^^^^^^^ known-typo -> receive
            >     let message = "Hello\nworld, occured"
                                               ^^^^^^^ known-typo -> occurred
            >     printfn "%s" message
            >     recieve person
        "#]],
    );
}

#[test]
fn dart_through_every_dimension() {
    check(
        r#"
        //- /lib/greet.dart
        // Seperates teh greeting.
        class Greeter {
          void recieve(String person) {
            var message = 'Hello\n$person, occured';
            print(message);
            recieve(person);
          }
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > class Greeter {
            >   void recieve(String person) {
                     ^^^^^^^ known-typo -> receive
            >     var message = 'Hello\n$person, occured';
                                                 ^^^^^^^ known-typo -> occurred
            >     print(message);
            >     recieve(person);
            >   }
            > }
        "#]],
    );
}

#[test]
fn zig_through_every_dimension() {
    check(
        r#"
        //- /src/greet.zig
        // Seperates teh greeting.
        const std = @import("std");

        fn recieve(person: []const u8) void {
            const message = "Hello\nworld, occured";
            std.debug.print("{s} {s}", .{ message, person });
            recieve(person);
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > const std = @import("std");
            >
            > fn recieve(person: []const u8) void {
                 ^^^^^^^ known-typo -> receive
            >     const message = "Hello\nworld, occured";
                                                 ^^^^^^^ known-typo -> occurred
            >     std.debug.print("{s} {s}", .{ message, person });
            >     recieve(person);
            > }
        "#]],
    );
}

#[test]
fn objective_c_through_every_dimension() {
    check(
        r#"
        //- /Sources/Greeter.m
        // Seperates teh greeting.
        @interface Greeter : NSObject
        - (void)recieve:(NSString *)person;
        @end

        @implementation Greeter
        - (void)recieve:(NSString *)person {
            NSString *message = @"Hello\nworld, occured";
            [self recieve:message];
        }
        @end
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > @interface Greeter : NSObject
            > - (void)recieve:(NSString *)person;
                      ^^^^^^^ known-typo -> receive
            > @end
            >
            > @implementation Greeter
            > - (void)recieve:(NSString *)person {
            >     NSString *message = @"Hello\nworld, occured";
                                                      ^^^^^^^ known-typo -> occurred
            >     [self recieve:message];
            > }
            > @end
        "#]],
    );
}

#[test]
fn haskell_through_every_dimension() {
    check(
        r#"
        //- /src/Greet.hs
        -- | Seperates teh greeting.
        module Greet where

        recieve :: String -> String
        recieve person = "Hello\nworld, occured"

        main = putStrLn (recieve "x")
        "#,
        &[("teh", "the")],
        expect![[r#"
            > -- | Seperates teh greeting.
                   ^^^^^^^^^ known-typo -> Separates
                             ^^^ known-typo -> the
            > module Greet where
            >
            > recieve :: String -> String
            > recieve person = "Hello\nworld, occured"
              ^^^^^^^ known-typo -> receive
                                              ^^^^^^^ known-typo -> occurred
            >
            > main = putStrLn (recieve "x")
        "#]],
    );
}

#[test]
fn ocaml_through_every_dimension() {
    check(
        r#"
        //- /lib/greet.ml
        (* Seperates teh greeting. *)
        let recieve person = "Hello\nworld, occured"

        let () = print_endline (recieve "x")
        "#,
        &[("teh", "the")],
        expect![[r#"
            > (* Seperates teh greeting. *)
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > let recieve person = "Hello\nworld, occured"
                  ^^^^^^^ known-typo -> receive
                                                  ^^^^^^^ known-typo -> occurred
            >
            > let () = print_endline (recieve "x")
        "#]],
    );
}

#[test]
fn ocaml_interface_through_every_dimension() {
    check(
        r#"
        //- /lib/greet.mli
        (** Seperates teh greeting. *)
        val recieve : string -> string
        "#,
        &[("teh", "the")],
        expect![[r#"
            > (** Seperates teh greeting. *)
                  ^^^^^^^^^ known-typo -> Separates
                            ^^^ known-typo -> the
            > val recieve : string -> string
                  ^^^^^^^ known-typo -> receive
        "#]],
    );
}

#[test]
fn elm_through_every_dimension() {
    check(
        r#"
        //- /src/Main.elm
        module Main exposing (main)

        -- Seperates teh greeting.
        recieve : String -> String
        recieve person =
            "Hello\nworld, occured"

        main = recieve "x"
        "#,
        &[("teh", "the")],
        expect![[r#"
            > module Main exposing (main)
            >
            > -- Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > recieve : String -> String
            > recieve person =
              ^^^^^^^ known-typo -> receive
            >     "Hello\nworld, occured"
                                 ^^^^^^^ known-typo -> occurred
            >
            > main = recieve "x"
        "#]],
    );
}

#[test]
fn elixir_through_every_dimension() {
    check(
        r##"
        //- /lib/greeter.ex
        # Seperates teh greeting.
        defmodule Greeter do
          @doc "Seperates occured."
          def recieve(person) do
            message = "Hello\nworld, occured #{person}"
            IO.puts(message)
            recieve(person)
          end
        end
        "##,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > defmodule Greeter do
            >   @doc "Seperates occured."
                      ^^^^^^^^^ known-typo -> Separates
                                ^^^^^^^ known-typo -> occurred
            >   def recieve(person) do
                    ^^^^^^^ known-typo -> receive
            >     message = "Hello\nworld, occured #{person}"
                                           ^^^^^^^ known-typo -> occurred
            >     IO.puts(message)
            >     recieve(person)
            >   end
            > end
        "#]],
    );
}

#[test]
fn erlang_through_every_dimension() {
    check(
        r##"
        //- /src/greeter.erl
        %% Seperates teh greeting.
        -module(greeter).
        recieve(Person) ->
            Message = "Hello\nworld, occured",
            io:format("~s~n", [Message]),
            recieve(Person).
        "##,
        &[("teh", "the")],
        expect![[r#"
            > %% Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > -module(greeter).
            > recieve(Person) ->
              ^^^^^^^ known-typo -> receive
            >     Message = "Hello\nworld, occured",
                                           ^^^^^^^ known-typo -> occurred
            >     io:format("~s~n", [Message]),
            >     recieve(Person).
        "#]],
    );
}

#[test]
fn r_through_every_dimension() {
    check(
        r#"
        //- /R/greet.R
        #' Seperates teh greeting.
        #' @param person The person.
        recieve <- function(person) {
          message <- "Hello\nworld, occured"
          recieve(message)
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > #' Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > #' @param person The person.
            > recieve <- function(person) {
              ^^^^^^^ known-typo -> receive
            >   message <- "Hello\nworld, occured"
                                          ^^^^^^^ known-typo -> occurred
            >   recieve(message)
            > }
        "#]],
    );
}

#[test]
fn julia_through_every_dimension() {
    check(
        r#"
        //- /src/greet.jl
        # Seperates teh greeting.
        function recieve(person)
            message = "Hello\nworld, occured"
            return recieve(message)
        end
        "#,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > function recieve(person)
                       ^^^^^^^ known-typo -> receive
            >     message = "Hello\nworld, occured"
                                           ^^^^^^^ known-typo -> occurred
            >     return recieve(message)
            > end
        "#]],
    );
}

#[test]
fn powershell_through_every_dimension() {
    check(
        r#"
        //- /scripts/greet.ps1
        # Seperates teh greeting.
        function Recieve-Greeting {
          $message = "Hello world, occured"
          Recieve-Greeting $message
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > function Recieve-Greeting {
                       ^^^^^^^ known-typo -> Receive
            >   $message = "Hello world, occured"
                                         ^^^^^^^ known-typo -> occurred
            >   Recieve-Greeting $message
            > }
        "#]],
    );
}

#[test]
fn makefile_through_every_dimension() {
    check(
        r#"
        //- /Makefile
        # Seperates teh greeting.
        recieve := hello
        $(info Hello world, occured)

        all:
        	@echo $(recieve)
        "#,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > recieve := hello
              ^^^^^^^ known-typo -> receive
            > $(info Hello world, occured)
                                  ^^^^^^^ known-typo -> occurred
            >
            > all:
            > 	@echo $(recieve)
        "#]],
    );
}

#[test]
fn cmake_through_every_dimension() {
    check(
        r#"
        //- /CMakeLists.txt
        # Seperates teh greeting.
        function(recieve person)
          message(STATUS "Hello world, occured")
          recieve(${person})
        endfunction()
        "#,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > function(recieve person)
                       ^^^^^^^ known-typo -> receive
            >   message(STATUS "Hello world, occured")
                                             ^^^^^^^ known-typo -> occurred
            >   recieve(${person})
            > endfunction()
        "#]],
    );
}

#[test]
fn dockerfile_through_every_dimension() {
    check(
        r#"
        //- /Dockerfile
        # Seperates teh greeting.
        ARG recieve=1
        FROM alpine AS builder
        LABEL description="Hello world, occured"
        RUN echo ${recieve}
        "#,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > ARG recieve=1
                  ^^^^^^^ known-typo -> receive
            > FROM alpine AS builder
            > LABEL description="Hello world, occured"
                                              ^^^^^^^ known-typo -> occurred
            > RUN echo ${recieve}
        "#]],
    );
}

#[test]
fn proto_through_every_dimension() {
    check(
        r#"
        //- /api/v1/greet.proto
        // Seperates teh greeting.
        syntax = "proto3";

        import "qzxv/other.proto";

        message Recieve {
          string occured_name = 1;
        }

        service Greeter {
          rpc Say(Recieve) returns (Recieve);
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > // Seperates teh greeting.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            > syntax = "proto3";
            >
            > import "qzxv/other.proto";
            >
            > message Recieve {
                      ^^^^^^^ known-typo -> Receive
            >   string occured_name = 1;
                       ^^^^^^^ known-typo -> occurred
            > }
            >
            > service Greeter {
            >   rpc Say(Recieve) returns (Recieve);
            > }
        "#]],
    );
}

#[test]
fn graphql_through_every_dimension() {
    check(
        r#"
        //- /schema.graphql
        # Seperates teh greeting.
        """
        Seperates the occured greeting.
        """
        type Recieve {
          qzxv(first: Int): Recieve
        }

        query Greet {
          recieve { qzxv }
        }
        "#,
        &[("teh", "the")],
        expect![[r#"
            > # Seperates teh greeting.
                ^^^^^^^^^ known-typo -> Separates
                          ^^^ known-typo -> the
            > """
            > Seperates the occured greeting.
              ^^^^^^^^^ known-typo -> Separates
                            ^^^^^^^ known-typo -> occurred
            > """
            > type Recieve {
                   ^^^^^^^ known-typo -> Receive
            >   qzxv(first: Int): Recieve
            > }
            >
            > query Greet {
            >   recieve { qzxv }
            > }
        "#]],
    );
}

#[test]
fn xml_through_every_dimension() {
    check(
        r#"
        //- /res/layout/main.xml
        <!-- Seperates teh header. -->
        <layout>
          <TextView android:id="@+id/recieve" android:text="Hello occured" />
          <description>Welcome&amp;Seperates teh link</description>
          <version>recieve</version>
          <script><![CDATA[ var recieve = 1; ]]></script>
        </layout>
        "#,
        &[("teh link", "the link")],
        expect![[r#"
            > <!-- Seperates teh header. -->
                   ^^^^^^^^^ known-typo -> Separates
                             ^^^ known-typo -> the
            > <layout>
            >   <TextView android:id="@+id/recieve" android:text="Hello occured" />
                                                                        ^^^^^^^ known-typo -> occurred
            >   <description>Welcome&amp;Seperates teh link</description>
                                         ^^^^^^^^^ known-typo -> Separates
                                                   ^^^ known-typo -> the
            >   <version>recieve</version>
                         ^^^^^^^ known-typo -> receive
            >   <script><![CDATA[ var recieve = 1; ]]></script>
            > </layout>
        "#]],
    );
}

/// tree-sitter-rst parses 9 to 18 % of single edits differently from a fresh parse of the result
/// (2026-11-09, fuzzing a tree and its edit against the same text parsed alone, on this fixture and
/// on Flask's quickstart; tree-sitter-rst-next 0.3.1 as well). The host parses every version
/// whole (`TreeSitterHost::parsed_whole`), and `incremental == from-scratch` holds.
#[test]
fn restructuredtext_through_every_dimension() {
    check(
        r#"
        //- /docs/index.rst
        .. Seperates teh header.

        Welcome to the Seperates
        ========================

        A paragraph with ``recieve()`` and :func:`os.occured` and teh link.

        Run it::

            recieve --occured

        .. note:: Seperates this teh note
        "#,
        &[("teh link", "the link")],
        expect![[r#"
            > .. Seperates teh header.
                 ^^^^^^^^^ known-typo -> Separates
                           ^^^ known-typo -> the
            >
            > Welcome to the Seperates
                             ^^^^^^^^^ known-typo -> Separates
            > ========================
            >
            > A paragraph with ``recieve()`` and :func:`os.occured` and teh link.
                                                           ^^^^^^^ known-typo -> occurred
                                                                        ^^^ known-typo -> the
            >
            > Run it::
            >
            >     recieve --occured
            >
            > .. note:: Seperates this teh note
                        ^^^^^^^^^ known-typo -> Separates
                                       ^^^ known-typo -> the
        "#]],
    );
}
