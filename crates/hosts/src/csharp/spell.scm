; Where prose lives in C#. Captures: @comment, @doc, @string, @declaration and @skip
; (`tree_sitter_host.rs`).

; Comments. A doc comment is XML behind `///`, or a block that opens with `/**`; the grammar
; reads each `///` line as a comment of its own.
((comment) @doc
  (#match? @doc "^(///|/\\*\\*[^/])"))
((comment) @comment
  (#not-match? @comment "^(///|/\\*\\*[^/])"))

; The text of a string, in pieces between its escapes: regular, raw, and the parts of an
; interpolated string around its `{...}`. A verbatim string has no escapes and the grammar gives
; it no content node, so it is read whole, with its `@"` and its doubled quotes.
(string_literal (string_literal_content) @string)
(raw_string_literal (raw_string_content) @string)
(interpolated_string_expression (string_content) @string)
(verbatim_string_literal) @string

; A `using` names a namespace or a type, and an attribute names a type and passes options:
; `[DllImport("libc")]`, `[JsonProperty("user_name")]`.
(using_directive) @skip
(attribute_list) @skip
(global_attribute) @skip

; Names being declared, never a use: a call, a member access and a type are somebody else's. A
; namespace is a product's name, and a constructor or a destructor repeats its type's.
(class_declaration name: (identifier) @declaration)
(struct_declaration name: (identifier) @declaration)
(interface_declaration name: (identifier) @declaration)
(enum_declaration name: (identifier) @declaration)
(record_declaration name: (identifier) @declaration)
(delegate_declaration name: (identifier) @declaration)
(method_declaration name: (identifier) @declaration)
(local_function_statement name: (identifier) @declaration)
(property_declaration name: (identifier) @declaration)
(enum_member_declaration name: (identifier) @declaration)
(type_parameter name: (identifier) @declaration)

; Fields, events and locals all hold a declarator.
(variable_declarator name: (identifier) @declaration)

; Parameters, `catch (Exception e)`, `foreach (var x in xs)` and a deconstruction `var (a, b)`.
(parameter name: (identifier) @declaration)
(catch_declaration name: (identifier) @declaration)
(foreach_statement left: (identifier) @declaration)
(tuple_pattern name: (identifier) @declaration)
(lambda_expression parameters: (implicit_parameter) @declaration)
