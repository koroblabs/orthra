use expect_test::expect;

use crate::tests::check;

#[test]
fn shell_raw_strings_and_heredocs_are_values_or_silent() {
    check(
        "/bin/probe.sh",
        r#"curl -d '{"name":"frtc-probe"}' | jq -r '.data.name'
cat <<EOF
[Unit]
Description=Runs the probe
EOF
text=$'Hello\tworld'
echo "frtc-probe" "Hello, $USER"
"#,
        expect![[r#"
            shell
            > curl -d '{"name":"frtc-probe"}' | jq -r '.data.name'
                      ^^^^^^^^^^^^^^^^^^^^^^^ value
                                                      ^^^^^^^^^^^^ value
            > cat <<EOF
            > [Unit]
            > Description=Runs the probe
            > EOF
            > text=$'Hello\tworld'
              ^^^^ declaration
                   ^^^^^^^^^^^^^^^ string
            > echo "frtc-probe" "Hello, $USER"
                    ^^^^^^^^^^ value
                                 ^^^^^^ value
        "#]],
    );
}

#[test]
fn shell_prose_is_comments_strings_and_declared_names() {
    check(
        "/bin/greet.sh",
        r#"#!/usr/bin/env bash
# Greets whoever asks.
set -eu

greeting="Hello world, $USER"
export GREETING_LANGUAGE=en

greet() {
  local person="$1"
  echo "Hello\nworld, ${person}!"
  printf 'Done %s\n' "$(date)"
  for file in *.txt; do
    cat "$file"
  done
}

greet --no-verify teh_flag
"#,
        expect![[r#"
            shell
            > #!/usr/bin/env bash
            > # Greets whoever asks.
              ^^^^^^^^^^^^^^^^^^^^^^ comment
            > set -eu
            >
            > greeting="Hello world, $USER"
              ^^^^^^^^ declaration
                        ^^^^^^^^^^^^ string
            > export GREETING_LANGUAGE=en
                     ^^^^^^^^^^^^^^^^^ declaration
            >
            > greet() {
              ^^^^^ declaration
            >   local person="$1"
                      ^^^^^^ declaration
            >   echo "Hello\nworld, ${person}!"
                      ^^^^^^^^^^^^^ string
                                             ^ value
            >   printf 'Done %s\n' "$(date)"
                       ^^^^^^^^^^^ value
            >   for file in *.txt; do
                    ^^^^ declaration
            >     cat "$file"
            >   done
            > }
            >
            > greet --no-verify teh_flag
        "#]],
    );
}

/// zsh's own syntax can open a string the bash grammar never closes, and the commands after it then
/// read as its text. What the grammar could not parse is not checked: ripgrep's `rg.zsh`, this
/// function, had `compset` reported five times.
#[test]
fn what_the_grammar_cannot_parse_is_not_words() {
    check(
        "/complete/_rg.zsh",
        r##"# Complete custom hyperlink format strings
(( $+functions[_rg_hyperlink_format_strings] )) ||
_rg_hyperlink_format_strings() {
  local bs
  local -a pfx sfx rmv

  [[ -n $compstate[quote] ]] || bs=\\

  sfx=( -S "$bs}" )
  rmv=( -r '}\-' )

  compset -S "$bs{*"
  compset -S "$bs}*" && sfx=( -S '' )
  compset -P '*}'
}
"##,
        expect![[r#"
            shell
            > # Complete custom hyperlink format strings
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > (( $+functions[_rg_hyperlink_format_strings] )) ||
            > _rg_hyperlink_format_strings() {
            >   local bs
            >   local -a pfx sfx rmv
            >
            >   [[ -n $compstate[quote] ]] || bs=\\
            >
            >   sfx=( -S "$bs}" )
            >   rmv=( -r '}\-' )
            >
            >   compset -S "$bs{*"
            >   compset -S "$bs}*" && sfx=( -S '' )
            >   compset -P '*}'
            > }
        "#]],
    );
}
