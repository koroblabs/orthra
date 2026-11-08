; Where prose lives in Java. Captures: @comment, @doc, @string, @declaration and @skip
; (`tree_sitter_host.rs`).

; Comments. A Javadoc block opens with `/**`; `/**/` is an empty comment.
((block_comment) @doc
  (#match? @doc "^/\\*\\*[^/]"))
((block_comment) @comment
  (#not-match? @comment "^/\\*\\*[^/]"))
(line_comment) @comment

; The text of a string and of a text block, in pieces between escapes.
(string_literal (string_fragment) @string)
(string_literal (multiline_string_fragment) @string)

; A package or an import is a dotted path, not prose.
(package_declaration) @skip
(import_declaration) @skip

; An annotation names a type and passes options: `@SuppressWarnings("unchecked")`.
(annotation) @skip
(marker_annotation) @skip

; Names being declared, never a use: a call, a field access and a type are somebody else's.
(class_declaration name: (identifier) @declaration)
(interface_declaration name: (identifier) @declaration)
(enum_declaration name: (identifier) @declaration)
(record_declaration name: (identifier) @declaration)
(annotation_type_declaration name: (identifier) @declaration)
(annotation_type_element_declaration name: (identifier) @declaration)
(method_declaration name: (identifier) @declaration)
(constructor_declaration name: (identifier) @declaration)
(compact_constructor_declaration name: (identifier) @declaration)
(enum_constant name: (identifier) @declaration)

; Fields, constants, locals and a varargs parameter all hold a declarator.
(variable_declarator name: (identifier) @declaration)

; Parameters, record components, `catch (Exception e)`, `for (T x : xs)` and `try (T r = ...)`.
(formal_parameter name: (identifier) @declaration)
(catch_formal_parameter name: (identifier) @declaration)
(enhanced_for_statement name: (identifier) @declaration)
(resource name: (identifier) @declaration)
(inferred_parameters (identifier) @declaration)
(lambda_expression parameters: (identifier) @declaration)
