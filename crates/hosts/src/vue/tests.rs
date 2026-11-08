use expect_test::expect;

use crate::tests::check;

#[test]
fn a_template_is_prose_comments_and_text_attributes() {
    check(
        "/c/Card.vue",
        r#"<template>
  <div class="a" :title="msgg" @click="go(wrld)" v-if="okk" v-for="itm in itms">
    <!-- A comment -->
    <h1 title="Hello wrld" id="idd">Welcome home</h1>
    <img src="a.png" alt="A red fox" :alt="fxx">
    <p>Use <code>fooBar()</code> and <kbd>Ctrl</kbd> here.</p>
    <my-comp v-bind:foo="bar" #default="{ itm }" placeholder="Your name">slot text</my-comp>
  </div>
</template>

<script setup lang="ts">
const msg = "hello wrld";
</script>

<style scoped>
.a { color: red; }
</style>
"#,
        expect![[r#"
            vue
            > <template>
            >   <div class="a" :title="msgg" @click="go(wrld)" v-if="okk" v-for="itm in itms">
            >     <!-- A comment -->
                  ^^^^^^^^^^^^^^^^^^ comment
            >     <h1 title="Hello wrld" id="idd">Welcome home</h1>
                             ^^^^^^^^^^ string
                                                  ^^^^^^^^^^^^ prose
            >     <img src="a.png" alt="A red fox" :alt="fxx">
                                        ^^^^^^^^^ string
            >     <p>Use <code>fooBar()</code> and <kbd>Ctrl</kbd> here.</p>
                     ^^^ prose
                                               ^^^ prose
                                                                   ^^^^^ prose
            >     <my-comp v-bind:foo="bar" #default="{ itm }" placeholder="Your name">slot text</my-comp>
                                                                            ^^^^^^^^^ string
                                                                                       ^^^^^^^^^ prose
            >   </div>
            > </template>
            >
            > <script setup lang="ts">
            > const msg = "hello wrld";
            > </script>
            >
            > <style scoped>
            > .a { color: red; }
            > </style>
        "#]],
    );
}

#[test]
fn an_interpolations_code_is_cut_out_of_the_text() {
    check(
        "/c/Greet.vue",
        "<template>\n<p>Hello {{ usr.name }}, welcome{{ x }} back.</p>\n<p>{{ count }} items</p>\n<p>An unclosed {{ tag</p>\n</template>\n",
        expect![[r#"
            vue
            > <template>
            > <p>Hello {{ usr.name }}, welcome{{ x }} back.</p>
                 ^^^^^ prose
                                     ^^^^^^^^^ prose
                                                      ^^^^^ prose
            > <p>{{ count }} items</p>
                             ^^^^^ prose
            > <p>An unclosed {{ tag</p>
                 ^^^^^^^^^^^ prose
            > </template>
        "#]],
    );
}

#[test]
fn an_entity_is_left_to_the_segmenter_in_the_text() {
    check(
        "/c/Entities.vue",
        "<template>\n<p>Fish&amp;chips&nbsp;and&#8212;more &lt;b&gt; text</p>\n</template>\n",
        expect![[r#"
            vue
            > <template>
            > <p>Fish&amp;chips&nbsp;and&#8212;more &lt;b&gt; text</p>
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ prose
            > </template>
        "#]],
    );
}
