use expect_test::expect;
use orthra_testkit::fixture::Fixture;

use super::toy::{Bug, FIXTURE, Toy};
use crate::{
    dimensions::check_all,
    subject::{Edit, Observed},
};

#[test]
fn a_correct_subject_holds_in_every_dimension() {
    let fixture = Fixture::parse(FIXTURE);
    let text = &fixture.subject().text;

    let mut edited = text.clone();
    let mut edits = Vec::new();
    for (old, new) in [
        ("recieve", "receive"),
        ("end", "teh end"),
        ("food", "fo od"),
    ] {
        let edit = Edit::replace(&edited, old, new);
        edit.apply(&mut edited);
        edits.push(edit);
    }

    let findings = check_all(&Toy(Bug::None), &fixture, &edits);
    expect![[r#"
        > The cat will recieve their food.
                       ^^^^^^^ known-typo
        > Teh end: teh.
          ^^^ known-typo
                   ^^^ known-typo
    "#]]
    .assert_eq(&Observed::render(text, &findings));
}
