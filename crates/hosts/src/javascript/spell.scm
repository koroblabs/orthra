; Where prose lives in JavaScript and JSX. Captures: @comment, @doc, @string, @declaration and
; @skip (`tree_sitter_host.rs`). The same shapes as TypeScript's query (`typescript/spell.scm`),
; over the node names this grammar uses.

; Comments. A JSDoc block opens with `/**`; `/**/` is an empty comment.
((comment) @doc
  (#match? @doc "^/\\*\\*[^/]"))
((comment) @comment
  (#not-match? @comment "^/\\*\\*[^/]"))

; The text of a string or a template, in pieces between escapes and substitutions. A JSX attribute's
; string is the same node.
(string (string_fragment) @string)
(template_string (string_fragment) @string)

; The text between JSX tags, which a reader sees as it is written.
(jsx_text) @string

; A module path is a string, not prose.
(import_statement source: (string) @skip)
(export_statement source: (string) @skip)
(call_expression
  function: (import)
  arguments: (arguments . (string) @skip))
(call_expression
  function: (identifier) @_require
  arguments: (arguments . (string) @skip)
  (#eq? @_require "require"))

; A regular expression's pattern and its flags (`'gims'`) are a pattern, not prose.
(new_expression
  constructor: (identifier) @_regexp
  arguments: (arguments (string) @skip)
  (#eq? @_regexp "RegExp"))
(call_expression
  function: (identifier) @_regexp
  arguments: (arguments (string) @skip)
  (#eq? @_regexp "RegExp"))

; Names being declared, never a use: `import { readFile } from "node:fs"` and `readFile(path)` are
; somebody else's to fix.
(function_declaration name: (identifier) @declaration)
(generator_function_declaration name: (identifier) @declaration)
(function_expression name: (identifier) @declaration)
(generator_function name: (identifier) @declaration)
(class_declaration name: (identifier) @declaration)
(class name: (identifier) @declaration)
(variable_declarator name: (identifier) @declaration)
(formal_parameters (identifier) @declaration)
(formal_parameters (assignment_pattern left: (identifier) @declaration))
(formal_parameters (rest_pattern (identifier) @declaration))
(arrow_function parameter: (identifier) @declaration)

; Members: class fields and methods.
(field_definition
  property: [(property_identifier) (private_property_identifier)] @declaration)
(class_body
  (method_definition
    name: [(property_identifier) (private_property_identifier)] @declaration))

; The keys of an object literal a variable is declared with; a key passed to a call is somebody
; else's option name.
(variable_declarator
  value: (object
    (pair key: (property_identifier) @declaration)))
(variable_declarator
  value: (object
    (method_definition name: (property_identifier) @declaration)))
