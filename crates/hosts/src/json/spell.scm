; Where prose lives in a JSON file.

; JSONC's comments: `tsconfig.json`, VS Code's settings.
(comment) @comment

; A key is a declaration: `"dependancies"` is a typo the author can fix.
(pair key: (string) @declaration)

; The commands of npm's and composer's `"scripts"` are code, held to known typos only.
(pair
  key: (string (string_content) @_key)
  value: (object) @value
  (#eq? @_key "scripts"))

; Every other string. The whole node, quotes and escapes too, so that whether it is one token is
; decided for the string and not for each piece an escape leaves.
(pair value: (string) @string)
(array (string) @string)
(document (string) @string)
