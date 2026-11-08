use expect_test::expect;

use crate::tests::check;

#[test]
fn less_prose_is_comments_content_and_declared_names() {
    check(
        "/src/styles.less",
        r#"// A line comment.
/* A block comment. */
@import "mixins.less";
@brand-color: #336;
.bordered(@width: 2px) {
  border: @width solid black;
}
#namespace { color: red; }
@keyframes fadein { from { opacity: 0; } }
.card:hover > #main-nav::before, a[type="checkbox"], div {
  .bordered(4px);
  &:extend(.alert-info all);
  color: darken(@brand-color, 10%);
  -webkit-font-smoothing: antialiased;
  font-family: "Segoe UI", system-ui;
  background: url("img/hero.png");
  content: "Hello\a world \201C";
}
"#,
        expect![[r#"
            less
            > // A line comment.
              ^^^^^^^^^^^^^^^^^^ comment
            > /* A block comment. */
              ^^^^^^^^^^^^^^^^^^^^^^ comment
            > @import "mixins.less";
            > @brand-color: #336;
               ^^^^^^^^^^^ declaration
            > .bordered(@width: 2px) {
               ^^^^^^^^ declaration
                         ^^^^^ declaration
            >   border: @width solid black;
                ^^^^^^ value
            > }
            > #namespace { color: red; }
               ^^^^^^^^^ declaration
                           ^^^^^ value
            > @keyframes fadein { from { opacity: 0; } }
                         ^^^^^^ declaration
                                         ^^^^^^^ value
            > .card:hover > #main-nav::before, a[type="checkbox"], div {
                    ^^^^^ declaration
                             ^^^^^^^^ declaration
            >   .bordered(4px);
            >   &:extend(.alert-info all);
            >   color: darken(@brand-color, 10%);
                ^^^^^ value
            >   -webkit-font-smoothing: antialiased;
            >   font-family: "Segoe UI", system-ui;
                ^^^^^^^^^^^ value
            >   background: url("img/hero.png");
                ^^^^^^^^^^ value
            >   content: "Hello\a world \201C";
                ^^^^^^^ value
                         ^^^^^^^^^^^^^^^^^^^^^ string
            > }
        "#]],
    );
}

#[test]
fn interpolation_is_a_use_and_is_not_read() {
    check(
        "/src/interpolation.less",
        r#"@selector: ~".my";
@{selector}-class { color: red; }
.card-@{name}:hover { color: red; }
.@{prefix}-btn { color: red; }
.a { @{side}-top: 1px; }
.b { width: ~"calc(100% - @{gap})"; }
.c { content: "a @{unlikelyname} c"; }
.d { content: "plain text"; }
"#,
        expect![[r#"
            less
            > @selector: ~".my";
               ^^^^^^^^ declaration
            > @{selector}-class { color: red; }
                                  ^^^^^ value
            > .card-@{name}:hover { color: red; }
               ^^^^^ declaration
                                    ^^^^^ value
            > .@{prefix}-btn { color: red; }
                         ^^^ declaration
                               ^^^^^ value
            > .a { @{side}-top: 1px; }
               ^ declaration
            > .b { width: ~"calc(100% - @{gap})"; }
               ^ declaration
                   ^^^^^ value
            > .c { content: "a @{unlikelyname} c"; }
               ^ declaration
                   ^^^^^^^ value
            > .d { content: "plain text"; }
               ^ declaration
                   ^^^^^^^ value
                            ^^^^^^^^^^^^ string
        "#]],
    );
}
