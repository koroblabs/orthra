; Where prose lives in Scala. Captures: @comment, @doc, @string, @declaration and @skip
; (`tree_sitter_host.rs`).

; Comments. Scaladoc opens with `/**`; `/**/` is an empty comment.
(comment) @comment
((block_comment) @doc
  (#match? @doc "^/\\*\\*[^/]"))
((block_comment) @comment
  (#not-match? @comment "^/\\*\\*[^/]"))

; A package and an import are paths of somebody's modules. So is an annotation:
; `@deprecated("use x", "1.0")` and `@SuppressWarnings("all")` hold a name and a version.
(package_clause) @skip
(import_declaration) @skip
(annotation) @skip

; The text of a string, whole: the grammar gives the pieces between escapes and `$name` no nodes.
(string) @string
(interpolated_string) @string

; Names being declared, never a use: `list.map` and `println(x)` are somebody else's to fix.
; Each names its child `(_)` and keeps it only when it is a plain name: tree-sitter compiles that
; in a fraction of the time it takes for `(identifier)`, which the grammar can reach from many more
; states. The query compiled in 36 ms instead of 60 (2026-11-09); a tuple, a typed pattern or an
; operator is not a plain name, and the patterns below read the ones that hold names.
((class_definition name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((object_definition name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((trait_definition name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((enum_definition name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((simple_enum_case name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((full_enum_case name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((function_definition name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((function_declaration name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((given_definition name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((type_definition name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))

; `val` and `var`, and the names a tuple pattern binds: `val (first, second) = pair`.
((val_definition pattern: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((var_definition pattern: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((val_declaration name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((var_declaration name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
(val_definition pattern: (tuple_pattern (identifier) @declaration))
(var_definition pattern: (tuple_pattern (identifier) @declaration))

; Parameters, constructor parameters, lambda parameters and `for (x <- xs)`.
((parameter name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((class_parameter name: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
((lambda_expression parameters: (_) @declaration)
  (#match? @declaration "^[A-Za-z_$][A-Za-z0-9_$]*$"))
(enumerator . (identifier) @declaration)
