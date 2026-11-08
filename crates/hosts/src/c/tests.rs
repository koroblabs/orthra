use expect_test::expect;

use crate::tests::check;

#[test]
fn c_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/greet.c",
        r##"/** Greets whoever asks. */
#include <stdio.h>
#include "greet.h"
#define GREET_MAX 4
#define SQUARE(value) ((value) * (value))

/// A greeter.
typedef struct greeter {
    const char *name;
    int counts[4];
    void (*callback)(int event);
} greeter_t;

enum color { COLOR_RED, COLOR_GREEN = 2 };

static int total = 0;
int greet(greeter_t *person, size_t length);

/*!
 * Builds the greeting.
 * @param person who to greet
 * @return how many were greeted
 */
int greet(greeter_t *person, size_t length) {
    // Print it.
    char message[16] = "Hello\nworld";
    printf("%s: %d\n", person->name, total);
    FILE *file = fopen("greet.txt", "rb"); // NOLINT(cert-err33-c)
    /** \param length how many */
    size_t index = strlen(message);
    printf("%dms of %s", total, "xyzzyq");
    puts("recieve");
    return index;
}
"##,
        expect![[r#"
            c
            > /** Greets whoever asks. */
                  ^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > #include <stdio.h>
            > #include "greet.h"
            > #define GREET_MAX 4
                      ^^^^^^^^^ declaration
            > #define SQUARE(value) ((value) * (value))
                      ^^^^^^ declaration
                             ^^^^^ declaration
            >
            > /// A greeter.
                  ^^^^^^^^^^ doccomment
            > typedef struct greeter {
                             ^^^^^^^ declaration
            >     const char *name;
                              ^^^^ declaration
            >     int counts[4];
                      ^^^^^^ declaration
            >     void (*callback)(int event);
                         ^^^^^^^^ declaration
                                       ^^^^^ declaration
            > } greeter_t;
                ^^^^^^^^^ declaration
            >
            > enum color { COLOR_RED, COLOR_GREEN = 2 };
                   ^^^^^ declaration
                           ^^^^^^^^^ declaration
                                      ^^^^^^^^^^^ declaration
            >
            > static int total = 0;
                         ^^^^^ declaration
            > int greet(greeter_t *person, size_t length);
                  ^^^^^ declaration
                                   ^^^^^^ declaration
                                                  ^^^^^^ declaration
            >
            > /*!
            >  * Builds the greeting.
                 ^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @param person who to greet
                               ^^^^^^^^^^^^ doccomment
            >  * @return how many were greeted
                         ^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  */
            > int greet(greeter_t *person, size_t length) {
                  ^^^^^ declaration
                                   ^^^^^^ declaration
                                                  ^^^^^^ declaration
            >     // Print it.
                  ^^^^^^^^^^^^ comment
            >     char message[16] = "Hello\nworld";
                       ^^^^^^^ declaration
                                      ^^^^^ value
                                             ^^^^^ value
            >     printf("%s: %d\n", person->name, total);
                          ^^^^^^ string
            >     FILE *file = fopen("greet.txt", "rb"); // NOLINT(cert-err33-c)
                        ^^^^ declaration
                                      ^^^^^^^^^ value
                                                   ^^ value
            >     /** \param length how many */
                                    ^^^^^^^^^^^ doccomment
            >     size_t index = strlen(message);
                         ^^^^^ declaration
            >     printf("%dms of %s", total, "xyzzyq");
                          ^^^^^^^^^^ string
                                               ^^^^^^ value
            >     puts("recieve");
                        ^^^^^^^ value
            >     return index;
            > }
        "#]],
    );
}

/// Doxygen writes its commands with a backslash as often as with `@`: `\param`'s name is a name,
/// `\brief` is a command and what follows it prose, and `\code` to `\endcode` is code.
#[test]
fn doxygen_backslash_commands_are_tags() {
    check(
        "/src/greet.c",
        r#"/**
 * \brief Greets a qzxva person.
 * \param qzxvname the person's name
 * \return the greeting
 * \code
 * qzxvcall(qzxvarg);
 * \endcode
 * After the qzxvb example.
 */
int greet(const char *qzxvname);
"#,
        expect![[r#"
            c
            > /**
            >  * \brief Greets a qzxva person.
                        ^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * \param qzxvname the person's name
                                 ^^^^^^^^^^^^^^^^^ doccomment
            >  * \return the greeting
                         ^^^^^^^^^^^^ doccomment
            >  * \code
            >  * qzxvcall(qzxvarg);
            >  * \endcode
            >  * After the qzxvb example.
                 ^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  */
            > int greet(const char *qzxvname);
                  ^^^^^ declaration
                                    ^^^^^^^^ declaration
        "#]],
    );
}
