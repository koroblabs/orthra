; Where prose lives in R. Captures: @comment, @doc, @string, @skip, @declaration
; (`tree_sitter_host.rs`).

; `#'` starts a roxygen comment, which is Markdown with `@tags`. Every other `#` is a comment.
((comment) @doc (#match? @doc "^#'"))
((comment) @comment (#not-match? @comment "^#'"))

; A package, a file or a namespace is named by a string, not described by it: `library("dplyr")`.
(call
  function: (identifier) @_call
  arguments: (arguments (argument value: (string) @skip))
  (#any-of? @_call
    "library" "require" "requireNamespace" "loadNamespace" "source" "sys.source" "file.path"
    "system.file"))

; The text of a string, without its quotes, in pieces between its escapes.
(string content: (string_content) @string)

; Names being declared, never a use: `df$col <- 1`, `names(x) <- y` and `pkg::fn()` are somebody
; else's. Assigning with `<-`, `<<-` or `=` declares the name on the left, `->` the one on the
; right.
(binary_operator
  lhs: (identifier) @declaration
  operator: ["<-" "<<-" "="])
(binary_operator
  operator: ["->" "->>"]
  rhs: (identifier) @declaration)
(parameter name: (identifier) @declaration)
