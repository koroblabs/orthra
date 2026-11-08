use expect_test::expect;

use crate::tests::check;

#[test]
fn typescript_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/greet.ts",
        r#"import { readFile } from "node:fs";

/** Says hello to whoever asks. */
export class Greeter {
  greeting: string = "Hello\nworld";
  #count = 0;
  // Builds the greeting.
  /* A block comment. */
  greet(person: string, times?: number): string {
    const message = `Dear ${person}, welcome`;
    return shout(message, times);
  }
}
"#,
        expect![[r#"
            typescript
            > import { readFile } from "node:fs";
            >
            > /** Says hello to whoever asks. */
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > export class Greeter {
                           ^^^^^^^ declaration
            >   greeting: string = "Hello\nworld";
                ^^^^^^^^ declaration
                                    ^^^^^ string
                                           ^^^^^ string
            >   #count = 0;
                 ^^^^^ declaration
            >   // Builds the greeting.
                ^^^^^^^^^^^^^^^^^^^^^^^ comment
            >   /* A block comment. */
                ^^^^^^^^^^^^^^^^^^^^^^ comment
            >   greet(person: string, times?: number): string {
                ^^^^^ declaration
                      ^^^^^^ declaration
                                      ^^^^^ declaration
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
fn typescript_declares_with_every_declaration_kind() {
    check(
        "/src/paint.mts",
        r#"export * from "./colours";
import legacy = require("legacy-paint");
const fs = require("node:fs");
const lazy = import("./lazy-brush");
interface Painter { brush: Brush; paint(canvas: Canvas): void }
type Palette = Colour[];
enum Colour { Crimson, Teal = 2 }
namespace Studio {}
abstract class Easel { abstract tilt(angle: number): void }
function* strokes() {}
const scale = factor => factor * 2;
let settings = { width: 3, reset() {} };
configure({ height: 4 });
"#,
        expect![[r#"
            typescript
            > export * from "./colours";
            > import legacy = require("legacy-paint");
            > const fs = require("node:fs");
                    ^^ declaration
            > const lazy = import("./lazy-brush");
                    ^^^^ declaration
            > interface Painter { brush: Brush; paint(canvas: Canvas): void }
                        ^^^^^^^ declaration
                                  ^^^^^ declaration
                                                ^^^^^ declaration
                                                      ^^^^^^ declaration
            > type Palette = Colour[];
                   ^^^^^^^ declaration
            > enum Colour { Crimson, Teal = 2 }
                   ^^^^^^ declaration
                            ^^^^^^^ declaration
                                     ^^^^ declaration
            > namespace Studio {}
                        ^^^^^^ declaration
            > abstract class Easel { abstract tilt(angle: number): void }
                             ^^^^^ declaration
                                              ^^^^ declaration
                                                   ^^^^^ declaration
            > function* strokes() {}
                        ^^^^^^^ declaration
            > const scale = factor => factor * 2;
                    ^^^^^ declaration
                            ^^^^^^ declaration
            > let settings = { width: 3, reset() {} };
                  ^^^^^^^^ declaration
                               ^^^^^ declaration
                                         ^^^^^ declaration
            > configure({ height: 4 });
        "#]],
    );
}
