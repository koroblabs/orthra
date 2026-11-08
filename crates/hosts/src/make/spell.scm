; Where prose lives in a makefile. Captures: @comment, @string, @declaration (`tree_sitter_host.rs`).

(comment) @comment

; The messages a makefile prints for the people who run it.
(function_call
  function: _ @_function
  (arguments . (text) @string)
  (#any-of? @_function "info" "warning" "error"))

; Variables are declared where they are assigned, `export`, `override` and `define` included. A
; reference, `$(CC)`, is a use and is not captured.
;
; Not captured: a variable's value, and recipes. They are flags, paths and shell, and as `@value`
; they would report a known typo inside a `$(reference)` too, which is a use of a name declared
; (and reported) elsewhere. A nested `@skip` does not carve a reference out of an outer capture.
(variable_assignment . (word) @declaration)
(define_directive name: (word) @declaration)
