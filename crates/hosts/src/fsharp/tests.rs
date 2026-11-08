use expect_test::expect;

use crate::tests::check;

#[test]
fn fsharp_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/Greeter.fs",
        r#"module My.Module

open System

/// Greets whoever asks.
type Greeter(name: string) =
    let mutable count = 0
    member this.Greet(person: string, times: int) =
        let message = "Hello\nworld"
        let verbatim = @"verb\atim"
        let tri = """triple"""
        let interp = $"Hi {person}"
        // line comment
        (* block comment *)
        printfn "%s" message
        message

type Color =
    | Red
    | Green of int

type Rec = { Field: int; Other: string }

let rec factorial n = if n = 0 then 1 else n * factorial (n - 1)
let add (a: int) (b: int) = a + b
[<Literal>]
let Thing = 3
"#,
        expect![[r#"
            fsharp
            > module My.Module
            >
            > open System
            >
            > /// Greets whoever asks.
                  ^^^^^^^^^^^^^^^^^^^^ doccomment
            > type Greeter(name: string) =
                   ^^^^^^^ declaration
                           ^^^^ declaration
            >     let mutable count = 0
                              ^^^^^ declaration
            >     member this.Greet(person: string, times: int) =
                              ^^^^^ declaration
                                    ^^^^^^ declaration
                                                    ^^^^^ declaration
            >         let message = "Hello\nworld"
                          ^^^^^^^ declaration
                                    ^^^^^^^^^^^^^^ string
            >         let verbatim = @"verb\atim"
                          ^^^^^^^^ declaration
            >         let tri = """triple"""
                          ^^^ declaration
                                ^^^^^^^^^^^^ string
            >         let interp = $"Hi {person}"
                          ^^^^^^ declaration
            >         // line comment
                      ^^^^^^^^^^^^^^^ comment
            >         (* block comment *)
                      ^^^^^^^^^^^^^^^^^^^ comment
            >         printfn "%s" message
                              ^^^^ string
            >         message
            >
            > type Color =
                   ^^^^^ declaration
            >     | Red
                    ^^^ declaration
            >     | Green of int
                    ^^^^^ declaration
            >
            > type Rec = { Field: int; Other: string }
                   ^^^ declaration
                           ^^^^^ declaration
                                       ^^^^^ declaration
            >
            > let rec factorial n = if n = 0 then 1 else n * factorial (n - 1)
                      ^^^^^^^^^ declaration
                                ^ declaration
            > let add (a: int) (b: int) = a + b
                  ^^^ declaration
                       ^ declaration
                                ^ declaration
            > [<Literal>]
            > let Thing = 3
                  ^^^^^ declaration
        "#]],
    );
}

#[test]
fn fsharp_verbatim_strings_and_interpolated_strings_are_left_unread() {
    check(
        "/src/Paths.fsx",
        r#"let sdk = @"C:\Program Files\Microsoft SDKs"
let hello = $"Hello {name} and {Foo.bar baz}"
let text = "Hello world"
"#,
        expect![[r#"
            fsharp
            > let sdk = @"C:\Program Files\Microsoft SDKs"
                  ^^^ declaration
            > let hello = $"Hello {name} and {Foo.bar baz}"
                  ^^^^^ declaration
            > let text = "Hello world"
                  ^^^^ declaration
                         ^^^^^^^^^^^^^ string
        "#]],
    );
}
