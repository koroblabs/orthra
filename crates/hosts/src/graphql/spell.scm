; Where prose lives in GraphQL. Captures: @comment, @string, @declaration, @value
; (`tree_sitter_host.rs`). The grammar has no fields, so a definition's own name is its direct
; `(name)` child: a type in `implements`, a field's type and a directive's arguments are deeper.

(comment) @comment

; A description is the documentation of the definition after it.
(description (string_value) @string)

; A string argument in an operation or a default value is a value, held to known typos only.
(value (string_value) @value)

; Names being declared, never a use: a field selected in a query, a named type, a variable's use
; and a spread are somebody else's schema.
(scalar_type_definition (name) @declaration)
(object_type_definition (name) @declaration)
(interface_type_definition (name) @declaration)
(union_type_definition (name) @declaration)
(enum_type_definition (name) @declaration)
(input_object_type_definition (name) @declaration)
(field_definition (name) @declaration)
(input_value_definition (name) @declaration)
(enum_value_definition (enum_value (name) @declaration))
(directive_definition (name) @declaration)
(fragment_definition (fragment_name (name) @declaration))
(operation_definition (name) @declaration)
(variable_definition (variable (name) @declaration))
