; Where prose lives in Ruby. Captures: @comment, @doc, @string, @declaration, @skip
; (`tree_sitter_host.rs`). A method call is somebody else's API, so none is captured; the two
; exceptions are `attr_accessor :name`, which declares what it names, and `require`, whose path is
; skipped.
;
; The patterns are few and use alternations on purpose: this grammar is large, and the query
; analysis is paid per pattern. As separate patterns, the declarations of methods, classes and
; modules cost the grammar load 45 ms and as alternations 23; the two calls cost 36 and as one 17.

; An indented comment is a verbatim block, in RDoc, YARD and Markdown alike: `#   "foo".indent(2)`
; is an example, and nothing in it is prose. A comment of its own line skips whole, being one node.
((comment) @skip
  (#match? @skip "^#[ ]{3,}[^ \t]"))

; A comment line that starts a YARD tag is read as a doc comment, so that `@param name [Type]` is
; read with the tag's type left out (`DocTags`). Ruby has no doc syntax: a comment above a `def` is
; a doc only by convention, and a pattern that looked at the `def` below would depend on a sibling,
; which an incremental query does not see. The test is the comment's own text.
((comment) @doc
  (#match? @doc "^#[ \t]*@[!a-z]"))

; Comments, but not the interpreter line, nor the magic comments that tools read, which are
; `key: value` directives and not prose, nor a line that is only an RDoc or SimpleCov directive,
; `# :nodoc:`, `# :stopdoc:`, `# :nocov:`.
((comment) @comment
  (#not-match? @comment "^#!")
  (#not-match? @comment "^#[ \t]*(-\\*-|frozen_string_literal:|typed:|encoding:|coding:|warn_indent:|shareable_constant_value:)")
  (#not-match? @comment "^#[ \t]*:[a-z_]+:[ \t]*$"))

; The text of a string, in pieces between its escapes and its `#{interpolation}`. A symbol, a
; regular expression, a `%w[]` word and a command are not strings of this rule and are not read.
(string (string_content) @string)

; The two calls that matter: a path that is loaded is not text (`require "active_support/core_ext"`),
; and `attr_accessor :name, :other` declares readers and writers by their symbols.
((call
   !receiver
   method: (identifier) @_m
   arguments: (argument_list [(string) @skip (simple_symbol) @declaration]))
  (#match? @_m "^(attr_reader|attr_writer|attr_accessor|require|require_relative|load|autoload)$"))

; A heredoc body is not captured. In Ruby it is an HTML page, a SQL statement, a usage text or a
; template. Captured, sinatra's and rack's 264 files went from 41 findings to 58, and all 17 new
; ones were identifiers of the HTML and CSS inside (`toggler`, `quicklinks`, `requestinfo`,
; `NOARCHIVE`): none was a typo.

; Names being declared, never a use. A hash key, a call and a constant that is read are not.
(method name: [(identifier) @declaration (constant) @declaration (setter (identifier) @declaration)])
(singleton_method name: [(identifier) @declaration (constant) @declaration (setter (identifier) @declaration)])
(class name: [(constant) @declaration (scope_resolution name: (constant) @declaration)])
(module name: [(constant) @declaration (scope_resolution name: (constant) @declaration)])

; Parameters: plain, optional, keyword, splat, double splat and block ones, of a method, a block
; and a lambda.
[
  (method_parameters (identifier) @declaration)
  (block_parameters (identifier) @declaration)
  (lambda_parameters (identifier) @declaration)
  (destructured_parameter (identifier) @declaration)
  (optional_parameter name: (identifier) @declaration)
  (keyword_parameter name: (identifier) @declaration)
  (splat_parameter name: (identifier) @declaration)
  (hash_splat_parameter name: (identifier) @declaration)
  (block_parameter name: (identifier) @declaration)
]

; Assignments: a local, a constant, and what `a, b = ...` and `x ||= ...` name.
[
  (assignment left: [(identifier) @declaration (constant) @declaration (left_assignment_list (identifier) @declaration)])
  (operator_assignment left: [(identifier) @declaration (constant) @declaration])
]
