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
; a struct_declarator. Where a child is `(_)` kept only when it is a plain name, one with no space
; or ASCII punctuation but `_` and `$` in it, it stands for `(identifier)`, which tree-sitter takes
; far longer to compile against this grammar: the `declaration` pattern alone cost 46 ms of the
; query's 94, which now compiles in 42 (2026-11-09).
((declaration declarator: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((init_declarator declarator: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((function_declarator declarator: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((pointer_declarator declarator: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((array_declarator declarator: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((parenthesized_declarator (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((parameter_declaration declarator: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((field_declaration declarator: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((struct_declarator (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((type_definition declarator: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))

((struct_specifier name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((union_specifier name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((enum_specifier name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((enumerator name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))

((preproc_def name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((preproc_function_def name: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((preproc_params (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))

; A class and a protocol are named where they are declared. `@interface Greeter (Extras)` reopens
; a class that is declared elsewhere, so only the category's name is its own. A class extension,
; `@interface Greeter ()`, looks like a class to the grammar and names the class again; an
; `@implementation` names a class its `@interface` declared, and is not captured.
((class_interface !category . (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((class_interface category: (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((protocol_declaration . (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))

; A selector is declared in an `@interface` or `@protocol`: `greet:times:` is its keyword parts
; and each parameter's name. A method defined in an `@implementation` is declared in the header
; or inherited, so only its parameters are named there, which are its own.
((method_declaration (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
((method_parameter (_) @declaration)
  (#not-match? @declaration "[\\s!-#%-/:-@\\[-^`{-~]"))
