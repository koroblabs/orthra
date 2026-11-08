//! A file a tool writes has no findings, opened in an editor as on disk: the editor's language id
//! names the host, and the host's generated names still leave the file alone.

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
