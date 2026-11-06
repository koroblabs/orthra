; Where prose lives in Go. Captures: @comment, @string, @declaration, @skip (`tree_sitter_host.rs`).
; Go has no doc-comment syntax: a doc comment is a comment that precedes a declaration.

(comment) @comment

; An import path is a module path, and a struct tag (`json:"user_name,omitempty"`) holds keys: not
; text either way.
(import_spec path: (_) @skip)
(field_declaration tag: (_) @skip)

; The text of a string, in pieces between its escapes.
(interpreted_string_literal (interpreted_string_literal_content) @string)
(raw_string_literal (raw_string_literal_content) @string)

; Names being declared, never a use: `fmt.Println` is somebody else's to fix.
(package_clause (package_identifier) @declaration)
(import_spec name: (package_identifier) @declaration)
(function_declaration name: (identifier) @declaration)
(method_declaration name: (field_identifier) @declaration)
(method_elem name: (field_identifier) @declaration)
(type_spec name: (type_identifier) @declaration)
(field_declaration name: (field_identifier) @declaration)
(parameter_declaration name: (identifier) @declaration)
(variadic_parameter_declaration name: (identifier) @declaration)
(var_spec name: (identifier) @declaration)
(const_spec name: (identifier) @declaration)
(short_var_declaration left: (expression_list (identifier) @declaration))
(range_clause left: (expression_list (identifier) @declaration))
