use expect_test::expect;

use crate::tests::check;

#[test]
fn scss_prose_is_comments_content_and_declared_names() {
    check(
        "/src/styles.scss",
        r#"// A line comment.
/* A block comment. */
@use "sass:math";
$brand-color: #336;
@mixin button-variant($background, $border: null) {
  color: $background;
  @include media-breakpoint-up(md) { display: none; }
}
@function rem($px) { @return math.div($px, 16) * 1rem; }
%placeholder-sel { margin: 0; }
@keyframes fadein { from { opacity: 0; } }
.card:hover > #main-nav::before, a[type="checkbox"], div {
  @extend .alert-info;
  @extend %placeholder-sel;
  @include button-variant($brand-color);
  --font-stack: "Helvetica Neue", sans-serif;
  -webkit-font-smoothing: antialiased;
  font-family: "Segoe UI", system-ui;
  background: url("img/hero.png");
  content: "Hello\a world \201C";
}
"#,
        expect![[r#"
            scss
            > // A line comment.
              ^^^^^^^^^^^^^^^^^^ comment
            > /* A block comment. */
              ^^^^^^^^^^^^^^^^^^^^^^ comment
            > @use "sass:math";
            > $brand-color: #336;
               ^^^^^^^^^^^ declaration
            > @mixin button-variant($background, $border: null) {
                     ^^^^^^^^^^^^^^ declaration
                                     ^^^^^^^^^^ declaration
                                                  ^^^^^^ declaration
            >   color: $background;
                ^^^^^ value
            >   @include media-breakpoint-up(md) { display: none; }
                                                   ^^^^^^^ value
            > }
            > @function rem($px) { @return math.div($px, 16) * 1rem; }
                        ^^^ declaration
                             ^^ declaration
            > %placeholder-sel { margin: 0; }
               ^^^^^^^^^^^^^^^ declaration
                                 ^^^^^^ value
            > @keyframes fadein { from { opacity: 0; } }
                         ^^^^^^ declaration
                                         ^^^^^^^ value
            > .card:hover > #main-nav::before, a[type="checkbox"], div {
               ^^^^ declaration
                             ^^^^^^^^ declaration
            >   @extend .alert-info;
            >   @extend %placeholder-sel;
            >   @include button-variant($brand-color);
            >   --font-stack: "Helvetica Neue", sans-serif;
                  ^^^^^^^^^^ declaration
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
        "/src/interpolation.scss",
        r#".card-#{$name}:hover { color: red; }
.#{$prefix}-btn, #nav-#{$id} { color: red; }
.a { #{$side}-top: 1px; }
.b { width: calc(100% - #{$gap}); }
.c { content: "a #{$unlikelyname} c"; }
.d { content: "plain text"; }
"#,
        expect![[r#"
            scss
            > .card-#{$name}:hover { color: red; }
               ^^^^^ declaration
                                     ^^^^^ value
            > .#{$prefix}-btn, #nav-#{$id} { color: red; }
                          ^^^ declaration
                                ^^^^ declaration
                                             ^^^^^ value
            > .a { #{$side}-top: 1px; }
               ^ declaration
            > .b { width: calc(100% - #{$gap}); }
               ^ declaration
            > .c { content: "a #{$unlikelyname} c"; }
               ^ declaration
            > .d { content: "plain text"; }
               ^ declaration
                   ^^^^^^^ value
                            ^^^^^^^^^^^^ string
        "#]],
    );
}
