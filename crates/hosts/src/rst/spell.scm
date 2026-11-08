; Where prose lives in reStructuredText. Captures: @prose, @comment, @skip (`tree_sitter_host.rs`).
; Not captured, so not checked: a literal block (after `::`), a doctest, the arguments, options
; and content of every directive but the few below, a role's target, a hyperlink target, a
; substitution definition, a field's name and a table.

(title) @prose
(comment) @comment

; A paragraph. Its `interpreted` text is code between backticks to the segmenter, and its
; ``literal`` and |substitution| are cut out by the host's delimiters (`mod.rs`): a capture
; cannot have a hole in it.
(paragraph) @prose

; A paragraph that holds a literal block is the sentence before `::` and the code after it, one
; node. The code is not words, and the sentence is given up with it.
((paragraph (literal_block)) @skip)

; A table is a paragraph to this grammar: simple (`=====  =====`) or grid (`+-----+-----+`).
((paragraph) @skip
  (#match? @skip "^(=+ |\\+-|-+ +-)"))

; A directive or a comment the grammar did not recognize, nested in a definition or a field
; (`.. versionchanged:: 5.3.0` indented under a term), comes out as a paragraph.
((paragraph) @skip
  (#match? @skip "^\\.\\. "))

; An option of a directive is a name, a path or a value. Three hold text for a reader.
((options (field (field_name) @_name) @skip)
  (#not-match? @_name "^(alt|caption|title)$"))

; A directive whose content is prose: the admonitions, `seealso`, `versionadded` and the like,
; and a figure's caption. The content is raw text to this grammar, and the argument of an
; admonition is its title; the argument of `versionadded` is a version, and of `figure` a path.
((directive (type) @_type (body (content) @prose))
  (#match? @_type "^(note|warning|tip|important|caution|danger|error|hint|attention|admonition|seealso|versionadded|versionchanged|versionremoved|deprecated|sidebar|topic|figure|epigraph|rubric)$"))
; That content is not parsed, so a literal block (`::`), a doctest or a directive nested in it
; would be read as words: when it holds one, all of it is given up, as a paragraph's is.
((directive (body (content) @skip))
  (#match? @skip "(::[ \\t]*(\\n|$))|(^|\\n)[ \\t]*(\\.\\. |>>> )"))
((directive (type) @_type (body (arguments) @prose))
  (#match? @_type "^(note|warning|tip|important|caution|danger|error|hint|attention|admonition|sidebar|topic|rubric)$"))
