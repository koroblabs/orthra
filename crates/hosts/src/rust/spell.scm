; Where prose lives in Rust. Captures: @comment, @doc, @string, @declaration, @test (`tree_sitter_host.rs`).

; Comments. A doc comment (`///`, `//!`, `/** */`, `/*! */`) has a `doc` field.
(line_comment !doc) @comment
(block_comment !doc) @comment
(line_comment doc: (doc_comment)) @doc
(block_comment doc: (doc_comment)) @doc

; The text of a string, in pieces between its escapes: "Hello\nworld" is `Hello` and `world`.
(string_literal (string_content) @string)
(raw_string_literal (string_content) @string)

; Names being declared, never a use: `use serde_json::from_str;` and `from_str(raw)` are somebody
; else's to fix.
(function_item name: (identifier) @declaration)
(function_signature_item name: (identifier) @declaration)
(struct_item name: (type_identifier) @declaration)
(union_item name: (type_identifier) @declaration)
(enum_item name: (type_identifier) @declaration)
(enum_variant name: (identifier) @declaration)
(trait_item name: (type_identifier) @declaration)
(type_item name: (type_identifier) @declaration)
(associated_type name: (type_identifier) @declaration)
(const_item name: (identifier) @declaration)
(static_item name: (identifier) @declaration)
(mod_item name: (identifier) @declaration)
(field_declaration name: (field_identifier) @declaration)
(macro_definition name: (identifier) @declaration)
(let_declaration pattern: (identifier) @declaration)
(parameter pattern: (identifier) @declaration)
(closure_parameters (identifier) @declaration)

; Test code: the strings inside are test data, not text, so they are left out. A test file is
; recognized by its path instead (the engine); these are the tests that live inside a source file.
((attribute_item (attribute (identifier) @_cfg arguments: (token_tree (identifier) @_test)))
  .
  (mod_item) @test
  (#eq? @_cfg "cfg")
  (#eq? @_test "test"))
((attribute_item (attribute (identifier) @_attr))
  .
  (function_item) @test
  (#eq? @_attr "test"))
