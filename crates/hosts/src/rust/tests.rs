use expect_test::expect;

use crate::tests::check;

#[test]
fn rust_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/greet.rs",
        r#"//! Greeting helpers.
use serde_json::from_str;

/// Says hello to whoever asks.
pub struct Greeter {
    /// Who to greet.
    name: String,
    count: usize,
}

// Builds the greeting.
/* A block comment. */
pub fn greet(person: &str) -> String {
    let message = "Hello\nworld";
    let pattern = r"raw text";
    shout(person, message, pattern)
}
"#,
        expect![[r#"
            rust
            > //! Greeting helpers.
                  ^^^^^^^^^^^^^^^^^ doccomment
            > use serde_json::from_str;
            >
            > /// Says hello to whoever asks.
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > pub struct Greeter {
                         ^^^^^^^ declaration
            >     /// Who to greet.
                      ^^^^^^^^^^^^^ doccomment
            >     name: String,
                  ^^^^ declaration
            >     count: usize,
                  ^^^^^ declaration
            > }
            >
            > // Builds the greeting.
              ^^^^^^^^^^^^^^^^^^^^^^^ comment
            > /* A block comment. */
              ^^^^^^^^^^^^^^^^^^^^^^ comment
            > pub fn greet(person: &str) -> String {
                     ^^^^^ declaration
                           ^^^^^^ declaration
            >     let message = "Hello\nworld";
                      ^^^^^^^ declaration
                                 ^^^^^ string
                                        ^^^^^ string
            >     let pattern = r"raw text";
                      ^^^^^^^ declaration
                                  ^^^^^^^^ string
            >     shout(person, message, pattern)
            > }
        "#]],
    );
}

#[test]
fn rust_declares_with_every_item_kind() {
    check(
        "/src/paint.rs",
        r#"mod storage;
pub enum Colour { Crimson, Teal(u8) }
pub trait Painter {
    type Brush;
    fn paint(&self, canvas: &mut Canvas);
}
type Palette = Vec<Colour>;
const MAXIMUM: usize = 8;
static mut COUNTER: u32 = 0;
macro_rules! twice { ($x:expr) => { $x + $x }; }
fn mix(mut first: u8) {
    let mut blend = first;
    let scale = |amount| amount * blend;
    Colour::Teal(scale(first));
}
"#,
        expect![[r#"
            rust
            > mod storage;
                  ^^^^^^^ declaration
            > pub enum Colour { Crimson, Teal(u8) }
                       ^^^^^^ declaration
                                ^^^^^^^ declaration
                                         ^^^^ declaration
            > pub trait Painter {
                        ^^^^^^^ declaration
            >     type Brush;
                       ^^^^^ declaration
            >     fn paint(&self, canvas: &mut Canvas);
                     ^^^^^ declaration
                                  ^^^^^^ declaration
            > }
            > type Palette = Vec<Colour>;
                   ^^^^^^^ declaration
            > const MAXIMUM: usize = 8;
                    ^^^^^^^ declaration
            > static mut COUNTER: u32 = 0;
                         ^^^^^^^ declaration
            > macro_rules! twice { ($x:expr) => { $x + $x }; }
                           ^^^^^ declaration
            > fn mix(mut first: u8) {
                 ^^^ declaration
                         ^^^^^ declaration
            >     let mut blend = first;
                          ^^^^^ declaration
            >     let scale = |amount| amount * blend;
                      ^^^^^ declaration
                               ^^^^^^ declaration
            >     Colour::Teal(scale(first));
            > }
        "#]],
    );
}

#[test]
fn test_code_and_doc_examples_are_not_prose() {
    crate::tests::check(
        "/src/glue.rs",
        r#"/// Reads a line. For example:
///
/// ```
/// let watsons = "Watsons and Holmeses";
/// ```
///
/// Then returns it.
pub fn read() -> &'static str { "a real message" }

#[cfg(test)]
mod tests {
    const SHERLOCK: &str = "For the Doctor Watsons of this world";

    fn helper() -> &'static str { "doctah" }
}

#[test]
fn reads() {
    assert_eq!(super::read(), "holmeses");
}
"#,
        expect_test::expect![[r#"
            rust
            > /// Reads a line. For example:
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > ///
            > /// ```
            > /// let watsons = "Watsons and Holmeses";
            > /// ```
            > ///
            > /// Then returns it.
                  ^^^^^^^^^^^^^^^^ doccomment
            > pub fn read() -> &'static str { "a real message" }
                     ^^^^ declaration
                                               ^^^^^^^^^^^^^^ string
            >
            > #[cfg(test)]
            > mod tests {
                  ^^^^^ declaration
            >     const SHERLOCK: &str = "For the Doctor Watsons of this world";
                        ^^^^^^^^ declaration
            >
            >     fn helper() -> &'static str { "doctah" }
                     ^^^^^^ declaration
            > }
            >
            > #[test]
            > fn reads() {
                 ^^^^^ declaration
            >     assert_eq!(super::read(), "holmeses");
            > }
        "#]],
    );
}

/// A `cfg` predicate's strings name targets and features: values, never text. The attribute's
/// item is still read.
#[test]
fn a_cfg_predicate_is_not_prose() {
    check(
        "/src/platform.rs",
        r#"#[cfg_attr(any(target_os = "emscripten", feature = "qzxvfeat"), ignore)]
#[cfg(not(target_os = "qzxvos"))]
fn platform() -> &'static str {
    if cfg!(feature = "qzxvstd") { "with the library" } else { "without it" }
}
"#,
        expect![[r#"
            rust
            > #[cfg_attr(any(target_os = "emscripten", feature = "qzxvfeat"), ignore)]
            > #[cfg(not(target_os = "qzxvos"))]
            > fn platform() -> &'static str {
                 ^^^^^^^^ declaration
            >     if cfg!(feature = "qzxvstd") { "with the library" } else { "without it" }
                                                  ^^^^^^^^^^^^^^^^ string
                                                                              ^^^^^^^^^^ string
            > }
        "#]],
    );
}
