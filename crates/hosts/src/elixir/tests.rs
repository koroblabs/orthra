use expect_test::expect;

use crate::tests::check;

#[test]
fn elixir_prose_is_comments_docs_strings_and_declared_names() {
    check(
        "/elixir/greet.ex",
        r#"defmodule Foo.BarBaz do
  @moduledoc """
  The module doc, with `code`.

      iex> Foo.BarBaz.greet("x")
  """
  # A comment.
  @doc "Short doc."
  @spec greet(String.t()) :: String.t()
  def greet(name, opts \\ []) when is_binary(name) do
    msg = "Hello #{name}\n"
    ~s(text here) <> ~r/regex here/ <> ~w(some words) <> msg
    foo(:atom, key: 1)
  end

  defp hidden(a, {b, c}), do: a
  defmacro mac(x), do: x
  def unquoted, do: :ok
  @attr "value"
end
"#,
        expect![[r#"
            elixir
            > defmodule Foo.BarBaz do
                        ^^^^^^^^^^ declaration
            >   @moduledoc """
            >   The module doc, with `code`.
                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >
            >       iex> Foo.BarBaz.greet("x")
            >   """
            >   # A comment.
                ^^^^^^^^^^^^ comment
            >   @doc "Short doc."
                      ^^^^^^^^^^ doccomment
            >   @spec greet(String.t()) :: String.t()
            >   def greet(name, opts \\ []) when is_binary(name) do
                    ^^^^^ declaration
                          ^^^^ declaration
                                ^^^^ declaration
            >     msg = "Hello #{name}\n"
                         ^^^^^ string
            >     ~s(text here) <> ~r/regex here/ <> ~w(some words) <> msg
                     ^^^^^^^^^ string
            >     foo(:atom, key: 1)
                      ^^^^^ value
                             ^^^^ value
            >   end
            >
            >   defp hidden(a, {b, c}), do: a
                     ^^^^^^ declaration
                            ^ declaration
                                        ^^^ value
            >   defmacro mac(x), do: x
                         ^^^ declaration
                             ^ declaration
                                 ^^^ value
            >   def unquoted, do: :ok
                    ^^^^^^^^ declaration
                              ^^^ value
                                  ^^^ value
            >   @attr "value"
                       ^^^^^ string
            > end
        "#]],
    );
}

/// An `@doc` heredoc has no comment marker, so its indented block is four spaces past the doc's
/// own indentation. An `iex>` example and its result run to the next blank line.
#[test]
fn doctests_and_indented_blocks_in_a_doc_are_code() {
    check(
        "/lib/greet.ex",
        r#"defmodule Greet do
  @doc """
  Greets a person.

      iex> Greet.hello("boing")
      "Hello, boing"

  iex> Greet.hello("qzxv")
  "Hello, qzxv"

  Returns the greeting.
  """
  def hello(name), do: name
end
"#,
        expect![[r#"
            elixir
            > defmodule Greet do
                        ^^^^^ declaration
            >   @doc """
            >   Greets a person.
                ^^^^^^^^^^^^^^^^ doccomment
            >
            >       iex> Greet.hello("boing")
            >       "Hello, boing"
            >
            >   iex> Greet.hello("qzxv")
            >   "Hello, qzxv"
            >
            >   Returns the greeting.
                ^^^^^^^^^^^^^^^^^^^^^ doccomment
            >   """
            >   def hello(name), do: name
                    ^^^^^ declaration
                          ^^^^ declaration
                                 ^^^ value
            > end
        "#]],
    );
}
