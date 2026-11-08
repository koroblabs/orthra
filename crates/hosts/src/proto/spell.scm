; Where prose lives in Protocol Buffers. Captures: @comment, @declaration, @value, @skip
; (`tree_sitter_host.rs`).

(comment) @comment

; An import path, the syntax and the edition are values, never text.
(import path: (string) @skip)
(syntax version: (string) @skip)
(edition year: (string) @skip)

; The names an option gives the generated code are derived from the file's own names, and an acronym
; in them (`java_outer_classname = "AlsProto"`) reads as a known typo: not text.
((option (identifier) @_name (constant (string) @skip))
  (#match? @_name "^(go_package|java_package|java_outer_classname|csharp_namespace|objc_class_prefix|php_namespace|php_class_prefix|php_metadata_namespace|ruby_package|swift_prefix)$"))

; Every other string is the value of an option (`go_package`, `java_package`, an HTTP route, a
; custom option's field), held to known typos only.
(constant (string) @value)

; Names being declared, never a use: a field's type and an rpc's request and response are somebody
; else's to fix.
(package (full_ident (identifier) @declaration))
(message_name (identifier) @declaration)
(enum_name (identifier) @declaration)
(service_name (identifier) @declaration)
(rpc_name (identifier) @declaration)
(field (identifier) @declaration)
(oneof (identifier) @declaration)
(oneof_field (identifier) @declaration)
(map_field (identifier) @declaration)
(enum_field (identifier) @declaration)
