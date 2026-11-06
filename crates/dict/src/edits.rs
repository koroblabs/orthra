//! How far a correction is from the word written.

/// The edits between `a` and `b`, counted in characters with case ignored: an insertion, a
/// deletion, a substitution or two neighbors swapped is one edit each (optimal string alignment).
/// `teh` is one edit from `the`, `Сюда` one from `сюди`.
pub fn between(a: &str, b: &str) -> u32 {
    let a: Vec<char> = a.chars().flat_map(char::to_lowercase).collect();
    let b: Vec<char> = b.chars().flat_map(char::to_lowercase).collect();
    // Three rows of the table: the one before the previous is what a swap reaches back to.
    let width = b.len() + 1;
    let mut before: Vec<u32> = vec![0; width];
    let mut previous: Vec<u32> = (0..).take(width).collect();
    let mut current: Vec<u32> = vec![0; width];
    for i in 1..=a.len() {
        current[0] = previous[0] + 1;
        for j in 1..width {
            let substitution = previous[j - 1] + u32::from(a[i - 1] != b[j - 1]);
            let mut edits = substitution.min(previous[j] + 1).min(current[j - 1] + 1);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                edits = edits.min(before[j - 2] + 1);
            }
            current[j] = edits;
        }
        std::mem::swap(&mut before, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    previous[width - 1]
}

/// [`between`] for characters already in one case, when only a distance of `most` or less
/// matters: `None` beyond it, found as soon as a row of the table is all beyond it. `rows` is
/// scratch the caller keeps, so that a search over many words allocates once.
pub fn within(a: &[char], b: &[char], most: u32, rows: &mut Vec<u32>) -> Option<u32> {
    if a.len().abs_diff(b.len()) > most as usize {
        return None;
    }
    let width = b.len() + 1;
    rows.clear();
    rows.resize(3 * width, 0);
    let (before, rest) = rows.split_at_mut(width);
    let (previous, current) = rest.split_at_mut(width);
    let (mut before, mut previous, mut current) = (before, previous, current);
    for (j, cell) in previous.iter_mut().enumerate() {
        *cell = u32::try_from(j).unwrap_or(u32::MAX);
    }
    for i in 1..=a.len() {
        current[0] = previous[0] + 1;
        let mut best = current[0];
        for j in 1..width {
            let mut edits = (previous[j - 1] + u32::from(a[i - 1] != b[j - 1]))
                .min(previous[j] + 1)
                .min(current[j - 1] + 1);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                edits = edits.min(before[j - 2] + 1);
            }
            current[j] = edits;
            best = best.min(edits);
        }
        if best > most {
            return None;
        }
        std::mem::swap(&mut before, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    Some(previous[width - 1]).filter(|edits| *edits <= most)
}

#[cfg(test)]
mod tests {
    use super::between;

    #[test]
    fn each_kind_of_edit_is_one_and_case_is_none() {
        let pairs = [
            ("the", "the"),
            ("teh", "the"),
            ("th", "the"),
            ("thee", "the"),
            ("tha", "the"),
            ("Сюда", "сюди"),
            ("очевидцьами", "очевидцями"),
            ("дівідентів", "дивідендів"),
            ("", "abc"),
            ("abc", ""),
            ("ca", "abc"),
        ];
        let got: Vec<String> = pairs
            .iter()
            .map(|(a, b)| format!("{a} {b} {}", between(a, b)))
            .collect();
        expect_test::expect![[r#"
            the the 0
            teh the 1
            th the 1
            thee the 1
            tha the 1
            Сюда сюди 1
            очевидцьами очевидцями 2
            дівідентів дивідендів 2
             abc 3
            abc  3
            ca abc 3"#]]
        .assert_eq(&got.join("\n"));
    }

    #[test]
    fn within_agrees_with_between_up_to_its_bound() {
        let words = [
            "the",
            "teh",
            "th",
            "thee",
            "codebase",
            "clodebase",
            "codebases",
            "kubectl",
            "kubctl",
            "abc",
            "ca",
            "",
            "сюда",
            "сюди",
        ];
        let mut rows = Vec::new();
        for a in words {
            for b in words {
                let full = between(a, b);
                let (a_chars, b_chars): (Vec<char>, Vec<char>) =
                    (a.chars().collect(), b.chars().collect());
                for most in 0..3 {
                    let bounded = super::within(&a_chars, &b_chars, most, &mut rows);
                    assert_eq!(
                        bounded,
                        Some(full).filter(|e| *e <= most),
                        "{a} {b} within {most}"
                    );
                }
            }
        }
    }
}
