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
