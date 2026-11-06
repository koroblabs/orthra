use expect_test::expect;

use crate::tests::check;

#[test]
fn python_prose_is_comments_docstrings_strings_and_declared_names() {
    check(
        "/src/greet.py",
        r#""""Greeting helpers."""
import json
from os import path as ospath


class Greeter:
    """Says hello to whoever asks."""

    count = 0

    def greet(self, person: str, *args, times=1, **kwargs) -> str:
        """Builds the greeting."""
        # Escapes stay inside the content here.
        message = f"Hello\nworld, {person}!"
        for item in args:
            shout(item)
        return json.dumps(message)
"#,
        expect![[r#"
            python
            > """Greeting helpers."""
                 ^^^^^^^^^^^^^^^^^ doccomment
            > import json
            > from os import path as ospath
                                     ^^^^^^ declaration
            >
            >
            > class Greeter:
                    ^^^^^^^ declaration
            >     """Says hello to whoever asks."""
                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >
            >     count = 0
                  ^^^^^ declaration
            >
            >     def greet(self, person: str, *args, times=1, **kwargs) -> str:
                      ^^^^^ declaration
                            ^^^^ declaration
                                  ^^^^^^ declaration
                                                ^^^^ declaration
                                                      ^^^^^ declaration
                                                                 ^^^^^^ declaration
            >         """Builds the greeting."""
                         ^^^^^^^^^^^^^^^^^^^^ doccomment
            >         # Escapes stay inside the content here.
                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            >         message = f"Hello\nworld, {person}!"
                      ^^^^^^^ declaration
                                  ^^^^^^^^^^^^^ string
                                                        ^ string
            >         for item in args:
                          ^^^^ declaration
            >             shout(item)
            >         return json.dumps(message)
        "#]],
    );
}
