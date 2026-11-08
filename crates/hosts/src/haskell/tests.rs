use expect_test::expect;

use crate::tests::check;

#[test]
fn haskell_prose_is_comments_haddock_strings_and_declared_names() {
    check(
        "/src/Greet.hs",
        r#"{-# LANGUAGE OverloadedStrings #-}
-- | Module doc.
module Greet.Core (greet) where

import Data.List (sort)

-- A line comment.
{- A block
comment -}

-- | Builds the greeting.
greet :: Int -> String
greet person n = "Hello\nworld, " ++ show n
  where helper y = y

{-| A color. -}
data Color = Red | Green { fieldOne :: Int, fieldTwo :: String }
newtype Wrap = Wrap { unWrap :: Int }
type Alias = Int

class Shape a where
  area :: a -> Int

instance Shape Int where
  area side = side

(<+>) :: Int -> Int -> Int
a <+> b = a + b

main = do
  let quiet = 'q'
  print quiet
"#,
        expect![[r#"
            haskell
            > {-# LANGUAGE OverloadedStrings #-}
            > -- | Module doc.
                 ^^^^^^^^^^^^^ doccomment
            > module Greet.Core (greet) where
            >
            > import Data.List (sort)
            >
            > -- A line comment.
              ^^^^^^^^^^^^^^^^^^ comment
            > {- A block
              ^^^^^^^^^^ comment (+1 lines)
            > comment -}
            >
            > -- | Builds the greeting.
                 ^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > greet :: Int -> String
            > greet person n = "Hello\nworld, " ++ show n
              ^^^^^ declaration
                    ^^^^^^ declaration
                           ^ declaration
                               ^^^^^^^^^^^^^^^^ string
            >   where helper y = y
                      ^^^^^^ declaration
                             ^ declaration
            >
            > {-| A color. -}
              ^^^^^^^^^^^^^^^ doccomment
            > data Color = Red | Green { fieldOne :: Int, fieldTwo :: String }
                   ^^^^^ declaration
                           ^^^ declaration
                                 ^^^^^ declaration
                                         ^^^^^^^^ declaration
                                                          ^^^^^^^^ declaration
            > newtype Wrap = Wrap { unWrap :: Int }
                      ^^^^ declaration
                             ^^^^ declaration
                                    ^^^^^^ declaration
            > type Alias = Int
                   ^^^^^ declaration
            >
            > class Shape a where
                    ^^^^^ declaration
            >   area :: a -> Int
                ^^^^ declaration
            >
            > instance Shape Int where
            >   area side = side
                     ^^^^ declaration
            >
            > (<+>) :: Int -> Int -> Int
            > a <+> b = a + b
            >
            > main = do
              ^^^^ declaration
            >   let quiet = 'q'
                    ^^^^^ declaration
            >   print quiet
        "#]],
    );
}

/// Haddock's code is a `>` bird track and a `>>>` example with its result, each running to the
/// next blank line: `xxxa` and `cdab` there are code. The prose after the blank line is read.
#[test]
fn haddock_bird_tracks_and_examples_are_code() {
    check(
        "/src/Seq.hs",
        r#"-- | Reverses a sequence.
--
-- > reverse (fromList "xxxa") == fromList "axxx"
--
-- >>> toList (reverse (fromList "cdab"))
-- "badc"
--
-- Runs in linear time.
reverse :: Seq a -> Seq a
reverse s = s
"#,
        expect![[r#"
            haskell
            > -- | Reverses a sequence.
                 ^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > --
            > -- > reverse (fromList "xxxa") == fromList "axxx"
            > --
            > -- >>> toList (reverse (fromList "cdab"))
            > -- "badc"
            > --
            > -- Runs in linear time.
                 ^^^^^^^^^^^^^^^^^^^^ doccomment
            > reverse :: Seq a -> Seq a
            > reverse s = s
              ^^^^^^^ declaration
                      ^ declaration
        "#]],
    );
}
