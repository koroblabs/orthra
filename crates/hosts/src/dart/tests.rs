use expect_test::expect;

use crate::tests::check;

#[test]
fn dart_prose_is_comments_strings_and_declared_names() {
    check(
        "/lib/src/greet.dart",
        r##"/// Greets [person] politely.
library greet;

import 'dart:io';
import 'package:foo/foo.dart' as foo;
export 'src/x.dart';
part 'greet.g.dart';

// A plain comment.
typedef Callback = void Function(int event);

mixin Greeter on Object {}
extension StringExt on String { int get size => length; }
enum Color { red, green }

@Deprecated('Use something else')
abstract class Person<T> extends Base with Greeter implements Other {
  final String name;
  int _count = 0;
  static const maxCount = 4;
  Person(this.name, {required this.age});
  Person.named(String nm) : name = nm;
  factory Person.make() => Person('x');

  String get title => name;
  set title(String value) {}

  /// Says hello to [other].
  /// Second line of the doc.
  void greet(String other, [int times = 1]) {
    var message = 'Hello $other, ${name.length} times\n';
    final raw = r'raw\d+ text';
    var multi = '''
      multi line text
    ''';
    print("recieve");
    greetAll(name: 'x');
    for (final item in items) { var inner = item; }
  }

  /** A block doc. */
  int block() => 1;
}

int topLevel(int a) => a;
final topVar = 1;
"##,
        expect![[r#"
            dart
            > /// Greets [person] politely.
                  ^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > library greet;
            >
            > import 'dart:io';
            > import 'package:foo/foo.dart' as foo;
            > export 'src/x.dart';
            > part 'greet.g.dart';
            >
            > // A plain comment.
              ^^^^^^^^^^^^^^^^^^^ comment
            > typedef Callback = void Function(int event);
                      ^^^^^^^^ declaration
                                                   ^^^^^ declaration
            >
            > mixin Greeter on Object {}
                    ^^^^^^^ declaration
            > extension StringExt on String { int get size => length; }
                        ^^^^^^^^^ declaration
                                                      ^^^^ declaration
            > enum Color { red, green }
                   ^^^^^ declaration
                           ^^^ declaration
                                ^^^^^ declaration
            >
            > @Deprecated('Use something else')
            > abstract class Person<T> extends Base with Greeter implements Other {
                             ^^^^^^ declaration
            >   final String name;
                             ^^^^ declaration
            >   int _count = 0;
                     ^^^^^ declaration
            >   static const maxCount = 4;
                             ^^^^^^^^ declaration
            >   Person(this.name, {required this.age});
            >   Person.named(String nm) : name = nm;
                       ^^^^^ declaration
                                    ^^ declaration
            >   factory Person.make() => Person('x');
                               ^^^^ declaration
                                                 ^ string
            >
            >   String get title => name;
                           ^^^^^ declaration
            >   set title(String value) {}
                    ^^^^^ declaration
                                 ^^^^^ declaration
            >
            >   /// Says hello to [other].
                    ^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >   /// Second line of the doc.
                    ^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >   void greet(String other, [int times = 1]) {
                     ^^^^^ declaration
                                  ^^^^^ declaration
                                              ^^^^^ declaration
            >     var message = 'Hello $other, ${name.length} times\n';
                      ^^^^^^^ declaration
                                 ^^^^^ string
                                             ^ string
                                                              ^^^^^ string
            >     final raw = r'raw\d+ text';
                        ^^^ declaration
            >     var multi = '''
                      ^^^^^ declaration
            >       multi line text
                    ^^^^^^^^^^^^^^^ string
            >     ''';
            >     print("recieve");
                         ^^^^^^^ string
            >     greetAll(name: 'x');
                                  ^ string
            >     for (final item in items) { var inner = item; }
                             ^^^^ declaration
                                                  ^^^^^ declaration
            >   }
            >
            >   /** A block doc. */
                    ^^^^^^^^^^^^^^^ doccomment
            >   int block() => 1;
                    ^^^^^ declaration
            > }
            >
            > int topLevel(int a) => a;
                  ^^^^^^^^ declaration
                               ^ declaration
            > final topVar = 1;
                    ^^^^^^ declaration
        "#]],
    );
}

/// A double-quoted string holds an apostrophe as often as a message does, and the grammar ends a
/// piece of its text there. The pieces are one span again, so `doesn't` is one word.
#[test]
fn an_apostrophe_in_a_double_quoted_string_is_inside_a_word() {
    check(
        "/lib/greet.dart",
        "void f() { a(\"Queue doesn't belong\"); b('say \"hi\"'); }\n",
        expect![[r#"
            dart
            > void f() { a("Queue doesn't belong"); b('say "hi"'); }
                   ^ declaration
                            ^^^^^^^^^^^^^^^^^^^^ string
                                                       ^^^ string
                                                            ^^ string
        "#]],
    );
}
