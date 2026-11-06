use tree_sitter::Point;

use super::{advance, point_at};

#[test]
fn rows_and_byte_columns_as_tree_sitter_counts_them() {
    let text = "ab\nПри\nc";
    // `При` is two bytes a letter; byte 9 is the line break itself, and 11 the end.
    let points = [0, 2, 3, 5, 9, 10, 11]
        .map(|at| point_at(text, at))
        .map(|Point { row, column }| format!("{row}:{column}"));
    expect_test::expect!["0:0 0:2 1:0 1:2 1:6 2:0 2:1"].assert_eq(&points.join(" "));

    let from = Point { row: 1, column: 2 };
    let advanced = ["", "xy", "x\n", "x\nyz\nП"]
        .map(|written| advance(from, written))
        .map(|Point { row, column }| format!("{row}:{column}"));
    expect_test::expect!["1:2 1:4 2:0 3:2"].assert_eq(&advanced.join(" "));
}
