; Where prose lives in SCSS. Captures: @comment, @string, @declaration, @value (`tree_sitter_host.rs`).
; What is not captured is not checked: `@use` and `@import` paths, `url()`, an attribute selector's
; value, and every use: `@include`, `@extend`, `$var` in a value, function calls.

(comment) @comment
(js_comment) @comment

; Names the stylesheet declares, never a use. The grammar gives a plain class name no children, and
; one with interpolation (`.card-#{$name}`) an `identifier` for each text piece around it, so the
; interpolated variable is never read as a word.
(class_selector (class_name) @declaration
  (#not-match? @declaration "#\\{"))
(class_selector (class_name (identifier) @declaration))
; An id with an interpolation is not parsed as one (`#nav-#{$id}` is `#nav-` and a tag), so an
; id name never holds a variable.
(id_selector (id_name) @declaration)
(keyframes_name) @declaration
; `@extend .other` names a class somebody declared, a use.
(extend_statement) @skip
; `%placeholder` where it is defined; `@extend %placeholder` is a use and is not under `selectors`.
(selectors (placeholder (identifier) @declaration))

; `$var: ...` and `--custom-property: ...` definitions.
((declaration (property_name) @declaration)
  (#match? @declaration "^(\\$|--)[A-Za-z_]"))

; `@mixin name($parameter)` and `@function name($parameter)`.
(mixin_statement name: (identifier) @declaration)
(function_statement name: (identifier) @declaration)
(parameters (parameter . (variable) @declaration))

; An ordinary property is the language's, held to known typos only: `widht`. A vendor prefix
; (`-webkit-`) and an interpolated name (`#{$side}-top`) are left alone.
((declaration (property_name) @value)
  (#match? @value "^[a-z]"))

; The text of a `content:`. A string with an interpolation holds a variable's name, a use.
(declaration
  (property_name) @_property
  (string_value) @string
  (#eq? @_property "content")
  (#not-match? @string "#\\{"))
