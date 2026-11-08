; Where prose lives in an HCL file.

(comment) @comment

; A block's label is a name its author chose where the block says what it declares: `variable
; "region"`, `output "vpc_id"`, `module "vpc"`, and a resource's name, the second label of `resource
; "aws_s3_bucket" "logs"`. The first label of a resource or data source is the provider's, and so is
; every other block's: `provider "aws"`, `backend "s3"`, `provisioner "local-exec"`. Those are one
; token, so the string rule below makes them values.
(block
  (identifier) @_type
  .
  (string_lit (template_literal) @declaration)
  .
  (block_start)
  (#any-of? @_type "variable" "output" "module"))
(block
  (identifier) @_type
  .
  (string_lit)
  .
  (string_lit (template_literal) @declaration)
  .
  (block_start)
  (#any-of? @_type "resource" "data"))

; A `locals` block declares its keys.
(block
  (identifier) @_type
  .
  (block_start)
  (body (attribute (identifier) @declaration))
  (#eq? @_type "locals"))

; A heredoc is a script or a policy document (`user_data`, `policy`): code, held to known typos only.
; One that is a `description` is text.
(attribute
  (identifier) @_key
  (expression (template_expr (heredoc_template (template_literal) @value)))
  (#not-eq? @_key "description"))

; The text of a string, between its interpolations. A literal of one token is a name, a CIDR block
; or an ARN, and the engine holds it to known typos.
(template_literal) @string
