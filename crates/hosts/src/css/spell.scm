; Where prose lives in CSS. Captures: @comment, @string, @declaration, @value (`tree_sitter_host.rs`).
; What is not captured is not checked: a `url()`, an `@import`, an attribute selector's value, a
; font family and `grid-template-areas` are strings that are values, so they have no pattern.

(comment) @comment

; Names the stylesheet declares, never a use: `var(--brand-color)`, `:hover`, `div` and
; `font-weight` are somebody else's to fix. A class name is cut at its escapes, so Tailwind's
; `.sm\:flex` is `sm` and `flex`. `:hover` has a `class_name` too, and is not one of these.
(class_selector (class_name (identifier) @declaration))
(id_selector (id_name) @declaration)
(keyframes_name) @declaration
((declaration (property_name) @declaration)
  (#match? @declaration "^--"))

; An ordinary property is the language's, held to known typos only: `widht`. A vendor prefix
; (`-webkit-`) is left alone.
((declaration (property_name) @value)
  (#match? @value "^[a-z]"))

; The text of a `content:`, in pieces between its escapes.
(declaration
  (property_name) @_property
  (string_value (string_content) @string)
  (#eq? @_property "content"))
