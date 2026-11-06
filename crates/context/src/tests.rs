use orthra_text::{TextRange, TextSize};

use super::{FakeScorer, ScoreError, Scorer, WINDOW_BYTES, Window, rerank};

fn range(text: &str, word: &str) -> TextRange {
    let start = text.find(word).expect("the word is in the text");
    TextRange::at(
        TextSize::try_from(start).unwrap(),
        TextSize::try_from(word.len()).unwrap(),
    )
}

fn shown(window: &Window) -> String {
    let slot = &window.sentence[window.slot];
    format!("{:?} slot {slot:?}", window.sentence)
}

#[test]
fn a_word_is_scored_in_its_line() {
    let text = "# Notes\nThe recieve call blocks.\nAnother line.\n";
    let window = Window::around(text, range(text, "recieve"));
    expect_test::expect![[r#""The recieve call blocks." slot "recieve""#]]
        .assert_eq(&shown(&window));
}

#[test]
fn a_long_line_is_cut_around_the_word_at_character_boundaries() {
    let before = "ж".repeat(WINDOW_BYTES);
    let after = "ї".repeat(WINDOW_BYTES);
    let text = format!("{before} помилкка {after}");
    let window = Window::around(&text, range(&text, "помилкка"));
    assert!(
        window.sentence.len() <= WINDOW_BYTES,
        "{}",
        window.sentence.len()
    );
    assert_eq!(&window.sentence[window.slot], "помилкка");
    let (left, right) = window.sentence.split_at(usize::from(window.slot.start()));
    assert!(left.chars().all(|c| c == 'ж' || c == ' '), "{left}");
    assert!(right.starts_with("помилкка "), "{right}");
}

#[test]
fn the_fake_scorer_turns_the_lexical_order_around() {
    let text = "Please recieve it.";
    let window = Window::around(text, range(text, "recieve"));
    let lexical = ["receive", "relieve", "reprieve"]
        .map(str::to_owned)
        .to_vec();
    let got = rerank(&FakeScorer, &window, lexical).unwrap();
    assert_eq!(got, ["reprieve", "relieve", "receive"]);
}

/// A scorer that answers what it is told to, whatever it is asked.
#[derive(Debug)]
struct Answers(Result<Vec<f32>, ScoreError>);

impl Scorer for Answers {
    fn score(&self, _: &str, _: TextRange, _: &[&str]) -> Result<Vec<f32>, ScoreError> {
        self.0.clone()
    }
}

#[test]
fn equal_scores_keep_the_lexical_order_and_a_wrong_answer_is_an_error() {
    let text = "Please recieve it.";
    let window = Window::around(text, range(text, "recieve"));
    let lexical = || {
        ["receive", "relieve", "reprieve"]
            .map(str::to_owned)
            .to_vec()
    };
    let answer =
        |scores: Result<Vec<f32>, ScoreError>| match rerank(&Answers(scores), &window, lexical()) {
            Ok(order) => order.join(" "),
            Err(err) => format!("error: {err}"),
        };
    let got = [
        answer(Ok(vec![0.0, 1.0, 1.0])),
        answer(Ok(vec![2.0, 2.0, 2.0])),
        answer(Ok(vec![1.0, 2.0])),
        answer(Ok(vec![1.0, f32::NAN, 0.0])),
        answer(Err(ScoreError("the model is not loaded".to_owned()))),
    ];
    expect_test::expect![[r#"
        relieve reprieve receive
        receive relieve reprieve
        error: 2 scores for 3 corrections
        error: a score of NaN
        error: the model is not loaded"#]]
    .assert_eq(&got.join("\n"));
}

#[test]
fn one_correction_or_none_is_not_scored() {
    let text = "Please recieve it.";
    let window = Window::around(text, range(text, "recieve"));
    let refuses = Answers(Err(ScoreError("asked".to_owned())));
    assert_eq!(
        rerank(&refuses, &window, vec!["receive".to_owned()]).unwrap(),
        ["receive"]
    );
    assert!(rerank(&refuses, &window, Vec::new()).unwrap().is_empty());
}
