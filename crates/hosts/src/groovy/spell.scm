; Where prose lives in Groovy. Captures: @comment, @doc, @string, @declaration and @skip
; (`tree_sitter_host.rs`).

; Comments. Groovydoc opens with `/**`; `/**/` is an empty comment.
((block_comment) @doc
  (#match? @doc "^/\\*\\*[^/]"))
((block_comment) @comment
  (#not-match? @comment "^/\\*\\*[^/]"))
(line_comment) @comment

; The text of a string and of a triple-quoted one, in pieces between escapes. A single-quoted
; string is a `character_literal`, whole.
(string_literal (string_fragment) @string)
(string_literal (multiline_string_fragment) @string)
(character_literal) @string

; A package or an import is a dotted path, not prose; so is an annotation.
(package_declaration) @skip
(import_declaration) @skip
(annotation) @skip
(marker_annotation) @skip

; Names being declared, never a use: a call and a field access are somebody else's.
(class_declaration name: (identifier) @declaration)
(interface_declaration name: (identifier) @declaration)
(enum_declaration name: (identifier) @declaration)
(enum_constant name: (identifier) @declaration)
(method_declaration name: (identifier) @declaration)
(function_definition name: (identifier) @declaration)
(constructor_declaration name: (identifier) @declaration)

; Fields. A local declares a name only with a value or a `def`: the grammar reads `println message`
; as a local `message` of type `println`, and that is a use.
(field_declaration declarator: (variable_declarator name: (identifier) @declaration))
(variable_declarator name: (identifier) @declaration value: (_))
(local_variable_declaration
  type: (_) @_type
  declarator: (variable_declarator name: (identifier) @declaration)
  (#any-of? @_type "def" "var"))

(formal_parameter name: (identifier) @declaration)
(catch_formal_parameter name: (identifier) @declaration)
(enhanced_for_statement name: (identifier) @declaration)
(lambda_expression parameters: (identifier) @declaration)
