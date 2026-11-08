use expect_test::expect;

use crate::tests::check;

#[test]
fn hcl_prose_is_comments_text_and_the_names_a_configuration_declares() {
    check(
        "/main.tf",
        r##"# Buckets for the logs.
terraform {
  required_version = ">= 1.5"
}

provider "aws" {
  region = "us-east-1"
}

variable "region" {
  description = "The region of the bucket"
  default     = "eu-west-1"
}

locals {
  bucket_name = "my-${var.region}-logs"
  tags        = { Name = "shared logs" }
}

resource "aws_s3_bucket" "logs" {
  bucket = local.bucket_name // the name
  tags   = { Name = "logs\nbucket" }
  /* a block comment */
  user_data = <<-EOT
    #!/bin/bash
    echo ${var.region}
  EOT
  note = <<EOT
    A heredoc that is prose.
  EOT
}

output "bucket_arn" {
  description = <<-EOT
    The ARN of the bucket.
  EOT
  value = aws_s3_bucket.logs.arn
}
"##,
        expect![[r#"
            hcl
            > # Buckets for the logs.
              ^^^^^^^^^^^^^^^^^^^^^^^ comment
            > terraform {
            >   required_version = ">= 1.5"
                                    ^^^^^^ string
            > }
            >
            > provider "aws" {
                        ^^^ value
            >   region = "us-east-1"
                          ^^^^^^^^^ value
            > }
            >
            > variable "region" {
                        ^^^^^^ declaration
            >   description = "The region of the bucket"
                               ^^^^^^^^^^^^^^^^^^^^^^^^ string
            >   default     = "eu-west-1"
                               ^^^^^^^^^ value
            > }
            >
            > locals {
            >   bucket_name = "my-${var.region}-logs"
                ^^^^^^^^^^^ declaration
                               ^^^ value
                                               ^^^^^ value
            >   tags        = { Name = "shared logs" }
                ^^^^ declaration
                                        ^^^^^^^^^^^ string
            > }
            >
            > resource "aws_s3_bucket" "logs" {
                        ^^^^^^^^^^^^^ value
                                        ^^^^ declaration
            >   bucket = local.bucket_name // the name
                                           ^^^^^^^^^^^ comment
            >   tags   = { Name = "logs\nbucket" }
                                   ^^^^^^^^^^^^ string
            >   /* a block comment */
                ^^^^^^^^^^^^^^^^^^^^^ comment
            >   user_data = <<-EOT
            >     #!/bin/bash
                  ^^^^^^^^^^^ value (+1 lines)
            >     echo ${var.region}
            >   EOT
            >   note = <<EOT
            >     A heredoc that is prose.
                  ^^^^^^^^^^^^^^^^^^^^^^^^ value
            >   EOT
            > }
            >
            > output "bucket_arn" {
                      ^^^^^^^^^^ declaration
            >   description = <<-EOT
            >     The ARN of the bucket.
                  ^^^^^^^^^^^^^^^^^^^^^^ string
            >   EOT
            >   value = aws_s3_bucket.logs.arn
            > }
        "#]],
    );
}
