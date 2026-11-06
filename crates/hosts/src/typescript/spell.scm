; Where prose lives in TypeScript. Captures: @comment, @doc, @string, @declaration and @skip
; (`tree_sitter_host.rs`). TSX reads this query too, and adds its own (`tsx/spell.scm`).

; Comments. A JSDoc block opens with `/**`; `/**/` is an empty comment.
((comment) @doc
  (#match? @doc "^/\\*\\*[^/]"))
((comment) @comment
  (#not-match? @comment "^/\\*\\*[^/]"))

; The text of a string or a template, in pieces between escapes and substitutions.
(string (string_fragment) @string)
(template_string (string_fragment) @string)

; A module path is a string, not prose.
(import_statement source: (string) @skip)
(export_statement source: (string) @skip)
(import_require_clause source: (string) @skip)
(call_expression
  function: (import)
  arguments: (arguments . (string) @skip))
(call_expression
  function: (identifier) @_require
  arguments: (arguments . (string) @skip)
  (#eq? @_require "require"))

; Names being declared, never a use: `import { readFile } from "node:fs"` and `readFile(path)` are
; somebody else's to fix.
(function_declaration name: (identifier) @declaration)
(generator_function_declaration name: (identifier) @declaration)
(function_signature name: (identifier) @declaration)
(function_expression name: (identifier) @declaration)
(class_declaration name: (type_identifier) @declaration)
(abstract_class_declaration name: (type_identifier) @declaration)
(class name: (type_identifier) @declaration)
(interface_declaration name: (type_identifier) @declaration)
(type_alias_declaration name: (type_identifier) @declaration)
(enum_declaration name: (identifier) @declaration)
(enum_body name: (property_identifier) @declaration)
(enum_assignment name: (property_identifier) @declaration)
(internal_module name: (identifier) @declaration)
(variable_declarator name: (identifier) @declaration)
(required_parameter pattern: (identifier) @declaration)
(optional_parameter pattern: (identifier) @declaration)
(arrow_function parameter: (identifier) @declaration)

; Members: class fields and methods, interface and type-literal members.
(public_field_definition
  name: [(property_identifier) (private_property_identifier)] @declaration)
(class_body
  (method_definition
    name: [(property_identifier) (private_property_identifier)] @declaration))
(abstract_method_signature name: (property_identifier) @declaration)
(method_signature name: (property_identifier) @declaration)
(property_signature name: (property_identifier) @declaration)

; The keys of an object literal a variable is declared with; a key passed to a call is somebody
; else's option name.
(variable_declarator
  value: (object
    (pair key: (property_identifier) @declaration)))
(variable_declarator
  value: (object
    (method_definition name: (property_identifier) @declaration)))
