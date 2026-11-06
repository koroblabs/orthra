//! What lspkit asks of every server's workspace, and orthra keeps (`CLAUDE.md` §7): `panic =
//! "unwind"` in both profiles and `abort` in none, so a handler's panic is contained; `unsafe_code
//! = "forbid"` for the workspace, inherited by every member; and `Cargo.lock` committed, since
//! orthra is a binary. If `orthra-dict` ever maps a dictionary into memory, that crate's own lints
//! are the exception, said here.

use lspkit_arch::{baseline::Baseline, workspace::Workspace};

#[test]
fn the_workspace_keeps_lspkits_baseline() {
    let problems = Baseline::default().violations(&Workspace::above(env!("CARGO_MANIFEST_DIR"), 3));
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}
