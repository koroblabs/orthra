use expect_test::{Expect, expect};
use orthra_testkit::annotate::Annotation;

use crate::Engine;

fn check(path: &str, text: &str, expect: Expect) {
    let mut engine = Engine::new(&["en-US"]).unwrap();
    let findings = engine.check_file(path, text);
    let annotations: Vec<Annotation> = findings
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect.assert_eq(&Annotation::render(text, &annotations));
}

#[test]
fn a_comment_line_labeled_with_people_is_names() {
    check(
        "/themes/dusk.toml",
        r#"# Author: Matous Dzivjak <matousdzivjak@gmail.com>
# Original author : rebelot
# Maintainers: Qzxv Wrtyp, Jens Getreu
#   Authors: listed below, Qzxva
# Thanks to Qzxvb for the palette.
# Note: qzxvc is still a finding.
name = "Author: Qzxvd"
"#,
        expect![[r#"
            > # Author: Matous Dzivjak <matousdzivjak@gmail.com>
            > # Original author : rebelot
            > # Maintainers: Qzxv Wrtyp, Jens Getreu
            > #   Authors: listed below, Qzxva
            > # Thanks to Qzxvb for the palette.
                          ^^^^^ unknown-word
            > # Note: qzxvc is still a finding.
                      ^^^^^ unknown-word
            > name = "Author: Qzxvd"
                              ^^^^^ unknown-word
        "#]],
    );
}
