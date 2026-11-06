; Where prose lives in a YAML file.

(comment) @comment

; A key is a declaration: `runs-on`, `dependancies`.
(block_mapping_pair
  key: (flow_node [(plain_scalar) (single_quote_scalar) (double_quote_scalar)] @declaration))
(flow_pair
  key: (flow_node [(plain_scalar) (single_quote_scalar) (double_quote_scalar)] @declaration))

; What a CI step or a container runs is code, held to known typos only: GitHub's `run:`, GitLab's
; `script:`, Compose's and Kubernetes' `command:`.
(block_mapping_pair
  key: (flow_node (plain_scalar (string_scalar) @_key))
  value: (_) @value
  (#any-of? @_key "run" "script" "before_script" "after_script" "command" "entrypoint" "args" "cmd"))

; A string wherever a value goes: unquoted, quoted, or a `|` or `>` block. A plain scalar that is a
; number, a boolean or null is not one.
(block_mapping_pair
  value: (flow_node [(plain_scalar (string_scalar)) (single_quote_scalar) (double_quote_scalar)] @string))
(block_mapping_pair value: (block_node (block_scalar) @string))
(flow_pair
  value: (flow_node [(plain_scalar (string_scalar)) (single_quote_scalar) (double_quote_scalar)] @string))
(block_sequence_item
  (flow_node [(plain_scalar (string_scalar)) (single_quote_scalar) (double_quote_scalar)] @string))
(block_sequence_item (block_node (block_scalar) @string))
(flow_sequence
  (flow_node [(plain_scalar (string_scalar)) (single_quote_scalar) (double_quote_scalar)] @string))
(document
  (flow_node [(plain_scalar (string_scalar)) (single_quote_scalar) (double_quote_scalar)] @string))
(document (block_node (block_scalar) @string))
