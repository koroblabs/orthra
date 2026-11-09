//! A built-in pack's frequency file is compiled when orthra is (`build.rs`): embedded once under
//! its text's name, however many packs name it, and its text left out of the binary.

use expect_test::expect;

use super::{LazyFrequencies, Pack, Text};

#[test]
fn englishs_frequencies_are_one_compiled_table() {
    let (us, gb) = (
        Pack::builtin("en-US").unwrap(),
        Pack::builtin("en-GB").unwrap(),
    );
    let table = |pack: &Pack| -> *const u8 {
        match pack.frequency.as_ref().map(|frequency| &frequency.text) {
            Some(Text::Compiled(bytes)) => bytes.as_ptr(),
            other => panic!("{} frequencies are {other:?}", pack.tag),
        }
    };
    assert_eq!(table(&us), table(&gb), "en-GB reads en-US's table");
    assert_eq!(Pack::builtin_file("en-US", "frequency.txt"), None);

    let frequencies: &LazyFrequencies = us.frequency.as_ref().unwrap();
    let ranks: Vec<String> = [
        "the", "Later", "letter", "liter", "leer", "lefter", "orthra",
    ]
    .iter()
    .map(|word| format!("{word} {:?}", frequencies.get().rank(word)))
    .collect();
    expect!["the Some(0), Later Some(0), letter Some(0), liter Some(1), leer Some(2), lefter Some(2), orthra None"].assert_eq(&ranks.join(", "));
}
