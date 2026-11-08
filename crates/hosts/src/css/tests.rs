use expect_test::expect;

use crate::tests::check;

#[test]
fn css_prose_is_comments_content_and_declared_names() {
    check(
        "/src/styles.css",
        r#"/* The brand colors. */
@charset "utf-8";
@import url("normalize.css") screen;
:root {
  --brand-color: #336;
  --font-stack: "Helvetica Neue", sans-serif;
  -webkit-font-smoothing: antialiased;
}
@keyframes fadein {
  from { opacity: 0; }
  to { opacity: 1; }
}
.card:hover > #main-nav::before, a[type="checkbox"], div {
  color: var(--brand-color);
  font-family: "Segoe UI", system-ui;
  background: url("img/hero.png");
  content: "Hello\a world \201C";
  grid-template-areas: "header header" "sidebar main";
}
"#,
        expect![[r#"
            css
            > /* The brand colors. */
              ^^^^^^^^^^^^^^^^^^^^^^^ comment
            > @charset "utf-8";
            > @import url("normalize.css") screen;
            > :root {
            >   --brand-color: #336;
                  ^^^^^^^^^^^ declaration
            >   --font-stack: "Helvetica Neue", sans-serif;
                  ^^^^^^^^^^ declaration
            >   -webkit-font-smoothing: antialiased;
            > }
            > @keyframes fadein {
                         ^^^^^^ declaration
            >   from { opacity: 0; }
                       ^^^^^^^ value
            >   to { opacity: 1; }
                     ^^^^^^^ value
            > }
            > .card:hover > #main-nav::before, a[type="checkbox"], div {
               ^^^^ declaration
                             ^^^^^^^^ declaration
            >   color: var(--brand-color);
                ^^^^^ value
            >   font-family: "Segoe UI", system-ui;
                ^^^^^^^^^^^ value
            >   background: url("img/hero.png");
                ^^^^^^^^^^ value
            >   content: "Hello\a world \201C";
                ^^^^^^^ value
                          ^^^^^ string
                                  ^^^^^ string
            >   grid-template-areas: "header header" "sidebar main";
                ^^^^^^^^^^^^^^^^^^^ value
            > }
        "#]],
    );
}

#[test]
fn escaped_class_names_keep_their_pieces() {
    check(
        "/dist/tailwind.css",
        r#".sm\:flex { display: flex; }
.w-1\/2 { width: 50%; }
.hover\:bg-blue-500:hover { color: blue; }
.btn { content: "\f101"; }
"#,
        expect![[r#"
            css
            > .sm\:flex { display: flex; }
               ^^ declaration
                   ^^^^ declaration
                          ^^^^^^^ value
            > .w-1\/2 { width: 50%; }
               ^^^ declaration
                    ^ declaration
                        ^^^^^ value
            > .hover\:bg-blue-500:hover { color: blue; }
               ^^^^^ declaration
                      ^^^^^^^^^^^ declaration
                                          ^^^^^ value
            > .btn { content: "\f101"; }
               ^^^ declaration
                     ^^^^^^^ value
        "#]],
    );
}
