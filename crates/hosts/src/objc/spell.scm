; Where prose lives in Objective-C. Captures: @comment, @doc, @string, @declaration, @skip
; (`tree_sitter_host.rs`). This is C's query (`c/spell.scm`), whose grammar this one extends, and
; what Objective-C adds. What is not captured is not checked: a message send, a property access
; and a type are somebody else's names.

; Comments. A Doxygen or HeaderDoc comment opens with `/**`, `/*!`, `///` or `//!`.
((comment) @doc
  (#match? @doc "^(/\\*\\*([^/*]|$)|/\\*!|///([^/]|$)|//!)"))
((comment) @comment
  (#not-match? @comment "^(/\\*\\*([^/*]|$)|/\\*!|///([^/]|$)|//!)"))

; A comment that is a directive to a tool: `// NOLINT`, `// clang-format off`,
; `// SPDX-License-Identifier: MIT`.
((comment) @skip
  (#match? @skip "^(//|/\\*)\\s*(NOLINT|clang-format|clang-tidy|cppcheck-suppress|IWYU|coverity|LCOV_EXCL|SPDX-)"))

; The text of a string, `"..."` or `@"..."`, in pieces between its escapes.
(string_literal (string_content) @string)

; An include or import path is not prose, and nor is an extern's linkage.
(preproc_include path: (_) @skip)
(linkage_specification value: (string_literal) @skip)
(gnu_asm_expression (string_literal) @skip)

; C's declarations. A declarator nests (`NSString *names[4]`), and each node of the nesting is one
; of these, so a name is found under whichever it sits. A property and an instance variable are
; a struct_declarator.
(declaration declarator: [(identifier) (type_identifier)] @declaration)
(init_declarator declarator: (identifier) @declaration)
(function_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
(pointer_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
(array_declarator declarator: [(identifier) (field_identifier) (type_identifier)] @declaration)
(parenthesized_declarator [(identifier) (field_identifier) (type_identifier)] @declaration)
(parameter_declaration declarator: (identifier) @declaration)
(field_declaration declarator: (field_identifier) @declaration)
(struct_declarator (identifier) @declaration)
(type_definition declarator: (type_identifier) @declaration)

(struct_specifier name: (type_identifier) @declaration)
(union_specifier name: (type_identifier) @declaration)
(enum_specifier name: (type_identifier) @declaration)
(enumerator name: (identifier) @declaration)

(preproc_def name: (identifier) @declaration)
(preproc_function_def name: (identifier) @declaration)
(preproc_params (identifier) @declaration)

; A class and a protocol are named where they are declared. `@interface Greeter (Extras)` reopens
; a class that is declared elsewhere, so only the category's name is its own. A class extension,
; `@interface Greeter ()`, looks like a class to the grammar and names the class again; an
; `@implementation` names a class its `@interface` declared, and is not captured.
(class_interface !category . (identifier) @declaration)
(class_interface category: (identifier) @declaration)
(protocol_declaration . (identifier) @declaration)

; A selector is declared in an `@interface` or `@protocol`: `greet:times:` is its keyword parts
; and each parameter's name. A method defined in an `@implementation` is declared in the header
; or inherited, so only its parameters are named there, which are its own.
(method_declaration (identifier) @declaration)
(method_parameter (identifier) @declaration)
