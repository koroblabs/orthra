use expect_test::expect;

use crate::tests::check;

#[test]
fn a_maven_value_is_held_to_known_typos_and_a_sentence_is_prose() {
    check(
        "/pom.xml",
        r#"<?xml version="1.0"?>
<!-- Seperates the build. -->
<project xmlns="http://maven.apache.org/POM/4.0.0">
  <artifactId>maven-compiler-plugin</artifactId>
  <version>3.11.0</version>
  <name>Apache Maven Compiler Plugin</name>
  <description>Compiles the sources. Fish &amp; chips.</description>
  <mixed>Use <b>this</b> one</mixed>
  <script><![CDATA[ var recieve = 1; ]]></script>
</project>
"#,
        expect![[r#"
            xml
            > <?xml version="1.0"?>
            > <!-- Seperates the build. -->
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > <project xmlns="http://maven.apache.org/POM/4.0.0">
            >   <artifactId>maven-compiler-plugin</artifactId>
                            ^^^^^^^^^^^^^^^^^^^^^ value
            >   <version>3.11.0</version>
                         ^^^^^^ value
            >   <name>Apache Maven Compiler Plugin</name>
                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ prose
            >   <description>Compiles the sources. Fish &amp; chips.</description>
                             ^^^^^^^^^^^^^^^^^^^^^^^^^^ prose
                                                              ^^^^^^ prose
            >   <mixed>Use <b>this</b> one</mixed>
                       ^^^ prose
                              ^^^^ value
                                       ^^^ prose
            >   <script><![CDATA[ var recieve = 1; ]]></script>
            > </project>
        "#]],
    );
}

#[test]
fn an_attribute_is_text_only_where_it_names_text_for_a_reader() {
    check(
        "/res/layout/main.xml",
        r#"<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
  <TextView android:id="@+id/greeting" android:text="Hello wrold" android:hint="Type here" />
  <Button android:contentDescription="Sends the form" android:onClick="send" />
</LinearLayout>
"#,
        expect![[r#"
            xml
            > <LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
            >   <TextView android:id="@+id/greeting" android:text="Hello wrold" android:hint="Type here" />
                                                                  ^^^^^^^^^^^^^ string
                                                                                             ^^^^^^^^^^^ string
            >   <Button android:contentDescription="Sends the form" android:onClick="send" />
                                                   ^^^^^^^^^^^^^^^^ string
            > </LinearLayout>
        "#]],
    );
}

#[test]
fn an_android_string_keeps_its_placeholders_out_of_its_words() {
    check(
        "/res/values/strings.xml",
        r#"<resources>
  <string name="app_name">Orthra</string>
  <string name="greeting">Hello <xliff:g id="name">%1$s</xliff:g>,\nwelcome back</string>
</resources>
"#,
        expect![[r#"
            xml
            > <resources>
            >   <string name="app_name">Orthra</string>
                                        ^^^^^^ value
            >   <string name="greeting">Hello <xliff:g id="name">%1$s</xliff:g>,\nwelcome back</string>
                                        ^^^^^ prose
                                                                               ^^^^^^^^^^^^^^^ prose
            > </resources>
        "#]],
    );
}

#[test]
fn a_project_file_is_checked_as_xml() {
    check(
        "/src/App.csproj",
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <Description>A library that parses things</Description>
  </PropertyGroup>
</Project>
"#,
        expect![[r#"
            xml
            > <Project Sdk="Microsoft.NET.Sdk">
            >   <PropertyGroup>
            >     <TargetFramework>net8.0</TargetFramework>
                                   ^^^^^^ value
            >     <Description>A library that parses things</Description>
                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ prose
            >   </PropertyGroup>
            > </Project>
        "#]],
    );
}
