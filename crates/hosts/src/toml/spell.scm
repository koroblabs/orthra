; Where prose lives in a TOML file.

(comment) @comment

; A string's quotes and escapes are not words, so the whole node is the span: TOML has no node for
; the text between them.
(string) @string

; Keys are the file's declarations: `[dependancies]` is a typo the author can fix.
(bare_key) @declaration
(quoted_key) @declaration
