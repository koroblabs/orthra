//! Every filesystem read on the path a request takes, with the reason it is there.
//!
//! In a language server, a fallback is a read: something asks, gets nothing, and goes to disk to
//! make up for it. So reads are enumerated, and a new one fails this test until somebody writes
//! down what it is for. See `CLAUDE.md` §1.2. The scan is `lspkit_arch::reads`; the table is here.

use lspkit_arch::{
    reads::{Allowed, DEFAULT_PATTERNS, Reads},
    workspace::Workspace,
};

/// Path prefixes no request passes through, and why. Everything else under `crates/` is policed,
/// so a new crate is policed until somebody says otherwise here.
const UNPOLICED: &[(&str, &str)] = &[
    (
        "crates/cli/",
        "the command line reads the files it is pointed at; it is not on a request's path",
    ),
    (
        "crates/lib/",
        "test support: the fixture format and the harness",
    ),
    ("crates/tools/", "the checks read the workspace to check it"),
];

/// Policed files that read, and why. Paths are from the workspace root.
const ALLOWED: &[Allowed] = &[
    Allowed {
        file: "crates/packs/build.rs",
        why: "the build script embeds packs/ in the binary at compile time; it never runs on a request",
    },
    Allowed {
        file: "crates/install/src/lib.rs",
        why: "writes a pack it was asked to install, and reads it back once to check it; installing is a command, not a request",
    },
    Allowed {
        file: "crates/install/src/models.rs",
        why: "writes the model it was asked to install, and looks for its file once when the context tier is turned on; installing is a command or the settings' consent, not a request",
    },
    Allowed {
        file: "crates/lsp/src/context.rs",
        why: "reads the installed model once, in the background, when the editor's settings turn the context tier on: an input, never a fallback, and never on a code action",
    },
    Allowed {
        file: "crates/packs/src/installed.rs",
        why: "looks for an installed pack's files when the settings enable it, and reads its dictionary on the first word of its script: an input, not a fallback",
    },
    Allowed {
        file: "crates/project/src/manifest.rs",
        why: "reads a project's manifests when it is discovered: its package, dependencies and authors are inputs, not a fallback",
    },
    Allowed {
        file: "crates/project/src/config.rs",
        why: "reads the user's and the project's settings and words files when a project is set up, and again when one changes; writes a words file when somebody adds a word to it",
    },
    Allowed {
        file: "crates/project/src/lib.rs",
        why: "reads every file of a project once, to learn what it declares, before anything is checked",
    },
];

#[test]
fn the_request_path_reads_only_what_allowed_says() {
    let workspace = Workspace::above(env!("CARGO_MANIFEST_DIR"), 3);
    let reads = Reads {
        patterns: DEFAULT_PATTERNS,
        unpoliced: UNPOLICED,
        allowed: ALLOWED,
        min_policed: 10,
    };
    let problems = reads.violations(&workspace.rust_files("crates"));
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}
