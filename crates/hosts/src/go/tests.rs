use expect_test::expect;

use crate::tests::check;

#[test]
fn go_prose_is_comments_strings_and_declared_names() {
    check(
        "/cmd/greet/main.go",
        r#"// Package main greets whoever asks.
package main

import (
	"fmt"
	str "strings"
)

// Greeter says hello.
type Greeter struct {
	Name  string `json:"user_name,omitempty"`
	count int
}

/* Greet builds the greeting. */
func (g *Greeter) Greet(person string, extra ...string) string {
	message := "Hello\nworld"
	raw := `raw text`
	for index, item := range extra {
		fmt.Println(index, item)
	}
	return str.ToUpper(message + raw)
}
"#,
        expect![[r#"
            go
            > // Package main greets whoever asks.
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > package main
                      ^^^^ declaration
            >
            > import (
            > 	"fmt"
            > 	str "strings"
              	^^^ declaration
            > )
            >
            > // Greeter says hello.
                 ^^^^^^^^^^^^^^^^^^^ doccomment
            > type Greeter struct {
                   ^^^^^^^ declaration
            > 	Name  string `json:"user_name,omitempty"`
              	^^^^ declaration
            > 	count int
              	^^^^^ declaration
            > }
            >
            > /* Greet builds the greeting. */
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > func (g *Greeter) Greet(person string, extra ...string) string {
                    ^ declaration
                                ^^^^^ declaration
                                      ^^^^^^ declaration
                                                     ^^^^^ declaration
            > 	message := "Hello\nworld"
              	^^^^^^^ declaration
              	            ^^^^^ string
              	                   ^^^^^ string
            > 	raw := `raw text`
              	^^^ declaration
              	        ^^^^^^^^ string
            > 	for index, item := range extra {
              	    ^^^^^ declaration
              	           ^^^^ declaration
            > 		fmt.Println(index, item)
            > 	}
            > 	return str.ToUpper(message + raw)
            > }
        "#]],
    );
}

/// A directive is not prose, a doc comment's code block indented by a tab is code, and a comment
/// in a body stays a comment.
#[test]
fn go_directives_and_doc_code_are_not_prose() {
    crate::tests::check(
        "/src/p.go",
        "//go:build linux || qzxvtag\n\n// Package p does things. For example:\n//\n//\tqzxvcode := run(qzxvarg)\n//\n// Then qzxvprose follows.\npackage p\n\n//go:generate go run qzxvgen.go\nfunc f() {\n\t// A comment in a body: qzxvbody.\n}\n",
        expect_test::expect![[r#"
            go
            > //go:build linux || qzxvtag
            >
            > // Package p does things. For example:
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > //
            > //	qzxvcode := run(qzxvarg)
            > //
            > // Then qzxvprose follows.
                 ^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > package p
                      ^ declaration
            >
            > //go:generate go run qzxvgen.go
            > func f() {
                   ^ declaration
            > 	// A comment in a body: qzxvbody.
              	^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > }
        "#]],
    );
}
