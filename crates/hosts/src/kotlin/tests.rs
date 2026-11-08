use expect_test::expect;

use crate::tests::check;

#[test]
fn kotlin_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/main/kotlin/Greeter.kt",
        r#"package com.example.greeting

import kotlin.collections.List as Items
import org.example.*

/**
 * Greets whoever asks.
 * @param name who to greet
 * @property count how often
 * @author Vladislav Kisel
 * @see [Items]
 */
@Suppress("unused")
class Greeter(val name: String, private var count: Int = DEFAULT) : Base(name) {
    companion object Factory {
        const val MAX = 1
    }

    // Says hello.
    fun String.shout(loud: Boolean): String {
        val (first, second) = pair
        val message = "Hello\n$name, ${first + 1} there"
        val raw = """raw $second text"""
        val coordinate = "com.squareup.okio"
        val handler = { item, (key, value) -> item }
        try {
            println(message + raw)
        } catch (failure: Exception) {
            throw failure
        }
        return handler.toString()
    }

    enum class Color { RED, GREEN }
}

object Single

typealias Handler = (Int) -> Unit
"#,
        expect![[r#"
            kotlin
            > package com.example.greeting
            >
            > import kotlin.collections.List as Items
            > import org.example.*
            >
            > /**
            >  * Greets whoever asks.
                 ^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @param name who to greet
                             ^^^^^^^^^^^^ doccomment
            >  * @property count how often
                                 ^^^^^^^^^ doccomment
            >  * @author Vladislav Kisel
            >  * @see [Items]
            >  */
            > @Suppress("unused")
            > class Greeter(val name: String, private var count: Int = DEFAULT) : Base(name) {
                    ^^^^^^^ declaration
                                ^^^^ declaration
                                                          ^^^^^ declaration
            >     companion object Factory {
                                   ^^^^^^^ declaration
            >         const val MAX = 1
                                ^^^ declaration
            >     }
            >
            >     // Says hello.
                  ^^^^^^^^^^^^^^ comment
            >     fun String.shout(loud: Boolean): String {
                             ^^^^^ declaration
                                   ^^^^ declaration
            >         val (first, second) = pair
                           ^^^^^ declaration
                                  ^^^^^^ declaration
            >         val message = "Hello\n$name, ${first + 1} there"
                          ^^^^^^^ declaration
                                     ^^^^^ value
                                            ^ value
                                             ^^^^^ value
                                                                ^^^^^ value
            >         val raw = """raw $second text"""
                          ^^^ declaration
                                   ^^^ value
                                               ^^^^ value
            >         val coordinate = "com.squareup.okio"
                          ^^^^^^^^^^ declaration
                                        ^^^^^^^^^^^^^^^^^ value
            >         val handler = { item, (key, value) -> item }
                          ^^^^^^^ declaration
                                      ^^^^ declaration
                                             ^^^ declaration
                                                  ^^^^^ declaration
            >         try {
            >             println(message + raw)
            >         } catch (failure: Exception) {
                               ^^^^^^^ declaration
            >             throw failure
            >         }
            >         return handler.toString()
            >     }
            >
            >     enum class Color { RED, GREEN }
                             ^^^^^ declaration
                                     ^^^ declaration
                                          ^^^^^ declaration
            > }
            >
            > object Single
                     ^^^^^^ declaration
            >
            > typealias Handler = (Int) -> Unit
                        ^^^^^^^ declaration
        "#]],
    );
}
