use super::Script;

#[test]
fn a_word_is_in_the_script_of_its_letters() {
    let cases = [
        ("receive", Script::Latin),
        ("кеш", Script::Cyrillic),
        ("м'ята", Script::Cyrillic),
        ("λόγος", Script::Greek),
        ("сount", Script::Mixed),
        ("Thе", Script::Mixed),
        ("x86_64", Script::Latin),
        ("42", Script::None),
        ("中文", Script::Other),
    ];
    for (word, script) in cases {
        assert_eq!(Script::of(word), script, "{word}");
    }
}
