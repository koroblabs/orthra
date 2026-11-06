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
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
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
              ^^^^^^^^^^^^^^^^^^^^^^ comment
            > type Greeter struct {
                   ^^^^^^^ declaration
            > 	Name  string `json:"user_name,omitempty"`
              	^^^^ declaration
            > 	count int
              	^^^^^ declaration
            > }
            >
            > /* Greet builds the greeting. */
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
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
