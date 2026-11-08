; Where prose lives in a CMake file. Captures: @comment, @string, @declaration (`tree_sitter_host.rs`).
; Commands and their arguments are CMake's and the toolchain's, so most are not captured.

(line_comment) @comment
(bracket_comment) @comment

; The text of a message.
(normal_command
  (identifier) @_command
  (argument_list (argument (quoted_argument (quoted_element) @string)))
  (#match? @_command "^(?i:message)$"))

; The documentation of `option(NAME "doc" ON)` and of `set(NAME value CACHE TYPE "doc")`, and the
; text after `COMMENT` and `DESCRIPTION`.
(normal_command
  (identifier) @_command
  (argument_list
    (argument (unquoted_argument))
    .
    (argument (quoted_argument (quoted_element) @string)))
  (#match? @_command "^(?i:option)$"))
(normal_command
  (identifier) @_command
  (argument_list
    (argument (unquoted_argument) @_cache)
    .
    (argument (unquoted_argument))
    .
    (argument (quoted_argument (quoted_element) @string)))
  (#match? @_command "^(?i:set)$")
  (#eq? @_cache "CACHE"))
(argument_list
  (argument (unquoted_argument) @_keyword)
  .
  (argument (quoted_argument (quoted_element) @string))
  (#any-of? @_keyword "COMMENT" "DESCRIPTION"))

; `function(name args...)` and `macro(name args...)` declare a name and its parameters.
(function_command (argument_list (argument (unquoted_argument) @declaration)))
(macro_command (argument_list (argument (unquoted_argument) @declaration)))

; The variable `set(NAME ...)` and `option(NAME ...)` define: the first argument, unless it is
; one of CMake's own (`CMAKE_CXX_STANDARD`), which is a use of somebody else's API.
(normal_command
  (identifier) @_command
  (argument_list . (argument (unquoted_argument) @declaration))
  (#match? @_command "^(?i:set|option)$")
  (#not-match? @declaration "^(CMAKE_|_)"))
