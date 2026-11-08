; Where prose lives in Haskell. Captures: @comment, @doc, @string, @declaration
; (`tree_sitter_host.rs`).

; `-- |`, `-- ^`, `{-|` and `{-^` are Haddock. Every other `--` and `{- -}` is a comment. A pragma
; (`{-# LANGUAGE ... #-}`) is a node of its own, and nothing here asks for it.
(haddock) @doc
(comment) @comment

; The text of a string, with its escapes read by the segmenter. A character (`'a'`) is not one.
(string) @string

; Names being declared, never a use. A signature (`f :: Int`) repeats the name its equation
; declares, so only the equation is captured, and an instance's methods are the class's to declare.
(declarations (function name: (variable) @declaration))
(declarations (bind name: (variable) @declaration))
(local_binds decl: (function name: (variable) @declaration))
(local_binds decl: (bind name: (variable) @declaration))
(class_declarations declaration: (signature name: (variable) @declaration))

; Parameters.
(patterns (variable) @declaration)

; Types, constructors and fields.
(data_type name: (name) @declaration)
(newtype name: (name) @declaration)
(type_synonym name: (name) @declaration)
(class name: (name) @declaration)
(prefix name: (constructor) @declaration)
(record name: (constructor) @declaration)
(newtype_constructor name: (constructor) @declaration)
(field name: (field_name) @declaration)
