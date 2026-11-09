use super::{Bytes, CompiledDict, FORMAT, build, build_in, within::Within};
use crate::{Dictionary, hunspell::HunspellDict};
use fst::{IntoStreamer, Set, Streamer};

/// A dictionary small enough to read, with each rule a compiled dictionary repeats from spellbook:
/// suffixes with a strip, an empty strip and conditions (`[^гкх]а`, `[гкх]а`, `ь`, `[^ь]`), a
/// stress mark to ignore, three apostrophes and Latin `a` converted on input, a hyphen to break
/// at, stems in three casings, a stem so short a suffix reaches its first letter (`ва`, `ви`), and
/// flags of two bytes that spellbook reads a byte at a time, as
/// dict_uk writes them: `\u{80}` and `\u{81}` are both the byte C2 in a header.
const AFF: &str = "SET UTF-8
TRY оаиі
IGNORE \u{301}
ICONV 3
ICONV ʼ '
ICONV ’ '
ICONV a 0
BREAK 1
BREAK -

SFX A Y 3
SFX A а и [^гкх]а
SFX A а і [гкх]а
SFX A 0 ю а

SFX B Y 2
SFX B 0 ові [^ь]
SFX B ь ю ь

SFX \u{80} Y 1
SFX \u{80} 0 ам [^ь]

SFX \u{81} Y 1
SFX \u{81} 0 ах [^ь]
";

const DIC: &str = "9
ва/A
вода/A
рука/A
кінь/B
стіл/B\u{80}
Київ/B
НАТО
м'ята/A
будь-який
";

fn both() -> (CompiledDict, HunspellDict) {
    (
        {
            let built = build(AFF, DIC).unwrap();
            let marks = (!built.marks.is_empty()).then_some(Bytes::Owned(built.marks));
            CompiledDict::new(AFF, Bytes::Owned(built.forms), marks).unwrap()
        },
        HunspellDict::new(AFF, DIC).unwrap(),
    )
}

/// Every form the dictionary makes, as the compiled one lists them.
fn forms(compiled: &CompiledDict) -> Vec<String> {
    let mut stream = compiled.forms.stream();
    let mut out = Vec::new();
    while let Some(form) = stream.next() {
        out.push(String::from_utf8(form.to_vec()).unwrap());
    }
    out
}

