; Where prose lives in C. Captures: @comment, @doc, @string, @declaration, @skip
; (`tree_sitter_host.rs`). What is not captured is not checked: a call, a member access, a type
; and a macro use are somebody else's names.

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

; The text of a string, in pieces between its escapes: `"%s: %d\n"` is `%s: %d`.
(string_literal (string_content) @string)

; An include path, and the name `extern "C"` gives a linkage, are not prose.
(preproc_include path: (_) @skip)
(linkage_specification value: (string_literal) @skip)
(gnu_asm_expression (string_literal) @skip)

; Names being declared, never a use. A declarator nests (`char *names[4]`, `int (*handler)(int)`),
; and each node of the nesting is one of these, so a name is found under whichever it sits.
(declaration declarator: [(identifier) (type_identifier)] @declaration)
(init_declarator declarator: (identifier) @declaration)
(function_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
(pointer_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
(array_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
(parenthesized_declarator [(identifier) (field_identifier) (type_identifier)] @declaration)
(parameter_declaration declarator: (identifier) @declaration)
(field_declaration declarator: (field_identifier) @declaration)
(type_definition declarator: (type_identifier) @declaration)

; Struct, union and enum names, and an enum's constants.
(struct_specifier name: (type_identifier) @declaration)
(union_specifier name: (type_identifier) @declaration)
(enum_specifier name: (type_identifier) @declaration)
(enumerator name: (identifier) @declaration)

; A macro and its parameters. Its body is raw text to the grammar, and so is not captured.
(preproc_def name: (identifier) @declaration)
(preproc_function_def name: (identifier) @declaration)
(preproc_params (identifier) @declaration)
