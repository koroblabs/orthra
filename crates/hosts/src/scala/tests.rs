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
