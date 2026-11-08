use expect_test::expect;

use crate::tests::check;

#[test]
fn markup_is_prose_comments_and_text_attributes() {
    check(
        "/c/Card.svelte",
        r#"<script lang="ts">
  let count = 0;
  const msg = "hello wrld";
</script>

<!-- A comment -->
<h1 title="Hello wrld" class:active={okk} on:click={go} bind:value={vl}>Welcome home</h1>
<img src="a.png" alt="A red fox" />
<input placeholder="Your name" aria-label="Name field" />
<p>Use <code>fooBar()</code> and <kbd>Ctrl</kbd> here &amp; there.</p>

<style>
  h1 { color: red; }
</style>
"#,
        expect![[r#"
            svelte
            > <script lang="ts">
            >   let count = 0;
            >   const msg = "hello wrld";
            > </script>
            >
            > <!-- A comment -->
              ^^^^^^^^^^^^^^^^^^ comment
            > <h1 title="Hello wrld" class:active={okk} on:click={go} bind:value={vl}>Welcome home</h1>
                         ^^^^^^^^^^ string
                                                                                      ^^^^^^^^^^^^ prose
            > <img src="a.png" alt="A red fox" />
                                    ^^^^^^^^^ string
            > <input placeholder="Your name" aria-label="Name field" />
                                  ^^^^^^^^^ string
                                                         ^^^^^^^^^^ string
            > <p>Use <code>fooBar()</code> and <kbd>Ctrl</kbd> here &amp; there.</p>
                 ^^^ prose
                                           ^^^ prose
                                                               ^^^^ prose
                                                                          ^^^^^^ prose
            >
            > <style>
            >   h1 { color: red; }
            > </style>
        "#]],
    );
}

#[test]
fn a_blocks_and_expressions_code_is_cut_out_of_the_text() {
    check(
        "/c/Blocks.svelte",
        r#"<h1>Hello, {name}, welcome{suffix} back.</h1>
{#if count > 1}
  <p>Count is {count} times</p>
{:else if wrld}
  <p>other text</p>
{:else}
  <p>Nothing here</p>
{/if}
{#each itms as itm (itm.id)}
  <li>{itm.name} is here</li>
{/each}
{#await prom then valu}<p>Done {valu}</p>{/await}
<p>{@html rw} then {@render foo()}</p>
<img alt="Photo of {usr}" title="Hello {wrld} there" />
"#,
        expect![[r#"
            svelte
            > <h1>Hello, {name}, welcome{suffix} back.</h1>
                  ^^^^^^ prose
                               ^^^^^^^^^ prose
                                                 ^^^^^ prose
            > {#if count > 1}
            >   <p>Count is {count} times</p>
                   ^^^^^^^^ prose
                                    ^^^^^ prose
            > {:else if wrld}
            >   <p>other text</p>
                   ^^^^^^^^^^ prose
            > {:else}
            >   <p>Nothing here</p>
                   ^^^^^^^^^^^^ prose
            > {/if}
            > {#each itms as itm (itm.id)}
            >   <li>{itm.name} is here</li>
                               ^^^^^^^ prose
            > {/each}
            > {#await prom then valu}<p>Done {valu}</p>{/await}
                                        ^^^^ prose
            > <p>{@html rw} then {@render foo()}</p>
                            ^^^^ prose
            > <img alt="Photo of {usr}" title="Hello {wrld} there" />
                        ^^^^^^^^ string
                                               ^^^^^ string
                                                            ^^^^^ string
        "#]],
    );
}
