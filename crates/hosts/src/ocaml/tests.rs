use expect_test::expect;

use crate::tests::check;

#[test]
fn ocaml_prose_is_comments_ocamldoc_strings_and_declared_names() {
    check(
        "/lib/greet.ml",
        r#"(* A comment. *)
(** Module doc. *)
open Stdlib

(** Doc for greet. *)
let greet ?(label = "x") person (a, b) ~named =
  let msg = "Hello\nworld {|raw|}" in
  let {field_a; field_b = renamed} = r in
  print_endline msg; Printf.printf "%d items" person

let rec fact n = if n = 0 then 1 else n * fact (n - 1)

type color = Red | Green of int
type 'a tree = Leaf | Node of { left_side : 'a tree; value : 'a }
exception My_error of string
module Foo = struct
  let inner = 1
end
module type S = sig
  val sig_val : int
end
external c_func : int -> int = "caml_c_func"
class point x_init = object
  val mutable x = x_init
  method get_x = x
end
let f = function Some value -> value | None -> 0
let s = {|raw string|}
let c = 'a'
[@@@ocaml.warning "-a"]
let x = 1 [@@deprecated "A deprecation message."]
"#,
        expect![[r#"
            ocaml
            > (* A comment. *)
              ^^^^^^^^^^^^^^^^ comment
            > (** Module doc. *)
              ^^^^^^^^^^^^^^^^^^ doccomment
            > open Stdlib
            >
            > (** Doc for greet. *)
              ^^^^^^^^^^^^^^^^^^^^^ doccomment
            > let greet ?(label = "x") person (a, b) ~named =
                  ^^^^^ declaration
                          ^^^^^ declaration
                                   ^ value
                                       ^^^^^^ declaration
                                               ^ declaration
                                                  ^ declaration
                                                      ^^^^^ declaration
            >   let msg = "Hello\nworld {|raw|}" in
                    ^^^ declaration
                           ^^^^^^^^^^^^^^^^^^^^ string
            >   let {field_a; field_b = renamed} = r in
            >   print_endline msg; Printf.printf "%d items" person
                                                  ^^^^^^^^ string
            >
            > let rec fact n = if n = 0 then 1 else n * fact (n - 1)
                      ^^^^ declaration
                           ^ declaration
            >
            > type color = Red | Green of int
                   ^^^^^ declaration
                           ^^^ declaration
                                 ^^^^^ declaration
            > type 'a tree = Leaf | Node of { left_side : 'a tree; value : 'a }
                      ^^^^ declaration
                             ^^^^ declaration
                                    ^^^^ declaration
                                              ^^^^^^^^^ declaration
                                                                   ^^^^^ declaration
            > exception My_error of string
                        ^^^^^^^^ declaration
            > module Foo = struct
                     ^^^ declaration
            >   let inner = 1
                    ^^^^^ declaration
            > end
            > module type S = sig
                          ^ declaration
            >   val sig_val : int
                    ^^^^^^^ declaration
            > end
            > external c_func : int -> int = "caml_c_func"
                       ^^^^^^ declaration
            > class point x_init = object
                    ^^^^^ declaration
                          ^^^^^^ declaration
            >   val mutable x = x_init
                            ^ declaration
            >   method get_x = x
                       ^^^^^ declaration
            > end
            > let f = function Some value -> value | None -> 0
                  ^ declaration
                                    ^^^^^ declaration
            > let s = {|raw string|}
                  ^ declaration
                        ^^^^^^^^^^ string
            > let c = 'a'
                  ^ declaration
            > [@@@ocaml.warning "-a"]
            > let x = 1 [@@deprecated "A deprecation message."]
                  ^ declaration
                                       ^^^^^^^^^^^^^^^^^^^^^^ string
        "#]],
    );
}

#[test]
fn ocaml_interface_prose_is_comments_ocamldoc_and_declared_names() {
    check(
        "/lib/greet.mli",
        r#"(** Interface doc. *)
(* plain *)
val greet : ?label:string -> string -> unit
(** Doc for greet. *)

type color = Red | Green of int
type t = { mutable count : int; name : string }
module type S = sig
  val inner : int
end
module Foo : S
exception My_error of string
external c_func : int -> int = "caml_c_func"
class point : int -> object method get_x : int end
"#,
        expect![[r#"
            ocaml_interface
            > (** Interface doc. *)
              ^^^^^^^^^^^^^^^^^^^^^ doccomment
            > (* plain *)
              ^^^^^^^^^^^ comment
            > val greet : ?label:string -> string -> unit
                  ^^^^^ declaration
            > (** Doc for greet. *)
              ^^^^^^^^^^^^^^^^^^^^^ doccomment
            >
            > type color = Red | Green of int
                   ^^^^^ declaration
                           ^^^ declaration
                                 ^^^^^ declaration
            > type t = { mutable count : int; name : string }
                   ^ declaration
                                 ^^^^^ declaration
                                              ^^^^ declaration
            > module type S = sig
                          ^ declaration
            >   val inner : int
                    ^^^^^ declaration
            > end
            > module Foo : S
                     ^^^ declaration
            > exception My_error of string
                        ^^^^^^^^ declaration
            > external c_func : int -> int = "caml_c_func"
                       ^^^^^^ declaration
            > class point : int -> object method get_x : int end
                    ^^^^^ declaration
                                                 ^^^^^ declaration
        "#]],
    );
}

/// odoc's references, `{!Arg.argterms}` and `{{!page-cli.envlookup}the lookup}`, are names, but
/// the text of a reference is prose.
#[test]
fn odoc_references_are_not_prose() {
    check(
        "/src/cli.ml",
        r#"(** Parses with {!Arg.argterms}; see {{!page-cli.envlookup}the lookup}. *)
let parse x = x
"#,
        expect![[r#"
            ocaml
            > (** Parses with {!Arg.argterms}; see {{!page-cli.envlookup}the lookup}. *)
              ^^^^^^^^^^^^^^^ doccomment
                                             ^^^^^^^ doccomment
                                                                         ^^^^^^^^^^^^^^^ doccomment
            > let parse x = x
                  ^^^^^ declaration
                        ^ declaration
        "#]],
    );
}

/// odoc aligns a doc's lines under the text after `(** `, four spaces in: that is prose, not an
/// indented block.
#[test]
fn ocamldoc_lines_aligned_under_the_text_are_prose() {
    check(
        "/src/cmd.mli",
        r#"(** Commands.

    Command line syntaxes are defined by terms. *)
val name : string
"#,
        expect![[r#"
            ocaml_interface
            > (** Commands.
              ^^^^^^^^^^^^^ doccomment
            >
            >     Command line syntaxes are defined by terms. *)
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > val name : string
                  ^^^^ declaration
        "#]],
    );
}
