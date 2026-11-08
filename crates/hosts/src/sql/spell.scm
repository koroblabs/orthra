; Where prose lives in SQL. Captures: @comment, @declaration (`tree_sitter_host.rs`).

(comment) @comment
(marginalia) @comment

; Names being declared, never a use: a table in a `FROM`, a column in a `SELECT` or an `INSERT`, or
; the target of `REFERENCES` is somebody else's to fix.
(create_table (object_reference name: (identifier) @declaration))
(create_view (object_reference name: (identifier) @declaration))
(create_materialized_view (object_reference name: (identifier) @declaration))
(create_function (object_reference name: (identifier) @declaration))
(create_type (object_reference name: (identifier) @declaration))
(create_sequence (object_reference name: (identifier) @declaration))
(create_schema (identifier) @declaration)
(create_index column: (identifier) @declaration)
(column_definition name: (identifier) @declaration)
(function_argument . (identifier) @declaration)
(function_declaration . (identifier) @declaration)

; A string, wherever it stands: a column's comment, seed data, a default. The grammar hides its
; quotes, so the node is a number, a keyword or a string, and a predicate tells which.
((literal) @string
  (#match? @string "^([nNeEuU]&?)?'"))
