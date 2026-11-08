use expect_test::expect;

use crate::tests::check;

#[test]
fn erlang_prose_is_comments_docs_strings_and_declared_names() {
    check(
        "/erlang/greet.erl",
        r#"%%% @doc Module doc.
-module(greet).
-include("greet.hrl").
-record(person, {name, age = 0 :: integer()}).
-define(MAX, 10).
-define(M(X), X + 1).
-type my_type() :: integer().
-callback init(term()) -> ok.
-doc "Attribute doc.".
-spec greet(string()) -> ok.
%% @doc Greet.
%% @spec greet(string()) -> ok
greet(Name) ->
    Msg = "Hello\n",
    io:format("~s~n", [Name]),
    #person{name = Name}, ?MAX, foo:bar(Msg), atom_value, <<"permessage-deflate">>.
other(A, B) when A > B -> ok;
other(_, _) -> error.
round_trip_test() -> "Hello teh world".
"#,
        expect![[r#"
            erlang
            > %%% @doc Module doc.
              ^^^^^^^^^^^^^^^^^^^^ comment
            > -module(greet).
            > -include("greet.hrl").
            > -record(person, {name, age = 0 :: integer()}).
                      ^^^^^^ declaration
                               ^^^^ declaration
                                     ^^^ declaration
            > -define(MAX, 10).
                      ^^^ declaration
            > -define(M(X), X + 1).
                      ^ declaration
            > -type my_type() :: integer().
                    ^^^^^^^ declaration
            > -callback init(term()) -> ok.
                        ^^^^ declaration
            > -doc "Attribute doc.".
                   ^^^^^^^^^^^^^^^^ doccomment
            > -spec greet(string()) -> ok.
            > %% @doc Greet.
              ^^^^^^^^^^^^^^ comment
            > %% @spec greet(string()) -> ok
            > greet(Name) ->
              ^^^^^ declaration
            >     Msg = "Hello\n",
                        ^^^^^^^^^ string
            >     io:format("~s~n", [Name]),
                            ^^^^^^ string
            >     #person{name = Name}, ?MAX, foo:bar(Msg), atom_value, <<"permessage-deflate">>.
                                                                          ^^^^^^^^^^^^^^^^^^^^ value
            > other(A, B) when A > B -> ok;
              ^^^^^ declaration
            > other(_, _) -> error.
              ^^^^^ declaration
            > round_trip_test() -> "Hello teh world".
              ^^^^^^^^^^^^^^^ declaration
        "#]],
    );
}

/// rebar's settings, an application's resource file, `erl -make`'s and an escript are Erlang terms
/// and code, as Zed's Erlang extension reads them.
#[test]
fn erlang_terms_files_are_erlang() {
    let hosts = crate::Hosts::builtin();
    let claimed: Vec<String> = [
        "rebar.config",
        "src/cowboy.app.src",
        "Emakefile",
        "bin/tool.escript",
    ]
    .iter()
    .map(|path| {
        format!(
            "{} {path}",
            hosts.for_path(path).map_or("-", |host| host.id())
        )
    })
    .collect();
    expect![[r#"
        erlang rebar.config
        erlang src/cowboy.app.src
        erlang Emakefile
        erlang bin/tool.escript
    "#]]
    .assert_eq(&(claimed.join("\n") + "\n"));
}
