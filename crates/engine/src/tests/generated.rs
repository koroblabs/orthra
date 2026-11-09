//! A file a tool writes has no findings, opened in an editor as on disk: the editor's language id
//! names the host, and the host's generated names still leave the file alone. So does a line no
//! person writes, whatever the file's name.

use expect_test::expect;

use super::subject::packs;
use crate::{Engine, Parsed};

#[test]
fn an_open_minified_bundle_has_no_findings() {
    let text = "/* Seperates teh greeting. */var a=\"recieve\";\n";
    let mut engine = Engine::with_packs(packs());
    let found: Vec<String> = ["src/app.js", "dist/app.min.js"]
        .iter()
        .map(|path| {
            let findings = engine.check_document(path, "javascript", text, &mut Parsed::default());
            let shown: Vec<String> = findings
                .iter()
                .map(|finding| format!("{} {}", &text[finding.range], finding.code.0))
                .collect();
            format!("{path}: {shown:?}")
        })
        .collect();
    expect![[r#"
        src/app.js: ["Seperates known-typo", "teh known-typo", "recieve known-typo"]
        dist/app.min.js: []
    "#]]
    .assert_eq(&(found.join("\n") + "\n"));
}

/// A bundle is one line, under any name. A line of 10 000 characters is still somebody's; one of
/// 10 001 is not. It is counted in characters, not in bytes.
#[test]
fn a_line_no_person_writes_is_a_tools_output() {
    let mut engine = Engine::with_packs(packs());
    let line = |chars: usize, fill: &str| {
        let head = "/* Seperates teh greeting. */";
        let mut line = head.to_owned();
        while line.chars().count() < chars {
            line.push_str(fill);
        }
        line.truncate(
            line.char_indices()
                .nth(chars)
                .map_or(line.len(), |(at, _)| at),
        );
        line + "\n"
    };
    let found: Vec<String> = [
        ("at the limit", line(10_000, " a")),
        ("one past it", line(10_001, " a")),
        (
            "over it in bytes, under it in letters of two",
            line(6_000, " ж"),
        ),
        (
            "one past it, on a line of its own",
            format!("var a = 1; // recieve\n{}", line(10_001, " a")),
        ),
    ]
    .iter()
    .map(|(what, text)| {
        let findings =
            engine.check_document("dist/bundle.js", "javascript", text, &mut Parsed::default());
        format!("{what}: {}", findings.len())
    })
    .collect();
    expect![[r#"
        at the limit: 2
        one past it: 0
        over it in bytes, under it in letters of two: 2
        one past it, on a line of its own: 0
    "#]]
    .assert_eq(&(found.join("\n") + "\n"));
}
