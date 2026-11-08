; Where prose lives in PowerShell. Captures: @comment, @string, @value, @skip, @declaration
; (`tree_sitter_host.rs`). Every cmdlet name and every parameter of a call is somebody else's, so
; none is captured.

; `#`, and `<# ... #>`, which holds comment-based help (`.SYNOPSIS`, `.PARAMETER Name`). `#Requires`
; names modules and versions.
((comment) @skip (#match? @skip "^#[Rr]equires"))
(comment) @comment

; A string is one token in this grammar: its quotes and the `$variables` in it are inside the node,
; so it cannot be cut into the pieces between them. A string with a `$variable` or a `$(...)` in
; it, a backtick escape (`` `n ``, `` `t ``), or a here-string, whose text is usually a program
; or a document in another format, is held to known typos only. The patterns for those come
; first: at the same range, a value wins over a string.
(expandable_string_literal (variable)) @value
(expandable_string_literal (sub_expression)) @value
((expandable_string_literal) @value (#match? @value "`"))
(expandable_here_string_literal) @value
(verbatim_here_string_characters) @value
(expandable_string_literal) @string
(verbatim_string_characters) @string

; Names being declared, never a use: `Get-Item`, `$env:PATH` and `$obj.Method()` are somebody
; else's.
(function_statement (function_name) @declaration)
(script_parameter (variable) @declaration)
(class_statement . (simple_name) @declaration)
(class_property_definition (variable) @declaration)
(class_method_definition (simple_name) @declaration)
(class_method_parameter (variable) @declaration)
(enum_statement . (simple_name) @declaration)
(enum_member (simple_name) @declaration)

; A variable is declared where it is assigned: `$result = Get-Item $path`.
(assignment_expression
  (left_assignment_expression
    (logical_expression
      (bitwise_expression
        (comparison_expression
          (additive_expression
            (multiplicative_expression
              (format_expression
                (range_expression
                  (array_literal_expression
                    (unary_expression (variable) @declaration)))))))))))
