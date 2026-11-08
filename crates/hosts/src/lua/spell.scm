; Where prose lives in Lua. Captures: @comment, @doc, @string, @skip, @declaration
; (`tree_sitter_host.rs`).

; `---` starts a LuaLS doc comment (`---@param`, `--- Text`); a row of dashes is a rule, which the
; doc reader leaves empty. Every other `--` and `--[[ ]]` comment is a comment.
((comment) @doc (#match? @doc "^---"))
((comment) @comment (#not-match? @comment "^---"))

; A module name is a path, not text: `require("lazy.core.util")`.
(function_call
  name: (identifier) @_call
  arguments: (arguments (string) @skip)
  (#eq? @_call "require"))

; So is the one `pcall(require, "pl.lexer")` gives it.
(function_call
  name: (identifier) @_call
  arguments: (arguments (identifier) @_require (string) @skip)
  (#eq? @_call "pcall")
  (#eq? @_require "require"))

; The text of a string, without its quotes, in pieces between its escapes.
(string content: (string_content) @string)

; Names being declared, never a use: `M.setup = ...` and `vim.api.nvim_buf_get_lines()` are
; somebody else's. `function M.foo()` and `function obj:foo()` declare their last name.
(function_declaration name: (identifier) @declaration)
(function_declaration name: (dot_index_expression field: (identifier) @declaration))
(function_declaration name: (method_index_expression method: (identifier) @declaration))
(parameters name: (identifier) @declaration)
(variable_declaration (variable_list name: (identifier) @declaration))
(variable_declaration (assignment_statement (variable_list name: (identifier) @declaration)))
