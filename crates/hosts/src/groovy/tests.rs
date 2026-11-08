use expect_test::expect;

use crate::tests::check;

#[test]
fn groovy_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/main/groovy/Greeter.groovy",
        r#"package com.example

import groovy.transform.Field

/**
 * Greets whoever asks.
 * @param name who to greet
 * @return the greeting
 * @see Items
 */
@Field
class Greeter extends Base {
    String name
    def count = 1
    // Says hello.
    String shout(boolean loud, String suffix = 'x') {
        def message = "Hello\n$name and ${count + 1} there"
        def raw = '''raw text'''
        String typed = 'two words'
        println message
        list.each { item -> item }
        for (x in xs) { x }
        return 'plain words'
    }
}
interface Tr { int f() }
enum Color { RED, GREEN }
"#,
        expect![[r#"
            groovy
            > package com.example
            >
            > import groovy.transform.Field
            >
            > /**
            >  * Greets whoever asks.
                 ^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @param name who to greet
                             ^^^^^^^^^^^^ doccomment
            >  * @return the greeting
                         ^^^^^^^^^^^^ doccomment
            >  * @see Items
            >  */
            > @Field
            > class Greeter extends Base {
                    ^^^^^^^ declaration
            >     String name
                         ^^^^ declaration
            >     def count = 1
                      ^^^^^ declaration
            >     // Says hello.
                  ^^^^^^^^^^^^^^ comment
            >     String shout(boolean loud, String suffix = 'x') {
                         ^^^^^ declaration
                                       ^^^^ declaration
                                                    ^^^^^^ declaration
                                                             ^^^ value
            >         def message = "Hello\n$name and ${count + 1} there"
                          ^^^^^^^ declaration
                                     ^^^^^ value
                                            ^^^^^^^^^ string
                                                                   ^^^^^ string
            >         def raw = '''raw text'''
                          ^^^ declaration
                                   ^^^^^^^^ string
            >         String typed = 'two words'
                             ^^^^^ declaration
                                     ^^^^^^^^^^^ string
            >         println message
            >         list.each { item -> item }
                                  ^^^^ declaration
            >         for (x in xs) { x }
                           ^ declaration
            >         return 'plain words'
                             ^^^^^^^^^^^^^ string
            >     }
            > }
            > interface Tr { int f() }
                        ^^ declaration
                                 ^ declaration
            > enum Color { RED, GREEN }
                   ^^^^^ declaration
                           ^^^ declaration
                                ^^^^^ declaration
        "#]],
    );
}

#[test]
fn a_gradle_build_script_is_coordinates_and_plugin_ids() {
    check(
        "/build.gradle",
        r#"plugins {
    id 'java-library'
    id "org.jetbrains.kotlin.jvm" version "1.9.0"
}
dependencies {
    implementation 'com.squareup.okio:okio:3.0.0'
    testImplementation("junit:junit:4.13")
}
tasks.register('hello') { doLast { println "hi there" } }
"#,
        expect![[r#"
            groovy
            > plugins {
            >     id 'java-library'
                     ^^^^^^^^^^^^^^ value
            >     id "org.jetbrains.kotlin.jvm" version "1.9.0"
                      ^^^^^^^^^^^^^^^^^^^^^^^^ value
                                                         ^^^^^ value
            > }
            > dependencies {
            >     implementation 'com.squareup.okio:okio:3.0.0'
                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ value
            >     testImplementation("junit:junit:4.13")
                                      ^^^^^^^^^^^^^^^^ value
            > }
            > tasks.register('hello') { doLast { println "hi there" } }
                             ^^^^^^^ value
                                                          ^^^^^^^^ string
        "#]],
    );
}
