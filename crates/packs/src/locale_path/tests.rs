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
        "app/src/main/res/values-es/strings.xml",
        "app/src/main/res/values-pt-rBR/strings.xml",
        "app/src/main/res/values-b+sr+Latn/strings.xml",
        "app/src/main/res/values/strings.xml",
        "app/src/main/res/values-night/colors.xml",
        "app/src/main/res/values-v21/styles.xml",
        "app/src/main/res/values-in/strings.xml",
        "app/src/main/res/values-ast/strings.xml",
        "app/src/main/res/values-car/dimens.xml",
        "Sources/Base.lproj/Main.storyboard",
        "Sources/de.lproj/Localizable.strings",
        "src/Properties/Strings.fr.resx",
        "src/Properties/Strings.resx",
        "src/Forms/Main.Designer.resx",
        "src/main/resources/messages_de.properties",
        "src/main/resources/l10n/JabRef_pt_BR.properties",
        "src/main/resources/messages.properties",
        "src/main/resources/my_app_messages.properties",
        "src/main/resources/application.properties",
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
        es       app/src/main/res/values-es/strings.xml
        pt       app/src/main/res/values-pt-rBR/strings.xml
        sr       app/src/main/res/values-b+sr+Latn/strings.xml
        -        app/src/main/res/values/strings.xml
        -        app/src/main/res/values-night/colors.xml
        -        app/src/main/res/values-v21/styles.xml
        in       app/src/main/res/values-in/strings.xml
        ast      app/src/main/res/values-ast/strings.xml
        -        app/src/main/res/values-car/dimens.xml
        -        Sources/Base.lproj/Main.storyboard
        de       Sources/de.lproj/Localizable.strings
        fr       src/Properties/Strings.fr.resx
        -        src/Properties/Strings.resx
        -        src/Forms/Main.Designer.resx
        de       src/main/resources/messages_de.properties
        pt       src/main/resources/l10n/JabRef_pt_BR.properties
        -        src/main/resources/messages.properties
        -        src/main/resources/my_app_messages.properties
        -        src/main/resources/application.properties
    "#]]
    .assert_eq(&(named.join("\n") + "\n"));
}
