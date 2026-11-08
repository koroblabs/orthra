; Where prose lives in OCaml, for `.ml` and `.mli` alike. Captures: @comment, @doc, @string,
; @skip, @declaration (`tree_sitter_host.rs`).

; `(** … *)` is an ocamldoc comment; `(**)` and a row of stars (`(*****`) are not. Every other
; `(* … *)` is a comment.
((comment) @doc (#match? @doc "^\\(\\*\\*[^*)]"))
((comment) @comment (#not-match? @comment "^\\(\\*\\*[^*)]"))

; The text of a string, in pieces between its escapes and its format conversions (`%d`).
(string (string_content) @string)
(quoted_string (quoted_string_content) @string)

; A C symbol's name is not text, and neither is an attribute's or an extension's payload
; (`[@@deriving show]`, `[%expect {| … |}]`), except the message of a deprecation, which is.
(external (string) @skip)

; A toplevel directive's argument names a file or a package: `#use "topfind"`, `#require "topkg"`.
(toplevel_directive (string) @skip)
(_
  (attribute_id) @_id
  (attribute_payload) @skip
  (#not-match? @_id "deprecated$"))

; Names being declared, never a use: `Printf.printf` and `print_endline` are somebody else's.
(let_binding pattern: (value_name) @declaration)
(value_pattern) @declaration
(value_specification (value_name) @declaration)
(external (value_name) @declaration)
(type_binding name: (type_constructor) @declaration)
(constructor_declaration (constructor_name) @declaration)
(field_declaration (field_name) @declaration)
(module_binding (module_name) @declaration)
(module_type_definition (module_type_name) @declaration)
(class_binding (class_name) @declaration)
(method_definition (method_name) @declaration)
(method_specification (method_name) @declaration)
(instance_variable_definition (instance_variable_name) @declaration)
