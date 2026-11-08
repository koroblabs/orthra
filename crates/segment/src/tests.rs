use expect_test::{Expect, expect};
use orthra_testkit::annotate::Annotation;
use orthra_text::{TextRange, TextSize, script::Script};

use crate::{Rules, Shape, words};

/// The words of `text`, rendered: shape, `code` for an identifier part or backtick code, and the script
/// when it is not Latin.
fn check(text: &str, backslash_escapes: bool, expect: Expect) {
    let rules = Rules {
        word_internal: &[('\'', Script::Latin), ('’', Script::Latin)],
        backslash_escapes,
        doubled_escapes: false,
    };
    let mut out = Vec::new();
    words(
        text,
        [TextRange::up_to(TextSize::of(text))],
        &rules,
        &mut out,
    );
    let annotations: Vec<Annotation> = out
        .iter()
        .map(|word| {
            let mut label = format!("{:?}", word.shape).to_lowercase();
            if word.code {
                label.push_str(" code");
            }
            if word.script != Script::Latin {
                label.push_str(&format!(" {:?}", word.script).to_lowercase());
            }
            Annotation::new(word.range, label)
        })
        .collect();
    expect.assert_eq(&Annotation::render(text, &annotations));
}

#[test]
fn escapes_and_placeholders_end_words() {
    check(
        "    format!(\"Hello\\nworld, {name}! Took %d ms\\tdone\")\n",
        true,
        expect![[r#"
            >     format!("Hello\nworld, {name}! Took %d ms\tdone")
                  ^^^^^^ lower
                           ^^^^^ title
                                  ^^^^^ lower
                                          ^^^^ lower
                                                 ^^^^ title
                                                             ^^^^ lower
        "#]],
    );
}

#[test]
fn identifiers_split_and_tokens_with_digits_are_not_words() {
    check(
        "pub struct HTTPServer;\npub const SHA256_LEN: usize = 32;\npub fn parse_json_v2(utf8_bytes: &[u8]) {}\npub fn route(x86_64: bool, k8s_ns: &str) {}\npub fn recieve_message() {}\n",
        false,
        expect![[r#"
            > pub struct HTTPServer;
              ^^^ lower
                  ^^^^^^ lower
                             ^^^^^^ title code
            > pub const SHA256_LEN: usize = 32;
              ^^^ lower
                  ^^^^^ lower
                                    ^^^^^ lower
            > pub fn parse_json_v2(utf8_bytes: &[u8]) {}
              ^^^ lower
                     ^^^^^ lower code
                           ^^^^ lower code
                                        ^^^^^ lower code
            > pub fn route(x86_64: bool, k8s_ns: &str) {}
              ^^^ lower
                     ^^^^^ lower code
                                   ^^^^ lower
                                                  ^^^ lower
            > pub fn recieve_message() {}
              ^^^ lower
                     ^^^^^^^ lower code
                             ^^^^^^^ lower code
        "#]],
    );
}

#[test]
fn camel_case_keeps_an_acronym_plural_whole() {
    check(
        "export function getURLsForIDs(ids: string[]) {}\nexport class XMLHttpRequestWrapper {}\nexport const recieveTimeoutMs = 3000;\n",
        false,
        expect![[r#"
            > export function getURLsForIDs(ids: string[]) {}
              ^^^^^^ lower
                     ^^^^^^^^ lower
                              ^^^ lower code
                                     ^^^ title code
                                            ^^^ lower
                                                 ^^^^^^ lower
            > export class XMLHttpRequestWrapper {}
              ^^^^^^ lower
                     ^^^^^ lower
                              ^^^^ title code
                                  ^^^^^^^ title code
                                         ^^^^^^^ title code
            > export const recieveTimeoutMs = 3000;
              ^^^^^^ lower
                     ^^^^^ lower
                           ^^^^^^^ lower code
                                  ^^^^^^^ title code
        "#]],
    );
}

#[test]
fn urls_paths_hashes_flags_and_versions_are_never_words() {
    check(
        "See https://docs.rs/x/y.html and mail jane@example.com; fixed in a3f9c2e, cached at\n~/.cache/orthra/v2/idx.bin, request 550e8400-e29b-41d4-a716-446655440000, color #ff00aa,\nflag --no-verify, version v1.2.3-rc.1, path C:\\Users\\jane.\n",
        false,
        expect![[r#"
            > See https://docs.rs/x/y.html and mail jane@example.com; fixed in a3f9c2e, cached at
              ^^^ title
                                           ^^^ lower
                                               ^^^^ lower
                                                                      ^^^^^ lower
                                                                                        ^^^^^^ lower
            > ~/.cache/orthra/v2/idx.bin, request 550e8400-e29b-41d4-a716-446655440000, color #ff00aa,
                                          ^^^^^^^ lower
                                                                                        ^^^^^ lower
            > flag --no-verify, version v1.2.3-rc.1, path C:\Users\jane.
              ^^^^ lower
                                ^^^^^^^ lower
                                                     ^^^^ lower
        "#]],
    );
}

#[test]
fn possessives_contractions_and_hyphens() {
    check(
        "The user's well-known, re-entrant cache doesn't evict what it’s given; Laravel’s too.\n",
        false,
        expect![[r#"
            > The user's well-known, re-entrant cache doesn't evict what it’s given; Laravel’s too.
              ^^^ title
                  ^^^^ lower
                         ^^^^ lower
                              ^^^^^ lower
                                        ^^^^^^^ lower
                                                ^^^^^ lower
                                                      ^^^^^^^ lower
                                                              ^^^^^ lower
                                                                    ^^^^ lower
                                                                              ^^^^^ lower
                                                                                     ^^^^^^^ title
                                                                                               ^^^ lower
        "#]],
    );
}

#[test]
fn numbers_units_backticks_and_scripts() {
    check(
        "Startup takes 100ms with a 4KiB buffer, 2x faster; call `qzxRender()` first. Привіт, сount!\n",
        false,
        expect![[r#"
            > Startup takes 100ms with a 4KiB buffer, 2x faster; call `qzxRender()` first. Привіт, сount!
              ^^^^^^^ title
                      ^^^^^ lower
                                  ^^^^ lower
                                              ^^^^^^ lower
                                                         ^^^^^^ lower
                                                                 ^^^^ lower
                                                                       ^^^ lower code
                                                                          ^^^^^^ title code
                                                                                    ^^^^^ lower
                                                                                           ^^^^^^ title cyrillic
                                                                                                   ^^^^^ lower mixed
        "#]],
    );
}

#[test]
fn shapes() {
    assert_eq!(Shape::of("receive"), Shape::Lower);
    assert_eq!(Shape::of("Receive"), Shape::Title);
    assert_eq!(Shape::of("RECEIVE"), Shape::Upper);
    assert_eq!(Shape::of("reCeive"), Shape::Mixed);
}

#[test]
fn a_file_extension_is_not_a_word() {
    check(
        "Edit services.yml and notes.txt, then self.name; e.g. the U.S. default.\n",
        false,
        expect![[r#"
            > Edit services.yml and notes.txt, then self.name; e.g. the U.S. default.
              ^^^^ title
                   ^^^^^^^^ lower
                                ^^^ lower
                                    ^^^^^ lower
                                               ^^^^ lower
                                                    ^^^^ lower
                                                                    ^^^ lower
                                                                             ^^^^^^^ lower
        "#]],
    );
}

#[test]
fn a_dotfile_is_not_a_word() {
    check(
        "Ignore .gitattributes and (.rgignore), keep /.cache and `.gir` out; wait...then.\n",
        false,
        expect![[r#"
            > Ignore .gitattributes and (.rgignore), keep /.cache and `.gir` out; wait...then.
              ^^^^^^ title
                                    ^^^ lower
                                                     ^^^^ lower
                                                                  ^^^ lower
                                                                             ^^^ lower
                                                                                  ^^^^ lower
                                                                                         ^^^^ lower
        "#]],
    );
}

#[test]
fn a_run_glued_to_an_at_sign_is_not_a_word() {
    check(
        "Ask @maintainer; see @Override and $v@variable@ariable here.\n",
        false,
        expect![[r#"
            > Ask @maintainer; see @Override and $v@variable@ariable here.
              ^^^ title
                               ^^^ lower
                                             ^^^ lower
                                                                     ^^^^ lower
        "#]],
    );
}

#[test]
fn a_glob_is_not_words() {
    check(
        "Match *.rs and [Mm]akefile, or src/** and *.{c,h}; why? Because.\n",
        false,
        expect![[r#"
            > Match *.rs and [Mm]akefile, or src/** and *.{c,h}; why? Because.
              ^^^^^ title
                         ^^^ lower
                                 ^^^^^^^ lower
                                                    ^^^ lower
                                                                 ^^^ lower
                                                                      ^^^^^^^ title
        "#]],
    );
}

#[test]
fn a_script_unlike_latin_ends_a_word_and_a_lookalike_does_not() {
    check(
        "# 补全窗口, preview窗口 и normal模式的光标; сount and Thе stay whole.\n",
        false,
        expect![[r#"
            > # 补全窗口, preview窗口 и normal模式的光标; сount and Thе stay whole.
                ^^^^ lower other
                      ^^^^^^^ lower
                                  ^^^^^^ lower
                                        ^^^^^ lower other
                                               ^^^^^ lower mixed
                                                     ^^^ lower
                                                         ^^^ title mixed
                                                             ^^^^ lower
                                                                  ^^^^^ lower
        "#]],
    );
}

#[test]
fn an_escaped_apostrophe_between_letters_is_an_apostrophe() {
    check(
        "It didn\\'t work for the user\\'s cache; App\\Models\\User and a\\'b stay apart.\n",
        false,
        expect![[r#"
            > It didn\'t work for the user\'s cache; App\Models\User and a\'b stay apart.
                 ^^^^^^^ lower
                         ^^^^ lower
                              ^^^ lower
                                  ^^^ lower
                                      ^^^^ lower
                                              ^^^^^ lower
                                                     ^^^ title
                                                         ^^^^^^ title
                                                                ^^^^ title
                                                                     ^^^ lower
                                                                              ^^^^ lower
                                                                                   ^^^^^ lower
        "#]],
    );
}

#[test]
fn a_language_or_locale_tag_is_never_a_word_and_a_word_like_one_still_is() {
    check(
        "Tags: uk-UA, en_US, zh-Hant-TW, es-419. Prose: non-Java, abc-Defg, foo-BAR, pre-war.\n",
        false,
        expect![[r#"
            > Tags: uk-UA, en_US, zh-Hant-TW, es-419. Prose: non-Java, abc-Defg, foo-BAR, pre-war.
              ^^^^ title
                                                      ^^^^^ title
                                                             ^^^ lower
                                                                 ^^^^ title
                                                                       ^^^ lower
                                                                           ^^^^ title
                                                                                 ^^^ lower
                                                                                          ^^^ lower
                                                                                              ^^^ lower
        "#]],
    );
}

#[test]
fn a_character_joins_words_only_in_the_script_whose_pack_says_so() {
    // uk-UA's hyphen and apostrophes, beside en-US's apostrophes.
    let rules = Rules {
        word_internal: &[
            ('\'', Script::Latin),
            ('-', Script::Cyrillic),
            ('\'', Script::Cyrillic),
        ],
        backslash_escapes: false,
        doubled_escapes: false,
    };
    let text = "будь-який м'ята, codebook-lsp don't.\n";
    let mut out = Vec::new();
    words(
        text,
        [TextRange::up_to(TextSize::of(text))],
        &rules,
        &mut out,
    );
    let shown: Vec<&str> = out.iter().map(|word| &text[word.range]).collect();
    expect!["будь-який | м'ята | codebook | lsp | don't"].assert_eq(&shown.join(" | "));
}

#[test]
fn a_doubled_escape_is_an_escape_where_strings_are_read_twice() {
    // A shell script's `printf "\\nexport ..."`: the shell makes `\\` a backslash, and printf makes
    // `\n` a newline. A host that reads its strings once keeps the `n` as the word's.
    let text = r"printf \\nexport the path";
    let shown = |doubled_escapes: bool| {
        let rules = Rules {
            word_internal: &[],
            backslash_escapes: true,
            doubled_escapes,
        };
        let mut out = Vec::new();
        words(
            text,
            [TextRange::up_to(TextSize::of(text))],
            &rules,
            &mut out,
        );
        out.iter()
            .map(|word| &text[word.range])
            .collect::<Vec<_>>()
            .join(" | ")
    };
    expect![[r#"
        printf | export | the | path
        printf | nexport | the | path"#]]
    .assert_eq(&format!("{}\n{}", shown(true), shown(false)));
}