#[test]
fn it_lists_every_form_the_stems_and_suffixes_make() {
    let (compiled, _) = both();
    expect_test::expect![[r#"
        Київ
        Київові
        НАТО
        будь-який
        ва
        ваю
        ви
        вода
        водаю
        води
        кінь
        кіню
        м'ята
        м'ятаю
        м'яти
        рука
        рукаю
        рукі
        стіл
        стілам
        стілах
        стілові"#]]
    .assert_eq(&forms(&compiled).join("\n"));
}

#[test]
fn building_in_passes_of_any_size_makes_the_same_dictionary() {
    let whole = build_in(AFF, DIC, usize::MAX).unwrap();
    let whole = (whole.forms, whole.marks);
    // A pass per key, and a pass that a single stem's forms overflow.
    for pass_bytes in [0, 1, 16, 40] {
        assert!(
            build_in(AFF, DIC, pass_bytes)
                .map(|built| (built.forms, built.marks))
                .unwrap()
                == whole,
            "passes of {pass_bytes} bytes"
        );
    }
}

/// The words to ask both about: every form, in every casing, and every form one edit away by a
/// deletion or a substitution, with what the rules turn on written out by hand.
fn words(compiled: &CompiledDict) -> Vec<String> {
    let mut words: Vec<String> = [
        "123",
        "1.5",
        "-2",
        "1.",
        "1..2",
        "вода.",
        "вода..",
        "мʼята",
        "м’ята",
        "М'ЯТА",
        "МʼЯТИ",
        "водa",
        "вoда",
        "во\u{301}да",
        "вода-рука",
        "вода-",
        "-вода",
        "вода--рука",
        "будь-який",
        "Будь-який",
        "БУДЬ-ЯКИЙ",
        "київ",
        "КИЇВ",
        "КиЇв",
        "Нато",
        "нато",
        "",
        "-",
        "'",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for form in forms(compiled) {
        let chars: Vec<char> = form.chars().collect();
        words.push(form.to_uppercase());
        words.push(super::title(&form));
        words.push(form.to_lowercase());
        for at in 0..chars.len() {
            let mut deleted = chars.clone();
            deleted.remove(at);
            words.push(deleted.into_iter().collect());
            for c in ['а', 'и', 'і', 'ю', '-'] {
                let mut changed = chars.clone();
                changed[at] = c;
                words.push(changed.into_iter().collect());
            }
        }
        words.push(form);
    }
    words.sort();
    words.dedup();
    words
}

#[test]
fn it_accepts_what_spellbook_accepts_and_nothing_else() {
    let (compiled, hunspell) = both();
    let words = words(&compiled);
    assert!(words.len() > 300, "{} words", words.len());
    let disagree: Vec<String> = words
        .iter()
        .filter(|word| compiled.check(word) != hunspell.check(word))
        .map(|word| format!("{word:?}: spellbook {}", hunspell.check(word)))
        .collect();
    assert!(disagree.is_empty(), "{}", disagree.join("\n"));

    // What the rules accepted beside the forms themselves.
    let forms = forms(&compiled);
    let accepted: Vec<&String> = words
        .iter()
        .filter(|word| compiled.check(word) && !forms.contains(word))
        .collect();
    expect_test::expect![[r#"["", "-2", "1.", "1.5", "123", "БУДЬ-ЯКИЙ", "Будь-який", "ВА", "ВАЮ", "ВИ", "ВОДА", "ВОДАЮ", "ВОДИ", "Ва", "Ваю", "Ви", "Вода", "Водаю", "Води", "КІНЬ", "КІНЮ", "КИЇВ", "КИЇВОВІ", "Кінь", "Кіню", "М'ЯТА", "М'ЯТАЮ", "М'ЯТИ", "М'ята", "М'ятаю", "М'яти", "МʼЯТИ", "РУКІ", "РУКА", "РУКАЮ", "Рука", "Рукаю", "Рукі", "СТІЛ", "СТІЛАМ", "СТІЛАХ", "СТІЛОВІ", "Стіл", "Стілам", "Стілах", "Стілові", "во\u{301}да", "вода-рука", "вода.", "вода..", "мʼята", "м’ята"]"#]]
    .assert_eq(&format!("{accepted:?}"));
}

#[test]
fn a_correction_is_every_form_within_two_edits_nearest_first_in_the_casing_written() {
    let (compiled, _) = both();
    let shown = |word: &str| {
        let mut out = Vec::new();
        compiled.suggest(word, &mut out);
        let out: Vec<String> = out
            .iter()
            .map(|candidate| format!("{} {}", candidate.word, candidate.edits))
            .collect();
        format!("{word}: {}", out.join(", "))
    };
    let got: Vec<String> = [
        // A Latin `a` is converted to `0` first, as spellbook converts it, and nothing is near.
        "вадa",
        // Four letters or fewer: one edit.
        "вада",
        // Longer: two, a swap of neighbors counted as one.
        "рукою",
        "стлові",
        // Capitalized, and a name found capitalized.
        "Вада",
        "Киівові",
        "КИЕВІ",
        // An apostrophe written another way finds the form written with `'`.
        "мʼяті",
    ]
    .iter()
    .map(|word| shown(word))
    .collect();
    expect_test::expect![[r#"
        вадa: 
        вада: вода 1
        рукою: рукаю 1, рука 2, рукі 2
        стлові: стілові 1
        Вада: Вода 1
        Киівові: Київові 1
        КИЕВІ: КИЇВ 2
        мʼяті: м'ята 1, м'яти 1, м'ятаю 2"#]]
    .assert_eq(&got.join("\n"));
}

#[test]
fn what_a_list_of_forms_cannot_hold_is_refused_by_name() {
    let refused = |aff: &str, dic: &str| match build(aff, dic) {
        Ok(_) => "compiled".to_owned(),
        Err(err) => err.0,
    };
    let got = [
        refused("COMPOUNDRULE 1\nCOMPOUNDRULE n*\n", "1\nвода/n\n"),
        refused("COMPOUNDFLAG X\n", "1\nвода\n"),
        refused("SFX A Y 1\nSFX A 0 и/B .\n", "1\nвода/A\n"),
        refused("SET ISO8859-1\n", "1\nвода\n"),
        refused("SFX A Y 2\nSFX A 0 и .\n", "1\nвода/A\n"),
        refused("", "2\nвода po:noun\nрука\n"),
        refused("", "1\nвода\\/рука\n"),
    ];
    expect_test::expect![[r#"
        .dic line 2: "вода" is in a compound rule, which is compiled only for numbers; use `dictionary.hunspell`
        .aff line 1: COMPOUNDFLAG is not something a compiled dictionary can hold; use `dictionary.hunspell`
        .aff line 2: SFX и/B: an affix that takes further affixes is not compiled; use `dictionary.hunspell`
        .aff line 1: SET ISO8859-1: only UTF-8 is read
        .aff line 1: SFX has too few rows
        .dic line 2: "вода po:noun" has a space, a backslash or a leading slash, which is not compiled
        .dic line 2: "вода\\/рука" has a space, a backslash or a leading slash, which is not compiled"#]].assert_eq(&got.join("\n"));
}

#[test]
fn a_compiled_dictionary_of_another_format_is_refused() {
    let mut builder = fst::raw::Builder::new_type(Vec::new(), FORMAT + 1).unwrap();
    builder.add("вода").unwrap();
    let err =
        CompiledDict::new(AFF, Bytes::Owned(builder.into_inner().unwrap()), None).unwrap_err();
    expect_test::expect![
        "a compiled dictionary of format 2, and this orthra reads 1: install the pack again"
    ]
    .assert_eq(&err.0);
}

/// Edits between `a` and `b` with no swaps: what the automaton counts.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut previous = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let next = (previous + usize::from(a[i - 1] != b[j - 1]))
                .min(row[j] + 1)
                .min(row[j - 1] + 1);
            previous = row[j];
            row[j] = next;
        }
    }
    row[b.len()]
}

#[test]
fn the_automaton_finds_exactly_the_words_within_the_edits() {
    let (compiled, _) = both();
    let mut list = forms(&compiled);
    list.extend(["a", "ab", "abc", "ba", "b", "😀", "a😀b", "ю"].map(str::to_owned));
    list.sort();
    list.dedup();
    let set = Set::from_iter(&list).unwrap();
    let mut queries = words(&compiled);
    queries.extend(["", "a", "ab", "ac", "a😀", "😀b", "ю😀"].map(str::to_owned));
    for query in &queries {
        for edits in 0..=2 {
            let mut stream = set.search(Within::new(query, edits).unwrap()).into_stream();
            let mut found = Vec::new();
            while let Some(word) = stream.next() {
                found.push(String::from_utf8(word.to_vec()).unwrap());
            }
            let wanted: Vec<String> = list
                .iter()
                .filter(|word| levenshtein(query, word) <= edits)
                .cloned()
                .collect();
            assert_eq!(found, wanted, "{query:?} within {edits}");
        }
    }
}

#[test]
fn a_query_too_long_for_a_row_of_bits_finds_nothing_rather_than_wrong_words() {
    assert!(Within::new(&"а".repeat(super::within::LONGEST), 2).is_some());
    assert!(Within::new(&"а".repeat(super::within::LONGEST + 1), 2).is_none());
    assert!(Within::new("а", super::within::MOST_EDITS + 1).is_none());
}

/// English's shape, as SCOWL writes it: a prefix that combines with a suffix (`Y`) and one that
/// does not, a suffix that does not, words never offered (`!`), ordinals that compound from digits
/// with a part that is never a word alone (`c`), and a word of mixed case.
const AFF_EN: &str = "SET UTF-8
TRY esianrtolcdugmphbyfvkwz
ICONV 1
ICONV ’ '
NOSUGGEST !
COMPOUNDMIN 1
ONLYINCOMPOUND c
COMPOUNDRULE 2
COMPOUNDRULE n*1t
COMPOUNDRULE n*mp
WORDCHARS 0123456789

PFX A Y 1
PFX A 0 re .

PFX U N 1
PFX U 0 un .

SFX D Y 2
SFX D 0 d e
SFX D 0 ed [^ey]

SFX S N 1
SFX S 0 s .
";

const DIC_EN: &str = "9
do/AU
use/ADS
work/AD
GitHub/S
damn/!S
1/n1
0th/pt
1st/p
1th/tc
";

fn both_en() -> (CompiledDict, HunspellDict) {
    let built = build(AFF_EN, DIC_EN).unwrap();
    let marks = (!built.marks.is_empty()).then_some(Bytes::Owned(built.marks));
    (
        CompiledDict::new(AFF_EN, Bytes::Owned(built.forms), marks).unwrap(),
        HunspellDict::new(AFF_EN, DIC_EN).unwrap(),
    )
}

/// The forms: a prefix on the stem and on its suffixed forms where both combine (`reused`, not
/// `reuses`), a prefix that does not combine on the stem alone (`undo`), the ordinals' parts but
/// `1th`, which is only ever inside a compound, and `damn`, a word.
#[test]
fn english_prefixes_combine_with_suffixes_where_both_say_so() {
    let (compiled, _) = both_en();
    expect_test::expect!["0th 1 1st GitHub GitHubs damn damns do redo reuse reused rework reworked undo use used uses work worked"].assert_eq(&forms(&compiled).join(" "));
}

/// spellbook's verdict on every form in every casing, on what the cross product leaves out, on
/// `Github`, which only `GITHUB` makes a word of, and on a word never offered. Words with digits
/// are left to spellbook's compounding: the segmenter never hands a dictionary one.
#[test]
fn english_is_accepted_as_spellbook_accepts_it() {
    let (compiled, hunspell) = both_en();
    let mut words: Vec<String> = forms(&compiled)
        .iter()
        .filter(|form| !form.starts_with(|c: char| c.is_ascii_digit()))
        .flat_map(|form| [form.clone(), form.to_uppercase(), super::title(form)])
        .collect();
    words.extend(
        [
            "reuses", "unused", "reworks", "undid", "Github", "GITHUB", "GITHUBS", "Damn", "DAMNS",
        ]
        .map(str::to_owned),
    );
    let disagree: Vec<String> = words
        .iter()
        .filter(|word| compiled.check(word) != hunspell.check(word))
        .map(|word| format!("{word:?}: spellbook {}", hunspell.check(word)))
        .collect();
    assert!(disagree.is_empty(), "{}", disagree.join("\n"));
}

/// A word never offered is a word, and no correction offers it: `dams` is near `damn` and `damns`
/// alone, and gets nothing, where `worke` gets the words near it.
#[test]
fn a_word_never_offered_is_accepted_and_not_offered() {
    let (compiled, _) = both_en();
    let offered = |word: &str| {
        let mut offered = Vec::new();
        compiled.suggest(word, &mut offered);
        offered
            .into_iter()
            .map(|candidate| candidate.word)
            .collect::<Vec<_>>()
    };
    expect_test::expect![[r#"true [] ["work", "worked"]"#]].assert_eq(&format!(
        "{} {:?} {:?}",
        compiled.check("damns"),
        offered("dams"),
        offered("worke")
    ));
}
