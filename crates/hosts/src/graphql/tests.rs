use expect_test::expect;

use crate::tests::check;

#[test]
fn graphql_prose_is_comments_descriptions_and_declared_names() {
    check(
        "/schema.graphql",
        r#"# The root of the schema.
"""
A user of the `service`.
"""
type User implements Node {
  "The display name."
  displayName(format: NameFormat = SHORT): String!
  friends(first: Int): [User!]!
}

enum NameFormat {
  SHORT
  LONG_FORM
}

query GetUser($userId: ID!) {
  user(id: $userId) {
    ...userFields
    displayName(format: SHORT)
  }
}

fragment userFields on User {
  displayName
}
"#,
        expect![[r#"
            graphql
            > # The root of the schema.
              ^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > """
              ^^^ string (+2 lines)
            > A user of the `service`.
            > """
            > type User implements Node {
                   ^^^^ declaration
            >   "The display name."
                ^^^^^^^^^^^^^^^^^^^ string
            >   displayName(format: NameFormat = SHORT): String!
                ^^^^^^^^^^^ declaration
                            ^^^^^^ declaration
            >   friends(first: Int): [User!]!
                ^^^^^^^ declaration
                        ^^^^^ declaration
            > }
            >
            > enum NameFormat {
                   ^^^^^^^^^^ declaration
            >   SHORT
                ^^^^^ declaration
            >   LONG_FORM
                ^^^^^^^^^ declaration
            > }
            >
            > query GetUser($userId: ID!) {
                    ^^^^^^^ declaration
                             ^^^^^^ declaration
            >   user(id: $userId) {
            >     ...userFields
            >     displayName(format: SHORT)
            >   }
            > }
            >
            > fragment userFields on User {
                       ^^^^^^^^^^ declaration
            >   displayName
            > }
        "#]],
    );
}
