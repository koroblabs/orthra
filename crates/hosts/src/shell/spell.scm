; Where prose lives in a shell script. Captures: @comment, @string, @declaration, @value
; (`tree_sitter_host.rs`). Every command word and every unquoted argument is somebody else's program
; and somebody else's flags, so none is captured.

; Comments, but not the `#!/usr/bin/env bash` line, which names an interpreter.
((comment) @comment
  (#not-match? @comment "^#!"))

; The text of a double-quoted string, in the pieces around `$var`, `${...}` and `$(...)`.
(string (string_content) @string)

; A single-quoted string is raw text, and in a script it is nearly always a program for another
; command: a `jq` filter, an `awk` script, a `grep` pattern, JSON. Known typos only. `$'...'` is
; what a script writes when it means text with escapes in it.
(raw_string) @value
(ansi_c_string) @string

; A heredoc body is not captured: in these scripts it is a systemd unit, a config file, a Python
; program, the output a test expects or the input it feeds. It is somebody else's format.

; Names being declared, never a use: `$FOO`, `cd "$dir"` and `git status` are not captured.
(function_definition name: (word) @declaration)
(variable_assignment name: (variable_name) @declaration)
(variable_assignment name: (subscript name: (variable_name) @declaration))
(declaration_command (variable_name) @declaration)
(for_statement variable: (variable_name) @declaration)
