use expect_test::expect;

use crate::tests::check;

#[test]
fn elm_prose_is_comments_docs_strings_and_declared_names() {
    check(
        "/src/Main.elm",
        r#"module Main exposing (main, Model)

{-| Module doc.

@docs main, Model
-}

import Html exposing (Html, div, text)

-- A line comment.
{- block -}

{-| Doc for Model. -}
type alias Model =
    { count : Int
    , userName : String
    }

type Msg
    = Increment
    | SetName String

port sendMessage : String -> Cmd msg

update : Msg -> Model -> Model
update msg model =
    case msg of
        Increment ->
            { model | count = model.count + 1 }

        SetName newName ->
            let
                helper x = x
                ( a, b ) = ( 1, 2 )
            in
            { model | userName = "Hello\nworld \u{1F600}" }

view : Model -> Html Msg
view model =
    div [] [ text """triple quoted""", text 'c' ]
"#,
        expect![[r#"
            elm
            > module Main exposing (main, Model)
            >
            > {-| Module doc.
              ^^^^^^^^^^^^^^^ doccomment
            >
            > @docs main, Model
            > -}
               ^ doccomment
            >
            > import Html exposing (Html, div, text)
            >
            > -- A line comment.
              ^^^^^^^^^^^^^^^^^^ comment
            > {- block -}
              ^^^^^^^^^^^ comment
            >
            > {-| Doc for Model. -}
              ^^^^^^^^^^^^^^^^^^^^^ doccomment
            > type alias Model =
                         ^^^^^ declaration
            >     { count : Int
                    ^^^^^ declaration
            >     , userName : String
                    ^^^^^^^^ declaration
            >     }
            >
            > type Msg
                   ^^^ declaration
            >     = Increment
                    ^^^^^^^^^ declaration
            >     | SetName String
                    ^^^^^^^ declaration
            >
            > port sendMessage : String -> Cmd msg
                   ^^^^^^^^^^^ declaration
            >
            > update : Msg -> Model -> Model
            > update msg model =
              ^^^^^^ declaration
                     ^^^ declaration
                         ^^^^^ declaration
            >     case msg of
            >         Increment ->
            >             { model | count = model.count + 1 }
            >
            >         SetName newName ->
                              ^^^^^^^ declaration
            >             let
            >                 helper x = x
                              ^^^^^^ declaration
                                     ^ declaration
            >                 ( a, b ) = ( 1, 2 )
                                ^ declaration
                                   ^ declaration
            >             in
            >             { model | userName = "Hello\nworld \u{1F600}" }
                                                ^^^^^ string
                                                       ^^^^^ string
            >
            > view : Model -> Html Msg
            > view model =
              ^^^^ declaration
                   ^^^^^ declaration
            >     div [] [ text """triple quoted""", text 'c' ]
                                   ^^^^^^^^^^^^^ string
        "#]],
    );
}

/// Elm's doc comment has no marker on its lines, and its examples are indented four spaces.
#[test]
fn an_indented_example_in_a_doc_is_code() {
    check(
        "/src/Greet.elm",
        r#"module Greet exposing (greet)

{-| Greets a person.

    greet "hahaha" == "Hello, hahaha"

Returns the greeting.
-}
greet : String -> String
greet name =
    name
"#,
        expect![[r#"
            elm
            > module Greet exposing (greet)
            >
            > {-| Greets a person.
              ^^^^^^^^^^^^^^^^^^^^ doccomment
            >
            >     greet "hahaha" == "Hello, hahaha"
            >
            > Returns the greeting.
              ^^^^^^^^^^^^^^^^^^^^^ doccomment
            > -}
               ^ doccomment
            > greet : String -> String
            > greet name =
              ^^^^^ declaration
                    ^^^^ declaration
            >     name
        "#]],
    );
}

/// A doc that holds nothing after its first line but an example: the example is indented past
/// the line the doc opens on.
#[test]
fn a_doc_of_one_line_and_an_example_reads_the_example_as_code() {
    check(
        "/src/Greet.elm",
        r#"{-| Repeat a string *n* times.

    repeat 3 "ha" == "hahaha"
-}
repeat : Int -> String -> String
repeat n s =
    s
"#,
        expect![[r#"
            elm
            > {-| Repeat a string *n* times.
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >
            >     repeat 3 "ha" == "hahaha"
            > -}
               ^ doccomment
            > repeat : Int -> String -> String
            > repeat n s =
              ^^^^^^ declaration
                     ^ declaration
                       ^ declaration
            >     s
        "#]],
    );
}
