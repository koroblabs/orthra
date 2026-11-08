use expect_test::expect;

use crate::tests::check;

#[test]
fn cpp_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/greet.cpp",
        r##"//! Greets whoever asks.
#include <string>
#include "greet.hpp"
#define GREET_MAX 4

namespace app::detail {

/// A greeter.
template <typename Value, int Count = 2, typename... Rest>
class Greeter : public Base {
public:
    explicit Greeter(const std::string &name) : name_(name) {}
    ~Greeter() = default;

    /**
     * Builds the greeting.
     * \brief Greets.
     * @param person who to greet
     * @tparam Value what it holds
     */
    std::string greet(const Person &person, int *length) const;

private:
    std::string name_;
    int counts_[4];
};

using Names = std::vector<std::string>;
enum class Color { Red, Green = 2 };

std::string Greeter::greet(const Person &person, int *length) const {
    auto raw = R"(raw text)"; // NOLINT(cert-err58-cpp)
    auto flag = "xyzzyq";
    std::string message = "Hello\nworld";
    for (const auto &item : names) {
        std::cout << item << "%s: %d\n";
    }
    auto [first, second] = pair;
    return message + std::to_string(*length);
}

}  // namespace app::detail
"##,
        expect![[r#"
            cpp
            > //! Greets whoever asks.
                  ^^^^^^^^^^^^^^^^^^^^ doccomment
            > #include <string>
            > #include "greet.hpp"
            > #define GREET_MAX 4
                      ^^^^^^^^^ declaration
            >
            > namespace app::detail {
                        ^^^ declaration
                             ^^^^^^ declaration
            >
            > /// A greeter.
                  ^^^^^^^^^^ doccomment
            > template <typename Value, int Count = 2, typename... Rest>
                                 ^^^^^ declaration
                                            ^^^^^ declaration
                                                                   ^^^^ declaration
            > class Greeter : public Base {
                    ^^^^^^^ declaration
            > public:
            >     explicit Greeter(const std::string &name) : name_(name) {}
                           ^^^^^^^ declaration
                                                      ^^^^ declaration
            >     ~Greeter() = default;
            >
            >     /**
            >      * Builds the greeting.
                     ^^^^^^^^^^^^^^^^^^^^ doccomment
            >      * \brief Greets.
                            ^^^^^^^ doccomment
            >      * @param person who to greet
                                   ^^^^^^^^^^^^ doccomment
            >      * @tparam Value what it holds
                                   ^^^^^^^^^^^^^ doccomment
            >      */
            >     std::string greet(const Person &person, int *length) const;
                              ^^^^^ declaration
                                                  ^^^^^^ declaration
                                                               ^^^^^^ declaration
            >
            > private:
            >     std::string name_;
                              ^^^^^ declaration
            >     int counts_[4];
                      ^^^^^^^ declaration
            > };
            >
            > using Names = std::vector<std::string>;
                    ^^^^^ declaration
            > enum class Color { Red, Green = 2 };
                         ^^^^^ declaration
                                 ^^^ declaration
                                      ^^^^^ declaration
            >
            > std::string Greeter::greet(const Person &person, int *length) const {
                                                       ^^^^^^ declaration
                                                                    ^^^^^^ declaration
            >     auto raw = R"(raw text)"; // NOLINT(cert-err58-cpp)
                       ^^^ declaration
                                ^^^^^^^^ string
            >     auto flag = "xyzzyq";
                       ^^^^ declaration
                               ^^^^^^ value
            >     std::string message = "Hello\nworld";
                              ^^^^^^^ declaration
                                         ^^^^^ value
                                                ^^^^^ value
            >     for (const auto &item : names) {
                                   ^^^^ declaration
            >         std::cout << item << "%s: %d\n";
                                            ^^^^^^ string
            >     }
            >     auto [first, second] = pair;
                        ^^^^^ declaration
                               ^^^^^^ declaration
            >     return message + std::to_string(*length);
            > }
            >
            > }  // namespace app::detail
                 ^^^^^^^^^^^^^^^^^^^^^^^^ comment
        "#]],
    );
}
