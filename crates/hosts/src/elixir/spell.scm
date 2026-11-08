; Where prose lives in Elixir. Captures: @comment, @doc, @string, @value, @skip, @declaration
; (`tree_sitter_host.rs`).

(comment) @comment

; The Markdown of `@doc """..."""`, `@moduledoc` and `@typedoc`, in pieces between its
; interpolations.
(unary_operator
  operand: (call
    target: (identifier) @_attr
    (arguments (string (quoted_content) @doc))
    (#any-of? @_attr "doc" "moduledoc" "typedoc")))

; The text of a string, without its quotes, in pieces between its escapes and interpolations.
(string (quoted_content) @string)

; `~s` and `~S` are strings. A regex, a word list and a template are code: nothing else is
; captured, so `~r/.../`, `~w(...)` and `~H"""..."""` are silent.
(sigil
  (sigil_name) @_name
  (quoted_content) @string
  (#any-of? @_name "s" "S"))

; Atoms and keys are somebody's API as often as they are ours: known typos only.
(atom) @value
(keyword) @value

; Names being declared, never a use, in one pattern: the grammar's `arguments` node is in so many
; places that every pattern naming it costs the host's load about 30 ms, and eight of them were 150
; (2026-11-08, a slow VM). A `def` declares its function, with or without parentheses and a guard,
; and its parameters, plain or with a default; `defmodule` its module. The grammar gives
; `Plug.Conn` as one node, so the whole path is read, parents included: a query cannot cut the last
; part out.
(call
  target: (identifier) @_def
  (arguments
    [
      (identifier) @declaration
      (alias) @declaration
      (call
        target: (identifier) @declaration
        (arguments
          [
            (identifier) @declaration
            (binary_operator left: (identifier) @declaration operator: "\\\\")
          ]))
      (binary_operator
        left: [
          (identifier) @declaration
          (call
            target: (identifier) @declaration
            (arguments
              [
                (identifier) @declaration
                (binary_operator left: (identifier) @declaration operator: "\\\\")
              ]))
        ]
        operator: "when")
    ])
  (#any-of? @_def
    "def" "defp" "defmacro" "defmacrop" "defguard" "defguardp" "defdelegate" "defmodule"
    "defprotocol"))
