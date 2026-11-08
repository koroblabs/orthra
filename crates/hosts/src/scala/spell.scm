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
(class_definition name: (identifier) @declaration)
(object_definition name: (identifier) @declaration)
(trait_definition name: (identifier) @declaration)
(enum_definition name: (identifier) @declaration)
(simple_enum_case name: (identifier) @declaration)
(full_enum_case name: (identifier) @declaration)
(function_definition name: (identifier) @declaration)
(function_declaration name: (identifier) @declaration)
(given_definition name: (identifier) @declaration)
(type_definition name: (type_identifier) @declaration)

; `val` and `var`, and the names a tuple pattern binds: `val (first, second) = pair`.
(val_definition pattern: (identifier) @declaration)
(var_definition pattern: (identifier) @declaration)
(val_declaration name: (identifier) @declaration)
(var_declaration name: (identifier) @declaration)
(val_definition pattern: (tuple_pattern (identifier) @declaration))
(var_definition pattern: (tuple_pattern (identifier) @declaration))

; Parameters, constructor parameters, lambda parameters and `for (x <- xs)`.
(parameter name: (identifier) @declaration)
(class_parameter name: (identifier) @declaration)
(lambda_expression parameters: (identifier) @declaration)
(enumerator . (identifier) @declaration)
