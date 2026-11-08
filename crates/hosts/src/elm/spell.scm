; Where prose lives in Elm. Captures: @comment, @doc, @string, @declaration
; (`tree_sitter_host.rs`).

; `{-| … -}` is a doc comment. Every other `--` and `{- -}` is a comment.
((block_comment) @doc (#match? @doc "^\\{-\\|"))
((block_comment) @comment (#not-match? @comment "^\\{-\\|"))
(line_comment) @comment

; The text of a string, between its escapes. A character (`'c'`) has parts too, and is not text.
(string_constant_expr (regular_string_part) @string)

; Names being declared, never a use. An annotation (`f : Int`) repeats the name its definition
; declares, so only the definition is captured, and a port has no definition.
(function_declaration_left . (lower_case_identifier) @declaration)
(port_annotation name: (lower_case_identifier) @declaration)
(lower_pattern (lower_case_identifier) @declaration)

; Types, constructors and record fields. A record built or updated names the fields its type
; declared, and is not captured.
(type_alias_declaration name: (upper_case_identifier) @declaration)
(type_declaration name: (upper_case_identifier) @declaration)
(union_variant name: (upper_case_identifier) @declaration)
(field_type name: (lower_case_identifier) @declaration)
