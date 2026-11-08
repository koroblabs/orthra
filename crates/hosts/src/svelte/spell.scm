(text) @prose
(comment) @comment
((element (start_tag (tag_name) @_tag)) @skip
  (#match? @_tag "^(code|kbd|samp|var|tt|pre)$"))
((attribute
   (attribute_name) @_name
   (quoted_attribute_value (attribute_value) @string))
  (#match? @_name "^(alt|title|placeholder|aria-label|aria-description|label)$"))
