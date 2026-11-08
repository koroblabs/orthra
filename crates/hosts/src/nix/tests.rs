use expect_test::expect;

use crate::tests::check;

#[test]
fn nix_prose_is_comments_strings_and_the_names_an_expression_declares() {
    check(
        "/default.nix",
        r##"{ pkgs, lib ? pkgs.lib, ... }:
# Builds the greeter.
let
  version = "1.2.3";
  /** Seperates the greeting. */
  greet = name: "Hello ${name}, welcome\n";
in
pkgs.stdenv.mkDerivation {
  pname = "greeter";
  inherit version;
  src = pkgs.fetchurl {
    url = "https://example.com/greeter-${version}.tar.gz";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };
  buildPhase = ''
    make recieve
    echo "${greet "x"}"
  '';
  installPhase = ''
    mkdir -p $out/bin
  '';
  /* a block comment */
  meta.description = "A greeter that occured once";
  "quoted-key" = ./path/to/file.nix;
}
"##,
        expect![[r#"
            nix
            > { pkgs, lib ? pkgs.lib, ... }:
                ^^^^ declaration
                      ^^^ declaration
            > # Builds the greeter.
              ^^^^^^^^^^^^^^^^^^^^^ comment
            > let
            >   version = "1.2.3";
                ^^^^^^^ declaration
                           ^^^^^ value
            >   /** Seperates the greeting. */
                    ^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >   greet = name: "Hello ${name}, welcome\n";
                ^^^^^ declaration
                        ^^^^ declaration
                               ^^^^^ value
                                            ^^^^^^^^^ string
            > in
            > pkgs.stdenv.mkDerivation {
            >   pname = "greeter";
                ^^^^^ declaration
                         ^^^^^^^ value
            >   inherit version;
            >   src = pkgs.fetchurl {
                ^^^ declaration
            >     url = "https://example.com/greeter-${version}.tar.gz";
                  ^^^ declaration
                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ value
                                                               ^^^^^^^ value
            >     hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
                  ^^^^ declaration
                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ value
            >   };
            >   buildPhase = ''
                ^^^^^^^^^^ declaration
                             ^^ value (+3 lines)
            >     make recieve
            >     echo "${greet "x"}"
            >   '';
            >   installPhase = ''
                ^^^^^^^^^^^^ declaration
                               ^^ value (+2 lines)
            >     mkdir -p $out/bin
            >   '';
            >   /* a block comment */
                ^^^^^^^^^^^^^^^^^^^^^ comment
            >   meta.description = "A greeter that occured once";
                ^^^^ declaration
                     ^^^^^^^^^^^ declaration
                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^ string
            >   "quoted-key" = ./path/to/file.nix;
                 ^^^^^^^^^^ value
            > }
        "#]],
    );
}
