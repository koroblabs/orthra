; Where prose lives in an INI file.

(comment) @comment

; What a tox environment or a coverage report runs is code, held to known typos only.
(setting
  (setting_name) @_key
  (setting_value) @value
  (#any-of? @_key "commands" "commands_pre" "commands_post" "deps" "install_command" "allowlist_externals" "setenv" "passenv" "exclude_lines" "exclude_also" "omit" "include"))

; A value is text where it has words: a description, a message. One token is a name, a path or a
; number, and the engine holds it to known typos.
(setting_value) @string

(setting_name) @declaration
(section_name (text) @declaration)
