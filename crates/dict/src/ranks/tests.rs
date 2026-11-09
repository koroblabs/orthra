use super::Ranks;
use crate::compiled::Bytes;

#[test]
fn a_word_is_ranked_by_its_count_and_listed_twice_keeps_the_better() {
    let compiled = Ranks::compile("я 168773\nне 143925\nсюди 900\n\nтреба 50\nне 3\n")
        .expect("frequencies compile");
    let table = Ranks::new(Bytes::Owned(compiled)).expect("a table reads");
    let ranks: Vec<String> = ["я", "Не", "сюди", "треба", "юда"]
        .iter()
        .map(|word| format!("{word} {:?}", table.rank(word)))
        .collect();
    expect_test::expect!["я Some(0), Не Some(1), сюди Some(2), треба Some(3), юда None"]
        .assert_eq(&ranks.join(", "));
}

#[test]
fn words_of_one_count_are_equals() {
    let compiled = Ranks::compile("the 90\nof 90\nlater 80\nletter 80\nleter 10\n")
        .expect("frequencies compile");
    let table = Ranks::new(Bytes::Owned(compiled)).expect("a table reads");
    let ranks: Vec<String> = ["the", "of", "later", "letter", "leter"]
        .iter()
        .map(|word| format!("{word} {:?}", table.rank(word)))
        .collect();
    expect_test::expect!["the Some(0), of Some(0), later Some(1), letter Some(1), leter Some(2)"]
        .assert_eq(&ranks.join(", "));
}

#[test]
fn a_dictionarys_forms_are_not_read_as_a_table() {
    let forms = crate::compiled::build("SET UTF-8\n", "1\nword\n")
        .expect("a dictionary compiles")
        .forms;
    expect_test::expect!["a frequency table of format 1, and this orthra reads 1380011595"]
        .assert_eq(&Ranks::new(Bytes::Owned(forms)).expect_err("refused").0);
}
