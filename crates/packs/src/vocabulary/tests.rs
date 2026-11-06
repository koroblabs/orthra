use super::Vocabulary;

const LIST: &str = "# A comment\nINOUT\nkubectl\nPostgreSQL\nrustup\n";

#[test]
fn a_word_is_found_in_any_case_and_a_word_in_capitals_only_in_capitals() {
    let vocabulary = Vocabulary::new(&[LIST]);
    let got: Vec<String> = [
        "kubectl",
        "Kubectl",
        "KUBECTL",
        "postgresql",
        "PostgreSQL",
        "rustup",
        "INOUT",
        "inout",
        "Inout",
        "kubect",
        "rustups",
        "comment",
    ]
    .iter()
    .map(|word| format!("{word} {}", vocabulary.accepts(word)))
    .collect();
    expect_test::expect![[r#"
        kubectl true
        Kubectl true
        KUBECTL true
        postgresql true
        PostgreSQL true
        rustup true
        INOUT true
        inout false
        Inout false
        kubect false
        rustups false
        comment false"#]]
    .assert_eq(&got.join("\n"));
}

#[test]
fn every_built_in_vocabulary_is_sorted_by_lower_case_with_one_spelling_of_each() {
    for (path, text) in crate::builtin::FILES {
        if !path.contains("vocabulary") {
            continue;
        }
        let entries: Vec<String> = text
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_lowercase)
            .collect();
        let unsorted = entries.windows(2).find(|pair| pair[0] >= pair[1]);
        assert!(
            unsorted.is_none(),
            "{path}: {unsorted:?} out of order or twice"
        );
    }
}

#[test]
fn the_entries_near_a_word_come_in_their_own_spelling_with_their_distance() {
    let vocabulary = Vocabulary::new(&[LIST, "kakoune\nkubeadm\nrust\n"]);
    let got: Vec<String> = [
        "kubctl",
        "postgressql",
        "rsut",
        "inuot",
        "kakoume",
        "kubeadmm",
        "zzz",
    ]
    .iter()
    .map(|word| {
        let mut near = Vec::new();
        vocabulary.within(word, 1, &mut near);
        let near: Vec<String> = near
            .iter()
            .map(|(entry, edits)| format!("{entry} {edits}"))
            .collect();
        match near.is_empty() {
            true => format!("{word}: nothing"),
            false => format!("{word}: {}", near.join(", ")),
        }
    })
    .collect();
    expect_test::expect![[r#"
        kubctl: kubectl 1
        postgressql: PostgreSQL 1
        rsut: rust 1
        inuot: INOUT 1
        kakoume: kakoune 1
        kubeadmm: kubeadm 1
        zzz: nothing"#]]
    .assert_eq(&got.join("\n"));
}
