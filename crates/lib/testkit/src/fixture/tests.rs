//! What orthra's flags mean. The format itself, markers and indentation included, is
//! lspkit-fixture's, and tested there.

use expect_test::expect;

use super::Fixture;

#[test]
fn a_flag_says_which_host_reads_a_file_and_whether_its_text_is_as_written() {
    let fixture = Fixture::parse(
        r#"
        //- /orthra.toml
        languages = ["en-US", "uk-UA"]

        //- /run.sh  raw  host=shell
        echo "$0 @a@b"

        //- /notes.md  expect
        > Teh end, $0 and @a@ as written.
          ^^^ known-typo

        //- /src/lib.rs
        /// Retruns the fi$0rst item.
        "#,
    );
    let rendered: Vec<String> = fixture
        .files()
        .iter()
        .map(|file| {
            format!(
                "{} host={:?} raw={} expected={:?}\n{}",
                file.path,
                file.host,
                file.raw,
                file.expected.as_ref().map(Vec::len),
                file.text
            )
        })
        .collect();
    expect![[r#"
        /orthra.toml host=None raw=false expected=None
        languages = ["en-US", "uk-UA"]
        ---
        /run.sh host=Some("shell") raw=true expected=None
        echo "$0 @a@b"
        ---
        /notes.md host=None raw=false expected=Some(1)
        Teh end, $0 and @a@ as written.
        ---
        /src/lib.rs host=None raw=false expected=None
        /// Retruns the first item.
    "#]]
    .assert_eq(&rendered.join("---\n"));

    assert_eq!(fixture.subject().path, "/src/lib.rs");
    let cursor = fixture.markers().cursor();
    assert_eq!(
        (
            cursor.path.as_str(),
            &fixture.subject().text[usize::from(cursor.offset)..][..4]
        ),
        ("/src/lib.rs", "rst ")
    );
    fixture.markers().assert_all_used();
}

#[test]
#[should_panic(expected = "unknown fixture flag \"opne\" on /a.txt")]
fn an_unknown_flag_is_a_panic() {
    Fixture::parse("//- /a.txt opne\none\n");
}

#[test]
#[should_panic(expected = "fixture paths start at the root, with the slash")]
fn a_lookup_spells_the_path_the_way_the_header_does() {
    Fixture::parse("//- /a.txt\none\n").file("a.txt");
}
