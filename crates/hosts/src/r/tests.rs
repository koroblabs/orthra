use expect_test::expect;

use crate::tests::check;

#[test]
fn r_prose_is_comments_roxygen_strings_and_declared_names() {
    check(
        "/R/greet.R",
        r#"library("dplyr")
# A line comment.
#' Builds the greeting.
#' @param person The person to greet.
#' @param greeting What to say.
#' @importFrom stats median
#' @return The message.
greet <- function(person, greeting = "Hello\nworld, friend", ...) {
  local_var <- paste0(greeting, person)
  df$col <- "kept"
  total = 5
  "arrow" -> reversed
  invisible(local_var)
}
mean <- \(vals) sum(vals)
"#,
        expect![[r#"
            r
            > library("dplyr")
            > # A line comment.
              ^^^^^^^^^^^^^^^^^ comment
            > #' Builds the greeting.
                 ^^^^^^^^^^^^^^^^^^^^ doccomment
            > #' @param person The person to greet.
                               ^^^^^^^^^^^^^^^^^^^^ doccomment
            > #' @param greeting What to say.
                                 ^^^^^^^^^^^^ doccomment
            > #' @importFrom stats median
            > #' @return The message.
                         ^^^^^^^^^^^^ doccomment
            > greet <- function(person, greeting = "Hello\nworld, friend", ...) {
              ^^^^^ declaration
                                ^^^^^^ declaration
                                        ^^^^^^^^ declaration
                                                    ^^^^^^^^^^^^^^^^^^^^ string
            >   local_var <- paste0(greeting, person)
                ^^^^^^^^^ declaration
            >   df$col <- "kept"
                           ^^^^ value
            >   total = 5
                ^^^^^ declaration
            >   "arrow" -> reversed
                 ^^^^^ value
                           ^^^^^^^^ declaration
            >   invisible(local_var)
            > }
            > mean <- \(vals) sum(vals)
              ^^^^ declaration
                        ^^^^ declaration
        "#]],
    );
}
