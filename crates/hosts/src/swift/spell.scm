; Where prose lives in Swift. Captures: @comment, @doc, @string, @declaration, @skip
; (`tree_sitter_host.rs`).

; Comments. `///` and `/** */` are doc comments; `////` and `/***` are plain ones, as in Swift.
((comment) @doc
  (#match? @doc "^///([^/]|$)"))
((comment) @comment
  (#not-match? @comment "^///([^/]|$)"))
((multiline_comment) @doc
  (#match? @doc "^/\\*\\*[^*/]"))
((multiline_comment) @comment
  (#not-match? @comment "^/\\*\\*[^*/]"))

; The text of a string, in pieces between its escapes and its `\(...)` interpolations.
(line_string_literal (line_str_text) @string)
(multi_line_string_literal (multi_line_str_text) @string)
(raw_string_literal (raw_str_part) @string)
(raw_string_literal (raw_str_end_part) @string)

; An attribute names something, `@available(*, deprecated, renamed: "foo")`: not text.
(attribute) @skip

; Names being declared, never a use. The name of an extension is the extended type's: a use, and
; a `user_type`, so `type_identifier` leaves it out.
(class_declaration name: (type_identifier) @declaration)
(protocol_declaration name: (type_identifier) @declaration)
(typealias_declaration name: (type_identifier) @declaration)
(associatedtype_declaration name: (type_identifier) @declaration)
(type_parameter (type_identifier) @declaration)
(enum_entry name: (simple_identifier) @declaration)
(_ "func" . name: (simple_identifier) @declaration)
(parameter external_name: (simple_identifier) @declaration)
(parameter name: (simple_identifier) @declaration)
(pattern bound_identifier: (simple_identifier) @declaration)
