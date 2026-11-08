use expect_test::expect;

use crate::tests::check;

#[test]
fn java_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/main/java/org/example/Greeter.java",
        r##"package org.example.greeting;

import java.util.List;
import static java.lang.Math.max;

/**
 * Says hello to whoever asks. See {@link List} and {@code max}.
 *
 * @param <T> The kind of greeting.
 * @author Jordi Boggiano
 */
@SuppressWarnings("unchecked")
public final class Greeter<T> implements Runnable {
    private static final String PREFIX = "Hello, ";
    private int count = 0, total;

    // Builds the greeting.
    /* A block comment. */
    public Greeter(int count) { this.count = count; }

    /**
     * Greets one person, like <code>greet("abc")</code> or {@code greet({@link Foo})}, quietly.
     * <pre>
     *   greeter.greet("abcxyz");
     * </pre>
     * After the block.
     *
     * @param name Who to greet.
     * @param extra More names.
     * @return The greeting for everyone.
     * @throws IllegalStateException When nobody is there.
     */
    @Override
    public String greet(String name, String... extra) throws Exception {
        String message = "Dear " + name + "\nwelcome";
        char quote = '\'';
        String block = """
            A text block,
            over two lines.\tTabbed
            """;
        for (String item : extra) {
            System.out.println(item.length());
        }
        try (var reader = open()) {
            list.forEach(entry -> use(entry));
        } catch (IOException failure) {
            throw failure;
        }
        return PREFIX + message + block;
    }

    enum Color { RED, DARK_GREEN }
    record Point(int xCoord, int yCoord) {
        Point { count(); }
    }
    @interface Marker { String label() default "none"; }
}
"##,
        expect![[r#"
            java
            > package org.example.greeting;
            >
            > import java.util.List;
            > import static java.lang.Math.max;
            >
            > /**
            >  * Says hello to whoever asks. See {@link List} and {@code max}.
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
                                                              ^^^ doccomment
                                                                             ^ doccomment
            >  *
            >  * @param <T> The kind of greeting.
                            ^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @author Jordi Boggiano
            >  */
            > @SuppressWarnings("unchecked")
            > public final class Greeter<T> implements Runnable {
                                 ^^^^^^^ declaration
            >     private static final String PREFIX = "Hello, ";
                                              ^^^^^^ declaration
                                                        ^^^^^^ value
            >     private int count = 0, total;
                              ^^^^^ declaration
                                         ^^^^^ declaration
            >
            >     // Builds the greeting.
                  ^^^^^^^^^^^^^^^^^^^^^^^ comment
            >     /* A block comment. */
                  ^^^^^^^^^^^^^^^^^^^^^^ comment
            >     public Greeter(int count) { this.count = count; }
                         ^^^^^^^ declaration
                                     ^^^^^ declaration
            >
            >     /**
            >      * Greets one person, like <code>greet("abc")</code> or {@code greet({@link Foo})}, quietly.
                     ^^^^^^^^^^^^^^^^^^^^^^^ doccomment
                                                                       ^^ doccomment
                                                                                                    ^^^^^^^^^^ doccomment
            >      * <pre>
            >      *   greeter.greet("abcxyz");
            >      * </pre>
            >      * After the block.
                     ^^^^^^^^^^^^^^^^ doccomment
            >      *
            >      * @param name Who to greet.
                                 ^^^^^^^^^^^^^ doccomment
            >      * @param extra More names.
                                  ^^^^^^^^^^^ doccomment
            >      * @return The greeting for everyone.
                             ^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >      * @throws IllegalStateException When nobody is there.
                                                   ^^^^^^^^^^^^^^^^^^^^^ doccomment
            >      */
            >     @Override
            >     public String greet(String name, String... extra) throws Exception {
                                ^^^^^ declaration
                                             ^^^^ declaration
                                                             ^^^^^ declaration
            >         String message = "Dear " + name + "\nwelcome";
                             ^^^^^^^ declaration
                                        ^^^^ value
                                                           ^^^^^^^ value
            >         char quote = '\'';
                           ^^^^^ declaration
            >         String block = """
                             ^^^^^ declaration
            >             A text block,
                          ^^^^^^^^^^^^^ string (+1 lines)
            >             over two lines.\tTabbed
                                           ^^^^^^ value
            >             """;
            >         for (String item : extra) {
                                  ^^^^ declaration
            >             System.out.println(item.length());
            >         }
            >         try (var reader = open()) {
                               ^^^^^^ declaration
            >             list.forEach(entry -> use(entry));
                                       ^^^^^ declaration
            >         } catch (IOException failure) {
                                           ^^^^^^^ declaration
            >             throw failure;
            >         }
            >         return PREFIX + message + block;
            >     }
            >
            >     enum Color { RED, DARK_GREEN }
                       ^^^^^ declaration
                               ^^^ declaration
                                    ^^^^^^^^^^ declaration
            >     record Point(int xCoord, int yCoord) {
                         ^^^^^ declaration
                                   ^^^^^^ declaration
                                               ^^^^^^ declaration
            >         Point { count(); }
                      ^^^^^ declaration
            >     }
            >     @interface Marker { String label() default "none"; }
                             ^^^^^^ declaration
                                             ^^^^^ declaration
                                                              ^^^^ value
            > }
        "#]],
    );
}
