use std::borrow::Cow;

use super::Frequencies;

#[test]
fn a_word_is_ranked_by_its_line_and_listed_twice_keeps_the_better() {
    let table = Frequencies::parse(Cow::Borrowed(
        "я 168773\nне 143925\nсюди 900\n\nтреба 50\nне 3\n",
    ));
    let ranks: Vec<String> = ["я", "Не", "сюди", "треба", "юда"]
        .iter()
        .map(|word| format!("{word} {:?}", table.rank(word)))
        .collect();
    expect_test::expect!["я Some(0), Не Some(1), сюди Some(2), треба Some(4), юда None"]
        .assert_eq(&ranks.join(", "));
}
