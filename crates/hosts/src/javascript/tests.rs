use expect_test::expect;

use crate::tests::check;

#[test]
fn javascript_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/greet.js",
        r#"#!/usr/bin/env node
import { readFile } from "node:fs";

/** Says hello to whoever asks. */
export class Greeter {
  greeting = "Hello\nworld";
  #count = 0;
  // Builds the greeting.
  /* A block comment. */
  greet(person, times = 1, ...rest) {
    const message = `Dear ${person}, welcome`;
    return shout(message, times);
  }
}
"#,
        expect![[r#"
            javascript
            > #!/usr/bin/env node
            > import { readFile } from "node:fs";
            >
            > /** Says hello to whoever asks. */
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > export class Greeter {
                           ^^^^^^^ declaration
            >   greeting = "Hello\nworld";
                ^^^^^^^^ declaration
                            ^^^^^ string
                                   ^^^^^ string
            >   #count = 0;
                 ^^^^^ declaration
            >   // Builds the greeting.
                ^^^^^^^^^^^^^^^^^^^^^^^ comment
            >   /* A block comment. */
                ^^^^^^^^^^^^^^^^^^^^^^ comment
            >   greet(person, times = 1, ...rest) {
                ^^^^^ declaration
                      ^^^^^^ declaration
                              ^^^^^ declaration
                                            ^^^^ declaration
            >     const message = `Dear ${person}, welcome`;
                        ^^^^^^^ declaration
                                   ^^^^ string
                                                 ^^^^^^^^^ string
            >     return shout(message, times);
            >   }
            > }
        "#]],
    );
}

#[test]
fn javascript_declares_with_every_declaration_kind() {
    check(
        "/src/paint.mjs",
        r#"export * from "./colours";
const fs = require("node:fs");
const lazy = import("./lazy-brush");
const heading = new RegExp("^#+ title", "gims");
function* strokes() {}
const scale = factor => factor * 2;
const Easel = class Stand {};
let settings = { width: 3, reset() {} };
configure({ height: 4 });
"#,
        expect![[r#"
            javascript
            > export * from "./colours";
            > const fs = require("node:fs");
                    ^^ declaration
            > const lazy = import("./lazy-brush");
                    ^^^^ declaration
            > const heading = new RegExp("^#+ title", "gims");
                    ^^^^^^^ declaration
            > function* strokes() {}
                        ^^^^^^^ declaration
            > const scale = factor => factor * 2;
                    ^^^^^ declaration
                            ^^^^^^ declaration
            > const Easel = class Stand {};
                    ^^^^^ declaration
                                  ^^^^^ declaration
            > let settings = { width: 3, reset() {} };
                  ^^^^^^^^ declaration
                               ^^^^^ declaration
                                         ^^^^^ declaration
            > configure({ height: 4 });
        "#]],
    );
}

#[test]
fn javascript_reads_the_text_between_jsx_tags() {
    check(
        "/src/banner.jsx",
        r#"// Shown at the top of every page.
export function Banner({ title }) {
  return (
    <h1 className="banner">
      Welcome to the {title} page
    </h1>
  );
}
"#,
        expect![[r#"
            javascript
            > // Shown at the top of every page.
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > export function Banner({ title }) {
                              ^^^^^^ declaration
            >   return (
            >     <h1 className="banner">
                                 ^^^^^^ string
            >       Welcome to the {title} page
                    ^^^^^^^^^^^^^^ string
                                           ^^^^ string
            >     </h1>
            >   );
            > }
        "#]],
    );
}
