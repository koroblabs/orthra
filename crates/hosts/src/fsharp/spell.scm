; Where prose lives in F#. Captures: @comment, @doc, @string, @declaration and @skip
; (`tree_sitter_host.rs`).

; Comments. A `///` line is XML doc, and the grammar reads each as a node of its own.
(xml_doc) @doc
(line_comment) @comment
(block_comment) @comment

; Strings have no content nodes, so each is read whole, with its quotes and escapes (the
; convention is to read backslashes as escapes). An interpolated string holds code between its
; braces, which cannot be cut out, so it is left unread.
((string) @string
  (#not-match? @string "^\\$"))
(triple_quoted_string) @string
; A verbatim string is a path or a pattern more often than text, and its backslashes are no
; escapes: `@"C:\Microsoft SDKs"` would lose the `M`.
(verbatim_string) @skip

; An `open` names a namespace, and an attribute a type.
(import_decl) @skip
(attributes) @skip

; Names being declared, never a use. A module or a namespace is a product's name.
(type_name type_name: (identifier) @declaration)
(record_field (identifier) @declaration)
(union_type_case (identifier) @declaration)
(function_declaration_left . (identifier) @declaration)
(value_declaration_left (identifier_pattern (long_identifier_or_op (identifier) @declaration)))
(method_or_prop_defn name: (property_or_ident method: (identifier) @declaration))
(typed_pattern (identifier_pattern (long_identifier_or_op (identifier) @declaration)))
(argument_patterns (long_identifier (identifier) @declaration))
