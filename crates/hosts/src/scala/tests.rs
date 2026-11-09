use expect_test::expect;

use crate::tests::check;

#[test]
fn scala_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/main/scala/Greeter.scala",
        r#"package com.example.greeting

import scala.collection.mutable.{Map => Items}

/** Greets whoever asks.
  * @param name who to greet
  * @tparam T the kind
  * @return the greeting
  * @see [[Items]]
  */
@deprecated("old way", "1.0")
final case class Greeter(name: String, count: Int = DEFAULT) extends Base {
  // Says hello.
  val message: String = s"Hello\n$name and ${count + 1} there"
  var total = 0
  lazy val coordinate = "org.typelevel"
  def shout(loud: Boolean)(implicit ctx: Ctx): String = {
    val (first, second) = pair
    val raw = """raw text"""
    list.map { item => item }
    for (x <- xs) yield x
    println(message)
    "plain words"
  }
  type Alias = Int
}
object Single extends App
trait Tr { def f: Int }
enum Color { case Red, Green }
given intOrd: Ordering[Int] = ???
"#,
        expect![[r#"
            scala
            > package com.example.greeting
            >
            > import scala.collection.mutable.{Map => Items}
            >
            > /** Greets whoever asks.
                  ^^^^^^^^^^^^^^^^^^^^ doccomment
            >   * @param name who to greet
                              ^^^^^^^^^^^^ doccomment
            >   * @tparam T the kind
                            ^^^^^^^^ doccomment
            >   * @return the greeting
                          ^^^^^^^^^^^^ doccomment
            >   * @see [[Items]]
            >   */
            > @deprecated("old way", "1.0")
            > final case class Greeter(name: String, count: Int = DEFAULT) extends Base {
                               ^^^^^^^ declaration
                                       ^^^^ declaration
                                                     ^^^^^ declaration
            >   // Says hello.
                ^^^^^^^^^^^^^^ comment
            >   val message: String = s"Hello\n$name and ${count + 1} there"
                    ^^^^^^^ declaration
                                       ^^^^^^^^^^^^^^^^^ string
                                                                      ^^^^^^ string
            >   var total = 0
                    ^^^^^ declaration
            >   lazy val coordinate = "org.typelevel"
                         ^^^^^^^^^^ declaration
                                      ^^^^^^^^^^^^^^^ value
            >   def shout(loud: Boolean)(implicit ctx: Ctx): String = {
                    ^^^^^ declaration
                          ^^^^ declaration
                                                  ^^^ declaration
            >     val (first, second) = pair
                       ^^^^^ declaration
                              ^^^^^^ declaration
            >     val raw = """raw text"""
                      ^^^ declaration
                            ^^^^^^^^^^^^^^ string
            >     list.map { item => item }
                             ^^^^ declaration
            >     for (x <- xs) yield x
                       ^ declaration
            >     println(message)
            >     "plain words"
                  ^^^^^^^^^^^^^ string
            >   }
            >   type Alias = Int
                     ^^^^^ declaration
            > }
            > object Single extends App
                     ^^^^^^ declaration
            > trait Tr { def f: Int }
                    ^^ declaration
                             ^ declaration
            > enum Color { case Red, Green }
                   ^^^^^ declaration
                                ^^^ declaration
                                     ^^^^^ declaration
            > given intOrd: Ordering[Int] = ???
                    ^^^^^^ declaration
        "#]],
    );
}

/// Scaladoc's code is between `{{{` and `}}}`, and a REPL session's `scala>` line and its result
/// run to the next blank line.
#[test]
fn scaladoc_code_blocks_and_repl_lines_are_code() {
    check(
        "/src/main/scala/Greet.scala",
        r#"/** Greets a person.
  *
  * {{{
  * val qzxv = greet("wrtyp")
  * }}}
  *
  * scala> greet("zyxw")
  * res0: String = Hello, zyxw
  *
  * Returns the greeting.
  */
def greet(name: String): String = name
"#,
        expect![[r#"
            scala
            > /** Greets a person.
                  ^^^^^^^^^^^^^^^^ doccomment
            >   *
            >   * {{{
            >   * val qzxv = greet("wrtyp")
            >   * }}}
            >   *
            >   * scala> greet("zyxw")
            >   * res0: String = Hello, zyxw
            >   *
            >   * Returns the greeting.
                  ^^^^^^^^^^^^^^^^^^^^^ doccomment
            >   */
            > def greet(name: String): String = name
                  ^^^^^ declaration
                        ^^^^ declaration
        "#]],
    );
}

/// What the first test does not write: an enum's full cases, abstract members, a lambda's typed
/// parameter.
#[test]
fn scala_declares_cases_members_and_parameters() {
    crate::tests::check(
        "/src/Shapes.scala",
        r#"enum QzxvShape { case QzxvCircle(qzxvradius: Double); case QzxvSquare(qzxvside: Double) }
trait QzxvSized { val qzxvsize: Int; var qzxvlabel: String; def qzxvarea(qzxvscale: Int): Int }
val qzxvdouble = (qzxvn: Int) => qzxvn * 2
"#,
        expect_test::expect![[r#"
            scala
            > enum QzxvShape { case QzxvCircle(qzxvradius: Double); case QzxvSquare(qzxvside: Double) }
                   ^^^^^^^^^ declaration
                                    ^^^^^^^^^^ declaration
                                               ^^^^^^^^^^ declaration
                                                                         ^^^^^^^^^^ declaration
                                                                                    ^^^^^^^^ declaration
            > trait QzxvSized { val qzxvsize: Int; var qzxvlabel: String; def qzxvarea(qzxvscale: Int): Int }
                    ^^^^^^^^^ declaration
                                    ^^^^^^^^ declaration
                                                       ^^^^^^^^^ declaration
                                                                              ^^^^^^^^ declaration
                                                                                       ^^^^^^^^^ declaration
            > val qzxvdouble = (qzxvn: Int) => qzxvn * 2
                  ^^^^^^^^^^ declaration
        "#]],
    );
}

/// A name in any script is declared, as a plain ASCII one is.
#[test]
fn scala_declares_names_in_any_script() {
    crate::tests::check(
        "/src/Names.scala",
        "object Привіт {\n  val résumé = 1\n  def наЗміну(значення: Int): Int = значення\n  val подвоїти = (θ: Int) => θ * 2\n}\n",
        expect_test::expect![[r#"
            scala
            > object Привіт {
                     ^^^^^^ declaration
            >   val résumé = 1
                    ^^^^^^ declaration
            >   def наЗміну(значення: Int): Int = значення
                    ^^^^^^^ declaration
                            ^^^^^^^^ declaration
            >   val подвоїти = (θ: Int) => θ * 2
                    ^^^^^^^^ declaration
            > }
        "#]],
    );
}
