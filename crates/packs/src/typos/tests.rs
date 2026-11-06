use super::KnownTypos;

/// Every fix for each word, as `word: fix, fix`, or `word: -` for none.
fn lookups(typos: &KnownTypos, words: &[&str]) -> String {
    words
        .iter()
        .map(|word| match typos.get(word) {
            Some(fixes) => format!("{word}: {}", fixes.iter().collect::<Vec<_>>().join(", ")),
            None => format!("{word}: -"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_typo_is_found_whatever_its_case_with_every_fix_in_order() {
    let typos =
        KnownTypos::parse("# a comment\naache\tache\tcache\nlonely\nrecieve\treceive\n").unwrap();
    assert_eq!(typos.len(), 2, "a line with no fix is not an entry");
    expect_test::expect![[r#"
        Recieve: receive
        AACHE: ache, cache
        receive: -
        lonely: -"#]]
    .assert_eq(&lookups(&typos, &["Recieve", "AACHE", "receive", "lonely"]));
}

#[test]
fn a_list_in_any_order_with_any_line_breaks_is_read_whole() {
    let typos = KnownTypos::parse("wierd\tweird\r\naache\tache\r\nteh\tthe").unwrap();
    expect_test::expect![[r#"
        aache: ache
        teh: the
        wierd: weird
        zzz: -"#]]
    .assert_eq(&lookups(&typos, &["aache", "teh", "wierd", "zzz"]));
}

#[test]
fn an_entry_that_could_never_be_found_is_an_error() {
    expect_test::expect![[r#"
        Err("\"Teh\" is not written in lower case")
        Err("\"teh\" is listed twice")"#]]
    .assert_eq(&format!(
        "{:?}\n{:?}",
        KnownTypos::parse("Teh\tthe\n").map(|typos| typos.len()),
        KnownTypos::parse("teh\tthe\nwierd\tweird\nteh\tten\n").map(|typos| typos.len()),
    ));
}

#[test]
fn every_built_in_list_is_read() {
    let typos = KnownTypos::parse(include_str!("../../../../packs/en-US/typos.tsv")).unwrap();
    assert!(typos.len() > 90_000, "{} entries", typos.len());
}
