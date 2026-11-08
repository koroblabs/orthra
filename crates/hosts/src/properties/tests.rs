use expect_test::expect;

use crate::tests::check;

#[test]
fn properties_prose_is_comments_and_the_text_of_values() {
    check(
        "/src/main/resources/messages.properties",
        r#"# Messages for the owner form.
! Seperates teh sections.
welcome=Welcome to the clinic
required=is required, occured twice
notFound=was not found: {0}
duplicate = is already taken \
    by somebody else
server.port=8080
spring.datasource.url=${DB_URL:jdbc:h2:mem:petclinic}
empty=
key\ with\ space: value
"#,
        expect![[r#"
            properties
            > # Messages for the owner form.
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > ! Seperates teh sections.
              ^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > welcome=Welcome to the clinic
              ^^^^^^^ value
                      ^^^^^^^^^^^^^^^^^^^^^ string
            > required=is required, occured twice
              ^^^^^^^^ value
                       ^^^^^^^^^^^^^^^^^^^^^^^^^^ string
            > notFound=was not found: {0}
              ^^^^^^^^ value
                       ^^^^^^^^^^^^^^^^^^ string
            > duplicate = is already taken \
              ^^^^^^^^^ value
                          ^^^^^^^^^^^^^^^^ string
            >     by somebody else
                  ^^^^^^^^^^^^^^^^ string
            > server.port=8080
              ^^^^^^^^^^^ value
                          ^^^^ value
            > spring.datasource.url=${DB_URL:jdbc:h2:mem:petclinic}
              ^^^^^^^^^^^^^^^^^^^^^ value
                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ value
            > empty=
              ^^^^^ value
            > key\ with\ space: value
              ^^^^^^^^^^^^^^^^ value
                                ^^^^^ value
        "#]],
    );
}
