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

/// A docstring's doctest, `>>>` and its output up to a blank line, is code, and so is a block
/// indented four spaces past the docstring's own indentation, as `cleandoc` reads it.
#[test]
fn doctests_and_indented_blocks_in_a_docstring_are_code() {
    check(
        "/src/greet.py",
        r#"def greet(name):
    """Greets a person.

    >>> greet("qzxv")
    'Hello, qzxv'

    An example::

        greet(wrtyp)

    Returns the greeting.
    """
    return name
"#,
        expect![[r#"
            python
            > def greet(name):
                  ^^^^^ declaration
                        ^^^^ declaration
            >     """Greets a person.
                     ^^^^^^^^^^^^^^^^ doccomment
            >
            >     >>> greet("qzxv")
            >     'Hello, qzxv'
            >
            >     An example::
                  ^^^^^^^^^^^^ doccomment
            >
            >         greet(wrtyp)
            >
            >     Returns the greeting.
                  ^^^^^^^^^^^^^^^^^^^^^ doccomment
            >     """
            >     return name
        "#]],
    );
}
