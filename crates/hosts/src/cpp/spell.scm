; Where prose lives in C++. Captures: @comment, @doc, @string, @declaration, @skip
; (`tree_sitter_host.rs`). The shapes of C's query (`c/spell.scm`), over this grammar, and what C++
; adds. A call, a member access, a type, a macro use and a qualified name (`std::string`, and the
; `Widget::` of an out-of-class definition) are somebody else's names, or declared elsewhere.

; Comments. A Doxygen comment opens with `/**`, `/*!`, `///` or `//!`; `/**/` and `////` are not.
((comment) @doc
  (#match? @doc "^(/\\*\\*([^/*]|$)|/\\*!|///([^/]|$)|//!)"))
((comment) @comment
  (#not-match? @comment "^(/\\*\\*([^/*]|$)|/\\*!|///([^/]|$)|//!)"))

; A comment that is a directive to a tool, not prose: `// NOLINT(cppcoreguidelines-avoid-goto)`,
; `// clang-format off`, `// IWYU pragma: keep`, `// SPDX-License-Identifier: MIT`.
; nlohmann/json has 150 of them.
((comment) @skip
  (#match? @skip "^(//|/\\*)\\s*(NOLINT|clang-format|clang-tidy|cppcheck-suppress|IWYU|coverity|LCOV_EXCL|SPDX-)"))

; The text of a string, in pieces between its escapes, and of a raw string, which has none.
(string_literal (string_content) @string)
(raw_string_literal (raw_string_content) @string)

; An include path, and the name `extern "C"` gives a linkage, are not prose.
(preproc_include path: (_) @skip)
(linkage_specification value: (string_literal) @skip)
(gnu_asm_expression (string_literal) @skip)

; Names being declared, never a use. A declarator nests (`char *names[4]`, `Widget &widget`), and
; each node of the nesting is one of these, so a name is found under whichever it sits.
(declaration declarator: [(identifier) (type_identifier)] @declaration)
(init_declarator declarator: (identifier) @declaration)
(function_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
; A constructor is a function with no return type, named for its class: the class's name is declared
; once, at the class, and `Greeter(...)` would report a typo in it twice. A skip wins over the
; declaration at the same range.
(function_definition !type
  declarator: (function_declarator declarator: [(identifier) (field_identifier)] @skip))
(declaration !type
  declarator: (function_declarator declarator: [(identifier) (field_identifier)] @skip))
(field_declaration !type
  declarator: (function_declarator declarator: [(identifier) (field_identifier)] @skip))
(pointer_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
(array_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
(reference_declarator [(identifier) (field_identifier) (type_identifier)] @declaration)
(variadic_declarator [(identifier) (field_identifier)] @declaration)
(parenthesized_declarator [(identifier) (field_identifier) (type_identifier)] @declaration)
(structured_binding_declarator (identifier) @declaration)
(for_range_loop declarator: (identifier) @declaration)
(parameter_declaration declarator: (identifier) @declaration)
(optional_parameter_declaration declarator: (identifier) @declaration)
(field_declaration declarator: (field_identifier) @declaration)
(type_definition declarator: (type_identifier) @declaration)

; Classes, structs, unions and enums, and an enum's constants.
(class_specifier name: (type_identifier) @declaration)
(struct_specifier name: (type_identifier) @declaration)
(union_specifier name: (type_identifier) @declaration)
(enum_specifier name: (type_identifier) @declaration)
(enumerator name: (identifier) @declaration)

; Namespaces, with the parts of `namespace a::b`, and their aliases.
(namespace_definition name: (namespace_identifier) @declaration)
(namespace_definition name: (nested_namespace_specifier (namespace_identifier) @declaration))
(namespace_alias_definition name: (namespace_identifier) @declaration)

; A `using` alias, a concept, and the parameters of a template.
(alias_declaration name: (type_identifier) @declaration)
(concept_definition name: (identifier) @declaration)
(type_parameter_declaration (type_identifier) @declaration)
(variadic_type_parameter_declaration (type_identifier) @declaration)
(optional_type_parameter_declaration name: (type_identifier) @declaration)

; A macro and its parameters. Its body is raw text to the grammar, and so is not captured.
(preproc_def name: (identifier) @declaration)
(preproc_function_def name: (identifier) @declaration)
(preproc_params (identifier) @declaration)
