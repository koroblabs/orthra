; Where prose lives in a Dockerfile. Captures: @comment, @string, @declaration, @value, @skip
; (`tree_sitter_host.rs`). `RUN`, `CMD`, `ENTRYPOINT` and `HEALTHCHECK` are shell: not captured.

; Comments, but not the parser directives, which are settings: `# syntax=docker/dockerfile:1`.
((comment) @comment
  (#not-match? @comment "^#\\s*(syntax|escape|check)\\s*="))

; The prose of an image's metadata: `LABEL description="..."` and the OCI annotations.
(label_pair
  key: (_) @_key
  value: [(double_quoted_string) (single_quoted_string)] @string
  (#match? @_key "^[\"']?(description|title|summary|org\\.opencontainers\\.image\\.(description|title)|org\\.label-schema\\.description)[\"']?$"))

; What `ENV` and `ARG` hold is a path, a version or a flag: known typos only.
(env_pair value: (_) @value)
(arg_instruction default: (_) @value)

; Names declared: `ARG name`, `ENV name=value` and the stage `FROM x AS name`.
(arg_instruction name: (unquoted_string) @declaration)
(env_pair name: (unquoted_string) @declaration)
(from_instruction as: (image_alias) @declaration)

; What the grammar could not parse is not read as words.
(ERROR) @skip
