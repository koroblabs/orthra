; Where prose lives in a Nix file.

; RFC 145 doc comments are `/** ... */`. Listed first: of two patterns that capture one range, the
; first wins.
((comment) @doc
  (#match? @doc "^/[*][*]"))
(comment) @comment

; The text of a string, between its escapes and interpolations.
(string_fragment) @string

; An attribute's name is a declaration where it is bound: `let` bindings and the keys of a set.
; A quoted key is the same name, written with quotes.
(binding
  attrpath: (attrpath attr: (identifier) @declaration))
(binding
  attrpath: (attrpath attr: (string_expression (string_fragment) @declaration)))

; An indented string that a build runs is shell code, held to known typos only: a derivation's
; phases and hooks, a shell hook, an activation script. Its words are commands and variables.
(binding
  attrpath: (attrpath attr: (identifier) @_key .)
  expression: (indented_string_expression) @value
  (#match? @_key "Phase$|^(pre|post)[A-Z]|^(shellHook|script|text|run|init|extraInit|initExtra|profileExtra|interactiveShellInit|shellInit|loginShellInit)$"))

; A function's parameters: `{ pkgs, lib, ... }:` and `name: body`.
(formal name: (identifier) @declaration)
(function_expression universal: (identifier) @declaration)
