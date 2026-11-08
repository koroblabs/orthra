; Where prose lives in Dart. Captures: @comment, @doc, @string, @value, @declaration, @skip
; (`tree_sitter_host.rs`). What is not captured is not checked: a call, a member access, a type and
; an annotation are somebody else's names.

; Comments. `///` and `/** */` are documentation; the grammar tells them apart.
(comment) @comment
(block_comment) @comment
(documentation_comment) @doc
(documentation_block_comment) @doc

; An annotation holds a symbol's name or a tool's setting, not text: `@JsonKey(name: 'user_id')`,
; `@pragma('vm:prefer-inline')`. A `// ignore: unused_element` is a directive to the analyzer.
(annotation) @skip
((comment) @skip
  (#match? @skip "^//\\s*(ignore|ignore_for_file|dart format|coverage:ignore|ignore-file):"))

; A library's name and what a file imports, exports or is a part of are paths and names of other
; people's modules.
(library_name) @skip
(import_or_export) @skip
(part_directive) @skip
(part_of_directive) @skip

; The text of a string, in pieces between its escapes and `$name` or `${...}`. A raw string is
; mostly a pattern or a path, and is held to known typos only.
(string_literal_single_quotes (template_chars_single_single) @string)
(string_literal_double_quotes (template_chars_double_single) @string)
(string_literal_single_quotes_multiple (template_chars_single) @string)
(string_literal_double_quotes_multiple (template_chars_double) @string)
(raw_string_literal_single_quotes (template_chars_single_single) @value)
(raw_string_literal_double_quotes (template_chars_double_single) @value)
(raw_string_literal_single_quotes_multiple (template_chars_single) @value)
(raw_string_literal_double_quotes_multiple (template_chars_double) @value)

; Names being declared, never a use. Types: a class, a mixin, an extension, an enum, a typedef.
(class_declaration name: (identifier) @declaration)
(mixin_declaration name: (identifier) @declaration)
(extension_declaration name: (identifier) @declaration)
(enum_declaration name: (identifier) @declaration)
(enum_entry name: (identifier) @declaration)
(type_alias (type_identifier) @declaration)

; Functions, methods, getters and setters. A constructor's own name is its class's, a use; the
; second name of `Person.named` is declared here.
(function_signature name: (identifier) @declaration)
(getter_signature name: (identifier) @declaration)
(setter_signature name: (identifier) @declaration)
(constructor_signature . name: (identifier) . name: (identifier) @declaration)
(factory_constructor_signature . name: (identifier) . name: (identifier) @declaration)

; Fields and variables, top-level, local and in a loop. A `this.name` parameter is a use of a field.
(initialized_identifier name: (identifier) @declaration)
(initialized_variable_declaration name: (identifier) @declaration)
(static_final_declaration name: (identifier) @declaration)
(for_statement name: (identifier) @declaration)

; A parameter's name.
(formal_parameter name: (identifier) @declaration)
(typed_identifier name: (identifier) @declaration)
