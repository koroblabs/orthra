//! A file a tool wrote, told by its text rather than its name: a minified bundle is one line, a
//! parser's tables are tens of megabytes, and a person writes neither. Such a file has no findings
//! and declares nothing.

/// The largest file a person writes, in bytes. The largest on the projects beside orthra and in
/// the zero corpus are 345 KB, a design document, and 279 KB, helix's changelog (2026-11-09). A
/// check costs about 25 MB of memory and 1.2 s a megabyte on the VM, and a project's files are
/// checked four at a time: in the cargo registry, tree-sitter grammars' generated `parser.c`, up
/// to 56 MB, took the server to 5 GB and its first diagnostic to 89 s.
const LARGEST_FILE: usize = 2 << 20;

/// The longest line a person writes, in characters. cspell blocks a file at the same length
/// (`blockCheckingWhenLineLengthGreaterThan`). A 2 MB bundle named `bundle.js` took 3.4 s to check,
/// and 463 ms a keystroke, holding the server meanwhile (2026-11-09, on the VM).
const LONGEST_LINE: usize = 10_000;

/// Blocks of half that length, aligned: a line longer than [`LONGEST_LINE`] covers at least one
/// whole, so only a block with no newline in it is a place to look. In every other text each block
/// has a newline within a few bytes of its start, and a check of the whole text asks a few dozen
/// of them where reading every line was measurable on a keystroke.
const BLOCK: usize = LONGEST_LINE / 2;

/// Whether `text` is larger than [`LARGEST_FILE`], or has a line longer than [`LONGEST_LINE`].
pub(crate) fn written_by_a_tool(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() > LARGEST_FILE
        || bytes.len() > LONGEST_LINE
            && bytes.chunks(BLOCK).enumerate().any(|(i, block)| {
                block.len() == BLOCK && !block.contains(&b'\n') && too_long(text, i * BLOCK)
            })
}

/// Whether the line around the byte `at` is longer than [`LONGEST_LINE`], in characters.
fn too_long(text: &str, at: usize) -> bool {
    let bytes = text.as_bytes();
    let start = bytes[..at]
        .iter()
        .rposition(|&b| b == b'\n')
        .map_or(0, |newline| newline + 1);
    let end = bytes[at..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |newline| at + newline);
    let line = &text[start..end];
    line.len() > LONGEST_LINE && line.chars().count() > LONGEST_LINE
}
