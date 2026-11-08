//! A translation's strings are in the language its path names. Into a language no enabled pack
//! speaks, they are not checked at all; its keys and comments still are.

use expect_test::expect;
use orthra_testkit::annotate::Annotation;

use super::subject::packs;
use crate::Engine;

/// The findings for `text` at each of `paths`, one rendering per path.
fn check(paths: &[&str], text: &str) -> String {
    let mut engine = Engine::with_packs(packs());
    paths
        .iter()
        .map(|path| {
            let annotations: Vec<Annotation> = engine
                .check_file(path, text)
                .iter()
                .map(|finding| Annotation::new(finding.range, finding.code.0))
                .collect();
            format!("{path}\n{}", Annotation::render(text, &annotations))
        })
        .collect()
}

#[test]
fn a_translation_into_a_language_nobody_enabled_has_unchecked_strings() {
    let text = r#"{
  "einstellungen": {
    "titel": "Einstellungen speichern",
    "hinweis": "Die Änderungen werden sofort übernommen, teh"
  },
  "recieve": "Nachricht empfangen"
}
"#;
    expect![[r#"
        web/src/i18n/de.json
        > {
        >   "einstellungen": {
        >     "titel": "Einstellungen speichern",
               ^^^^^ known-typo
        >     "hinweis": "Die Änderungen werden sofort übernommen, teh"
        >   },
        >   "recieve": "Nachricht empfangen"
             ^^^^^^^ known-typo
        > }
        web/src/i18n/en.json
        > {
        >   "einstellungen": {
        >     "titel": "Einstellungen speichern",
               ^^^^^ known-typo
                                      ^^^^^^^^^ unknown-word
        >     "hinweis": "Die Änderungen werden sofort übernommen, teh"
                              ^^^^^^^^^^ unknown-word
                                         ^^^^^^ unknown-word
                                                ^^^^^^ unknown-word
                                                       ^^^^^^^^^^ unknown-word
                                                                   ^^^ known-typo
        >   },
        >   "recieve": "Nachricht empfangen"
             ^^^^^^^ known-typo
                        ^^^^^^^^^ unknown-word
                                  ^^^^^^^^^ unknown-word
        > }
    "#]]
    .assert_eq(&check(
        &["web/src/i18n/de.json", "web/src/i18n/en.json"],
        text,
    ));
}
