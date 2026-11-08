use expect_test::expect;

use crate::tests::check;

#[test]
fn html_prose_is_text_comments_and_text_attributes() {
    check(
        "/site/index.html",
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width">
  <meta name="description" content="A fast spell checker">
  <meta content="Another page summary" property="og:description" />
  <title>Home page</title>
  <style>.hero { color: red; }</style>
  <script>var greeting = "hello wrld";</script>
</head>
<body class="main-nav" id="top">
  <!-- The main heading -->
  <h1 title="Welcome here">Hello, world</h1>
  <img src="img/hero.png" alt="A red fox" data-info="somthing">
  <input type="text" placeholder="Your name" aria-label="Name field">
  <a href="/about-us" style="color: blue">About us</a>
  <p>Use <code>fooBar()</code> and <kbd>Ctrl</kbd> here.</p>
</body>
</html>
"#,
        expect![[r#"
            html
            > <!DOCTYPE html>
            > <html lang="en">
            > <head>
            >   <meta charset="utf-8">
            >   <meta name="viewport" content="width=device-width">
            >   <meta name="description" content="A fast spell checker">
                                                  ^^^^^^^^^^^^^^^^^^^^ string
            >   <meta content="Another page summary" property="og:description" />
                               ^^^^^^^^^^^^^^^^^^^^ string
            >   <title>Home page</title>
                       ^^^^^^^^^ prose
            >   <style>.hero { color: red; }</style>
            >   <script>var greeting = "hello wrld";</script>
            > </head>
            > <body class="main-nav" id="top">
            >   <!-- The main heading -->
                ^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            >   <h1 title="Welcome here">Hello, world</h1>
                           ^^^^^^^^^^^^ string
                                         ^^^^^^^^^^^^ prose
            >   <img src="img/hero.png" alt="A red fox" data-info="somthing">
                                             ^^^^^^^^^ string
            >   <input type="text" placeholder="Your name" aria-label="Name field">
                                                ^^^^^^^^^ string
                                                                       ^^^^^^^^^^ string
            >   <a href="/about-us" style="color: blue">About us</a>
                                                        ^^^^^^^^ prose
            >   <p>Use <code>fooBar()</code> and <kbd>Ctrl</kbd> here.</p>
                   ^^^ prose
                                             ^^^ prose
                                                                 ^^^^^ prose
            > </body>
            > </html>
        "#]],
    );
}

#[test]
fn an_entity_is_cut_out_of_the_text() {
    check(
        "/site/entities.html",
        "<p>Fish&amp;chips&nbsp;and&#8212;more &lt;b&gt; text</p>\n",
        expect![[r#"
            html
            > <p>Fish&amp;chips&nbsp;and&#8212;more &lt;b&gt; text</p>
                 ^^^^ prose
                          ^^^^^ prose
                                     ^^^ prose
                                               ^^^^ prose
                                                        ^ prose
                                                              ^^^^ prose
        "#]],
    );
}

#[test]
fn a_templates_code_is_cut_out_of_the_page() {
    check(
        "/templates/greet.html",
        r#"{% extends "base.html" %}
{% block content %}
<p title="{{ page.title }}">Hello {{ user.name }}, welcome{# a note #} back.</p>
<p>{% if items %}{{ items|length }} items{% endif %}</p>
<p>An unclosed {{ tag</p>
{% endblock %}
"#,
        expect![[r#"
            html
            > {% extends "base.html" %}
            > {% block content %}
            > <p title="{{ page.title }}">Hello {{ user.name }}, welcome{# a note #} back.</p>
                                          ^^^^^ prose
                                                               ^^^^^^^^^ prose
                                                                                     ^^^^^ prose
            > <p>{% if items %}{{ items|length }} items{% endif %}</p>
                                                  ^^^^^ prose
            > <p>An unclosed {{ tag</p>
                 ^^^^^^^^^^^ prose
            > {% endblock %}
        "#]],
    );
}
