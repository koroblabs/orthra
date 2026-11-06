use expect_test::expect;

use crate::tests::check;

#[test]
fn tsx_reads_the_text_between_tags() {
    check(
        "/src/banner.tsx",
        r#"// Shown at the top of every page.
export function Banner({ title }: Props) {
  return (
    <h1 className="banner">
      Welcome to the {title} page
    </h1>
  );
}
"#,
        expect![[r#"
            tsx
            > // Shown at the top of every page.
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > export function Banner({ title }: Props) {
                              ^^^^^^ declaration
            >   return (
            >     <h1 className="banner">
                                 ^^^^^^ string
            >       Welcome to the {title} page
                    ^^^^^^^^^^^^^^ string
                                           ^^^^ string
            >     </h1>
            >   );
            > }
        "#]],
    );
}
