; Where prose lives in Julia. Captures: @comment, @doc, @string, @declaration
; (`tree_sitter_host.rs`). Strings with a prefix (`r"..."`, `raw"..."`, `md"..."`) and commands in
; backticks are patterns, paths and programs, and are not captured.

(line_comment) @comment
(block_comment) @comment

; A docstring is a string on the line before a definition. Its pattern comes before the plain
; string's, so that when both capture the same text, @doc is found first and kept.
((string_literal (content) @doc)
  .
  [(line_comment) (block_comment)]*
  .
  [(function_definition) (macro_definition) (struct_definition) (abstract_definition)
   (primitive_definition) (module_definition) (const_statement) (assignment) (identifier)
   (macrocall_expression) (call_expression)])

; What follows a doubled backslash is LaTeX, a path or a regex, not a word: `"\\mathbf{x}"`, which
; a docstring writes for math. Its pattern comes before the plain string's, and a skip wins at the
; same range.
((string_literal (escape_sequence) @_escape . (content) @skip)
  (#eq? @_escape "\\\\"))

; The text of a string, without its quotes, in the pieces around its escapes and `$interpolations`.
(string_literal (content) @string)

; A name that mixes Greek and Latin letters, `αsum`, is how a Julia program writes its math, and
; no homoglyph. Not read.
((identifier) @skip (#match? @skip "\\p{Greek}"))

; Names being declared, never a use: `Base.show(io, x) = ...` extends somebody else's function
; and `x.field = 1` sets somebody else's field.

; A function or a macro, written out: `function f(x)`, `f(x) where T`. Every pattern here is paid
; for when the grammar loads: `Query::new` takes about 10 ms for each one that reaches into an
; argument list or a type head, in a grammar this size (measured 2026-11-08, on a slow VM). So the
; rarer forms are left out, and their names go unread: `f(x)::T`, `xs...`, `S <: T`, and the
; parameters of a `where` function or a short one.
; Each declared child is `(_)`, kept when its text has no space, dot, bracket or operator in it: a
; plain name in any script, `résumé!` and `θ` too. tree-sitter compiles that in less time than
; `(identifier)`: the query in 70 ms instead of 86 (2026-11-09).
((signature (call_expression . (_) @declaration))
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))
((signature (where_expression (call_expression . (_) @declaration)))
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))

; Its parameters: `x`, `x::T` and the keyword `k = 1`.
((signature (call_expression (argument_list (_) @declaration)))
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))
((signature (call_expression (argument_list (typed_expression . (_) @declaration))))
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))
((signature (call_expression (argument_list (named_argument . (_) @declaration))))
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))

; A function written as an assignment: `f(x) = x + 1`.
((assignment . (call_expression . (_) @declaration))
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))

; A type: `struct P`, `struct P{T}`, `abstract type A end`, and a struct's fields.
((type_head (_) @declaration)
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))
((type_head (parametrized_type_expression . (_) @declaration))
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))
((struct_definition (_) @declaration)
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))
((struct_definition (typed_expression . (_) @declaration))
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))

((module_definition name: (_) @declaration)
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))

; A variable: `x = 1`, `const LIMIT = 5`, `for i in 1:3`.
((assignment . (_) @declaration)
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))
((for_binding . (_) @declaration)
  (#not-match? @declaration "[\\s.,;:(){}\\[\\]=<>\"'+*/^&|\\\\@$%~?-]"))
