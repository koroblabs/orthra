; Where prose lives in a PHP file.
;
; Text outside `<?php ... ?>` is HTML, and no host here reads HTML yet, so it is left unchecked.

; A `/** */` docblock documents what follows it; `//`, `#` and `/* */` are comments.
((comment) @doc (#match? @doc "^/\\*\\*"))
((comment) @comment (#not-match? @comment "^/\\*\\*"))

; What `include` and `require` name is a path, not text.
(include_expression) @skip
(include_once_expression) @skip
(require_expression) @skip
(require_once_expression) @skip
; A backtick string is a shell command.
(shell_command_expression) @skip

; The text of a string, without its quotes, escapes and interpolations.
(string (string_content) @string)
(encapsed_string (string_content) @string)
(heredoc_body (string_content) @string)
(nowdoc_body (nowdoc_string) @string)

; Declared names: where an identifier is the author's to fix.
(namespace_definition name: (namespace_name (name) @declaration))
(namespace_use_clause alias: (name) @declaration)
(function_definition name: (name) @declaration)
(class_declaration name: (name) @declaration)
(interface_declaration name: (name) @declaration)
(trait_declaration name: (name) @declaration)
(enum_declaration name: (name) @declaration)
(enum_case name: (name) @declaration)
(method_declaration name: (name) @declaration)
(property_element name: (variable_name (name) @declaration))
(const_element (name) @declaration)
(simple_parameter name: (variable_name (name) @declaration))
(variadic_parameter name: (variable_name (name) @declaration))
(property_promotion_parameter name: (variable_name (name) @declaration))
(assignment_expression left: (variable_name (name) @declaration))
; `foreach ($rows as $key => &$row)`: the first child is what is walked, and only what follows it is
; declared.
(foreach_statement . (_) . (variable_name (name) @declaration))
(foreach_statement . (_) . (by_ref (variable_name (name) @declaration)))
(foreach_statement . (_) . (pair (variable_name (name) @declaration)))
(foreach_statement . (_) . (pair (by_ref (variable_name (name) @declaration))))
(catch_clause name: (variable_name (name) @declaration))
(static_variable_declaration name: (variable_name (name) @declaration))
