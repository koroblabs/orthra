; Where prose lives in Zig. Captures: @comment, @doc, @string, @declaration, @skip
; (`tree_sitter_host.rs`). What is not captured is not checked: a call, a member access and a
; type are somebody else's names.

; Comments. `///` documents the declaration after it and `//!` its container; `////` is not.
((comment) @doc
  (#match? @doc "^//(/[^/]|/$|!)"))
((comment) @comment
  (#not-match? @comment "^//(/[^/]|/$|!)"))

; The text of a string, in pieces between its escapes.
(string (string_content) @string)

; A multiline string (`\\` at the start of each line) holds source code, a format, or data as
; often as it holds text: zls's tests keep Zig, with `lself` and `rself` in it, in them. It is
; held to known typos only.
(multiline_string) @value

; A string that is an argument of a builtin is a path or a name: `@import("std")`,
; `@embedFile("data.txt")`, `@hasDecl(T, "init")`, `@field(value, "name")`. A linkage is not
; prose either: `extern "c" fn puts(...)`.
(builtin_function (arguments (string) @skip))
(function_declaration (string) @skip)

; Names being declared, never a use: a constant, a variable or a function, a field of a struct,
; an enum or a union, an error of an error set, and a parameter.
(variable_declaration . (identifier) @declaration)
(function_declaration name: (identifier) @declaration)
(container_field name: (identifier) @declaration)
(error_set_declaration (identifier) @declaration)
(parameter name: (identifier) @declaration)
