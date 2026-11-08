; Where prose lives in Erlang. Captures: @comment, @doc, @string, @skip, @declaration
; (`tree_sitter_host.rs`).

; Every `%` comment is one node. The doc reader trims no `%`, so edoc is read as a comment, and the
; tags whose line is code or a name are left out: `%% @spec greet(string()) -> ok`,
; `%% @type`, `%% @see foo:bar/1`.
((comment) @comment
  (#not-match? @comment "^%+\\s*@(spec|type|see|equiv|end|clear|headerfile|reference|version|since|author|copyright|private|hidden|docfile)"))

; `-doc "..."` and `-moduledoc "..."` are Markdown.
(wild_attribute
  name: (attr_name name: (atom) @_attr)
  value: (string) @doc
  (#any-of? @_attr "doc" "moduledoc"))

; A file a `-include` names is a path, not text.
(pp_include file: (string) @skip)
(pp_include_lib file: (string) @skip)

; The text of a string, with its quotes. One in a binary, `<<"permessage-deflate">>`, is a protocol
; token or a key as often as it is text: known typos only.
(string) @string
(bin_element element: (string) @value)

; Names being declared, never a use. A function once, at its first clause; a type, a callback, a
; record and its fields, and a macro.
(fun_decl . clause: (function_clause name: (atom) @declaration))
(type_alias name: (type_name name: (atom) @declaration))
(opaque name: (type_name name: (atom) @declaration))
(callback fun: (atom) @declaration)
(record_decl name: (atom) @declaration)
(record_decl fields: (record_field name: (atom) @declaration))
(pp_define lhs: (macro_lhs name: (var) @declaration))

; eunit: `foo_test()` and the generator `foo_test_()` are tests, and the strings of their cases are
; labels and data.
((fun_decl clause: (function_clause name: (atom) @_name)) @test
  (#match? @_name "_tests?_?$"))
