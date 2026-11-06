use expect_test::{Expect, expect};
use orthra_testkit::annotate::Annotation;

use crate::Hosts;

/// The host for `path`, then the spans it finds in `text`, each labeled with its kind.
pub(crate) fn check(path: &str, text: &str, expect: Expect) {
    let hosts = Hosts::builtin();
    let host = hosts
        .for_path(path)
        .unwrap_or_else(|| panic!("no host for {path}"));
    let mut spans = Vec::new();
    host.spans(text, &mut spans);
    let annotations: Vec<Annotation> = spans
        .iter()
        .map(|span| Annotation::new(span.range, format!("{:?}", span.kind).to_lowercase()))
        .collect();
    expect.assert_eq(&format!(
        "{}\n{}",
        host.id(),
        Annotation::render(text, &annotations)
    ));
}

#[test]
fn markdown_prose_is_text_and_inline_code_is_a_value() {
    check(
        "/README.md",
        "# Using the `fooBarBaz` helper\n\nCall `qzxRender()` first; see [the docs](https://example.com/qzx).\n\n```rust\nlet xyzzy = 1;\n```\n\n<div>raw html</div>\n",
        expect![[r#"
            markdown
            > # Using the `fooBarBaz` helper
                ^^^^^^^^^^ prose
                          ^^^^^^^^^^^ value
                                     ^^^^^^^ prose
            >
            > Call `qzxRender()` first; see [the docs](https://example.com/qzx).
              ^^^^^ prose
                   ^^^^^^^^^^^^^ value
                                ^^^^^^^^^^^^ prose
                                             ^^^^^^^^ prose
                                                                               ^ prose
            >
            > ```rust
            > let xyzzy = 1;
            > ```
            >
            > <div>raw html</div>
        "#]],
    );
}

#[test]
fn a_commit_message_without_its_comments_trailers_and_diff() {
    check(
        "/.git/COMMIT_EDITMSG",
        "Fix teh build\n\nThe flag broke.\n\n# Please enter the commit message.\nSigned-off-by: Jane Doe <jane@example.com>\n# ------------------------ >8 ------------------------\ndiff --git a/x b/x\n",
        expect![[r#"
            git-commit
            > Fix teh build
              ^^^^^^^^^^^^^ prose
            >
            > The flag broke.
              ^^^^^^^^^^^^^^^ prose
            >
            > # Please enter the commit message.
            > Signed-off-by: Jane Doe <jane@example.com>
            > # ------------------------ >8 ------------------------
            > diff --git a/x b/x
        "#]],
    );
}

#[test]
fn a_file_no_host_claims_is_not_checked() {
    assert!(Hosts::builtin().for_path("/data/blob.bin").is_none());
    assert_eq!(
        Hosts::builtin().for_path("/notes.TXT").unwrap().id(),
        "plain-text"
    );
    assert_eq!(
        Hosts::builtin().for_language("markdown").unwrap().id(),
        "markdown"
    );
}

#[test]
fn a_generated_file_has_no_host() {
    let hosts = Hosts::builtin();
    let claimed: String = [
        "/web/package.json",
        "/web/package-lock.json",
        "/web/npm-shrinkwrap.json",
        "/web/tsconfig.json",
    ]
    .iter()
    .map(|path| {
        let host = hosts.for_path(path).map_or("-", |host| host.id());
        format!("{host:<4} {path}\n")
    })
    .collect();
    expect![[r#"
        json /web/package.json
        -    /web/package-lock.json
        -    /web/npm-shrinkwrap.json
        json /web/tsconfig.json
    "#]]
    .assert_eq(&claimed);
}
