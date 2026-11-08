use expect_test::expect;

use crate::tests::check;

#[test]
fn swift_prose_is_comments_strings_and_declared_names() {
    check(
        "/Sources/Greeter.swift",
        r##"import Foundation

/// Greets whoever asks.
///
/// - Parameter person: Who to greet.
/// - Returns: The greeting.
///
/// Usage:
///
///     let greeter = Greeter(label: "Wrld", extra: 1)
@available(*, deprecated, message: "Use greetPerson")
public struct Greeter<Element>: Sendable {
    // A plain comment.
    public var count: Int = 0
    let (left, right) = (1, 2)

    /** Builds one. */
    public init(label name: String, extra: Int) {}

    func greet(person: String) -> String {
        let message = "Hello\nworld, \(person) here"
        let raw = #"raw text"#
        let block = """
            Multi line
            text \(message)
            """
        print(message.count, Greeter<Int>.self)
        return message + raw + block
    }
}

enum Mode { case fast, slow(Int) }
typealias Alias = Int
protocol Driver { associatedtype Wheel; var speed: Int { get }; func drive(now: Bool) }
extension Greeter { var shout: String { "" } }
"##,
        expect![[r##"
            swift
            > import Foundation
            >
            > /// Greets whoever asks.
                  ^^^^^^^^^^^^^^^^^^^^ doccomment
            > ///
            > /// - Parameter person: Who to greet.
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > /// - Returns: The greeting.
                  ^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > ///
            > /// Usage:
                  ^^^^^^ doccomment
            > ///
            > ///     let greeter = Greeter(label: "Wrld", extra: 1)
            > @available(*, deprecated, message: "Use greetPerson")
            > public struct Greeter<Element>: Sendable {
                            ^^^^^^^ declaration
                                    ^^^^^^^ declaration
            >     // A plain comment.
                  ^^^^^^^^^^^^^^^^^^^ comment
            >     public var count: Int = 0
                             ^^^^^ declaration
            >     let (left, right) = (1, 2)
            >
            >     /** Builds one. */
                      ^^^^^^^^^^^^^^ doccomment
            >     public init(label name: String, extra: Int) {}
                              ^^^^^ declaration
                                    ^^^^ declaration
                                                  ^^^^^ declaration
            >
            >     func greet(person: String) -> String {
                       ^^^^^ declaration
                             ^^^^^^ declaration
            >         let message = "Hello\nworld, \(person) here"
                          ^^^^^^^ declaration
                                     ^^^^^ string
                                            ^^^^^^ string
                                                             ^^^^ string
            >         let raw = #"raw text"#
                          ^^^ declaration
                                ^^^^^^^^^^^^ string
            >         let block = """
                          ^^^^^ declaration
            >             Multi line
                          ^^^^^^^^^^ string (+1 lines)
            >             text \(message)
            >             """
            >         print(message.count, Greeter<Int>.self)
            >         return message + raw + block
            >     }
            > }
            >
            > enum Mode { case fast, slow(Int) }
                   ^^^^ declaration
                               ^^^^ declaration
                                     ^^^^ declaration
            > typealias Alias = Int
                        ^^^^^ declaration
            > protocol Driver { associatedtype Wheel; var speed: Int { get }; func drive(now: Bool) }
                       ^^^^^^ declaration
                                               ^^^^^ declaration
                                                          ^^^^^ declaration
                                                                                   ^^^^^ declaration
                                                                                         ^^^ declaration
            > extension Greeter { var shout: String { "" } }
                                      ^^^^^ declaration
        "##]],
    );
}
