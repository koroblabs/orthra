; Where prose lives in HTML. Captures: @prose, @comment, @string, @skip (`tree_sitter_host.rs`).
; What is not captured is not checked: `class`, `id`, `href`, `src`, `style`, `data-*` and `type`
; are values, so they have no pattern, and nor has a tag or attribute name.
;
; A `<script>` or `<style>` body is a `raw_text`, a different node from `text`, so it is not
; checked. It is JavaScript or CSS, and injecting those hosts into it is later work.

; The text of the page is prose, not a string: a sentence, not a value. An entity (`&amp;`,
; `&nbsp;`, `&#8212;`) is a node of its own between two texts, so it never glues onto a word.
(text) @prose
(comment) @comment

; Code and keyboard input in the middle of a sentence is not the page's words.
((element (start_tag (tag_name) @_tag)) @skip
  (#match? @_tag "^(code|kbd|samp|var|tt|pre)$"))

; The attributes whose value is text for a reader, or for a screen reader.
((attribute
   (attribute_name) @_name
   (quoted_attribute_value (attribute_value) @string))
  (#match? @_name "^(alt|title|placeholder|aria-label|aria-description|label)$"))

; What a page says about itself: `<meta name="description" content="...">`, in either order of
; its attributes, with or without the closing slash.
((_
   (tag_name) @_tag
   (attribute (attribute_name) @_key (quoted_attribute_value (attribute_value) @_kind))
   (attribute (attribute_name) @_content (quoted_attribute_value (attribute_value) @string)))
  (#eq? @_tag "meta")
  (#match? @_key "^(name|property)$")
  (#match? @_kind "^(description|og:title|og:description|twitter:title|twitter:description)$")
  (#eq? @_content "content"))
((_
   (tag_name) @_tag
   (attribute (attribute_name) @_content (quoted_attribute_value (attribute_value) @string))
   (attribute (attribute_name) @_key (quoted_attribute_value (attribute_value) @_kind)))
  (#eq? @_tag "meta")
  (#match? @_key "^(name|property)$")
  (#match? @_kind "^(description|og:title|og:description|twitter:title|twitter:description)$")
  (#eq? @_content "content"))
