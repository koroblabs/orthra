; Where prose lives in Kotlin. Captures: @comment, @doc, @string, @declaration and @skip
; (`tree_sitter_host.rs`).

; Comments. A KDoc block opens with `/**`; `/**/` is an empty comment.
(line_comment) @comment
((block_comment) @doc
  (#match? @doc "^/\\*\\*[^/]"))
((block_comment) @comment
  (#not-match? @comment "^/\\*\\*[^/]"))

; A package and an import are paths of somebody's modules. So is an annotation: `@Suppress("unused")`
; and `@JvmName("shout")` hold a diagnostic's name and a symbol's, not text.
(package_header) @skip
(import) @skip
(annotation) @skip
(file_annotation) @skip

; The text of a string, raw or not, in pieces between its escapes and `$name` or `${...}`.
(string_literal (string_content) @string)
(multiline_string_literal (string_content) @string)

; Names being declared, never a use: `println(x)` and `list.map { }` are somebody else's to fix.
(class_declaration name: (identifier) @declaration)
(object_declaration name: (identifier) @declaration)
(companion_object name: (identifier) @declaration)
(type_alias type: (identifier) @declaration)
(enum_entry (identifier) @declaration)

; Only the function's own name: an extension function's receiver is a type that somebody else
; declared.
(function_declaration name: (identifier) @declaration)

; `val` and `var` properties, locals, destructuring names, loop variables and lambda parameters:
; every one is a variable_declaration.
(variable_declaration (identifier) @declaration)
(catch_block (identifier) @declaration)

; A parameter's name is the identifier before its `:`; a default value after it can be an
; identifier too, and that is a use.
(parameter (identifier) @declaration . ":")
(class_parameter (identifier) @declaration . ":")
