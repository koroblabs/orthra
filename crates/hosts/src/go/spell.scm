; Where prose lives in Go. Captures: @comment, @doc, @string, @declaration, @skip
; (`tree_sitter_host.rs`).

; A directive to the toolchain, not prose: `//go:build linux || shbe`, `//go:generate go run
; gen.go`, `//go:linkname`, the old `// +build`, `//line`, `//export`, a linter's `//nolint`. In the
; standard library they were 95 findings, `shbe`, `gojs`, `decgen` (2026-11-09).
((comment) @skip
  (#match? @skip "^//(go:|line |export |extern |nolint|lint:| ?\\+build )"))

; A doc comment is the comments that precede a declaration, and gofmt writes its code blocks
; indented by a tab: `//\tx := f()`, which the doc's rules leave out.
((comment)+ @doc
  .
  [(package_clause) (function_declaration) (method_declaration) (type_declaration)
   (const_declaration) (var_declaration)])

(comment) @comment

; An import path is a module path, and a struct tag (`json:"user_name,omitempty"`) holds keys: not
; text either way.
(import_spec path: (_) @skip)
(field_declaration tag: (_) @skip)

; The text of a string, in pieces between its escapes.
(interpreted_string_literal (interpreted_string_literal_content) @string)
(raw_string_literal (raw_string_literal_content) @string)

; Names being declared, never a use: `fmt.Println` is somebody else's to fix.
(package_clause (package_identifier) @declaration)
(import_spec name: (package_identifier) @declaration)
(function_declaration name: (identifier) @declaration)
(method_declaration name: (field_identifier) @declaration)
(method_elem name: (field_identifier) @declaration)
(type_spec name: (type_identifier) @declaration)
(field_declaration name: (field_identifier) @declaration)
(parameter_declaration name: (identifier) @declaration)
(variadic_parameter_declaration name: (identifier) @declaration)
(var_spec name: (identifier) @declaration)
(const_spec name: (identifier) @declaration)
(short_var_declaration left: (expression_list (identifier) @declaration))
(range_clause left: (expression_list (identifier) @declaration))
