use expect_test::expect;

use super::named_language;

#[test]
fn a_translation_is_named_by_its_path() {
    let paths = [
        "web/src/i18n/de.json",
        "config/locales/devise.de.yml",
        "config/locales/en.yml",
        "public/locales/pt-BR/common.json",
        "extension/_locales/zh_CN/messages.json",
        "resources/lang/es/validation.php",
        "messages/fr.json",
        "src/i18n/index.ts",
        "lang/api.json",
        "src/de.json",
        "docs/de/README.md",
        "locales/sr-Latn/a.json",
        "i18n",
    ];
    let named: Vec<String> = paths
        .iter()
        .map(|path| format!("{:<9}{path}", named_language(path).unwrap_or("-")))
        .collect();
    expect![[r#"
        de       web/src/i18n/de.json
        de       config/locales/devise.de.yml
        en       config/locales/en.yml
        pt       public/locales/pt-BR/common.json
        zh       extension/_locales/zh_CN/messages.json
        es       resources/lang/es/validation.php
        fr       messages/fr.json
        -        src/i18n/index.ts
        -        lang/api.json
        -        src/de.json
        -        docs/de/README.md
        sr       locales/sr-Latn/a.json
        -        i18n
    "#]]
    .assert_eq(&(named.join("\n") + "\n"));
}
