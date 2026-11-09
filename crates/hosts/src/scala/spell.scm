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
; Each names its child `(_)` and keeps it only when it is a plain name, one with no space or ASCII
; punctuation but `_` and `$` in it, in any script: tree-sitter compiles that in a fraction of the
; time it takes for `(identifier)`, which the grammar can reach from many more states. The query
; compiled in 33 ms instead of 60 (2026-11-09); a tuple, a typed pattern or an operator is not a
; plain name, and the patterns below read the ones that hold names. A regex of Unicode classes per
; pattern cost 10 ms more to compile.
((class_definition name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((object_definition name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((trait_definition name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((enum_definition name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((simple_enum_case name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((full_enum_case name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((function_definition name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((function_declaration name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((given_definition name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((type_definition name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))

; `val` and `var`, and the names a tuple pattern binds: `val (first, second) = pair`.
((val_definition pattern: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((var_definition pattern: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((val_declaration name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((var_declaration name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
(val_definition pattern: (tuple_pattern (identifier) @declaration))
(var_definition pattern: (tuple_pattern (identifier) @declaration))

; Parameters, constructor parameters, lambda parameters and `for (x <- xs)`.
((parameter name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((class_parameter name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((lambda_expression parameters: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
(enumerator . (identifier) @declaration)
