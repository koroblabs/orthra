; Where prose lives in Python. Captures: @comment, @doc, @string, @declaration (`tree_sitter_host.rs`).

(comment) @comment

; A docstring is the first statement of a module, a class or a function, when it is a string. Its
; patterns come before the plain string's, so that when both capture the same text, @doc is found
; first and kept.
(module . (expression_statement (string (string_content) @doc)))
(class_definition body: (block . (expression_statement (string (string_content) @doc))))
(function_definition body: (block . (expression_statement (string (string_content) @doc))))

; The text of a string, without its quotes and without an f-string's `{interpolations}`.
(string (string_content) @string)

; Names being declared, never a use: `import json` and `json.loads(raw)` are somebody else's.
(function_definition name: (identifier) @declaration)
(class_definition name: (identifier) @declaration)
(parameters (identifier) @declaration)
(lambda_parameters (identifier) @declaration)
(typed_parameter (identifier) @declaration)
(default_parameter name: (identifier) @declaration)
(typed_default_parameter name: (identifier) @declaration)
(list_splat_pattern (identifier) @declaration)
(dictionary_splat_pattern (identifier) @declaration)
(assignment left: (identifier) @declaration)
(assignment left: (pattern_list (identifier) @declaration))
(for_statement left: (identifier) @declaration)
(aliased_import alias: (identifier) @declaration)
