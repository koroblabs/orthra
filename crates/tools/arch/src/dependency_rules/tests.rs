use expect_test::expect;
use lspkit_arch::workspace::{Manifest, Workspace};

use super::EDGES;

#[test]
fn the_workspace_obeys_its_rules() {
    let workspace = Workspace::above(env!("CARGO_MANIFEST_DIR"), 3);
    let mut problems = EDGES.violations(workspace.manifests());
    // orthra writes a rule when it creates the crate, never before, so a rule without a crate is a
    // leftover.
    problems.extend(EDGES.stale_rules(workspace.manifests()));
    problems.extend(EDGES.cycles());
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

/// orthra's rules, against manifests that break them. The engine's own cases are lspkit's tests;
/// these pin what *these rules* say about this project's shape.
#[test]
fn the_rules_refuse_what_the_architecture_forbids() {
    let manifests = [
        // The bottom of the graph reaching for the top.
        r#"
            [package]
            name = "orthra-text"
            [dependencies]
            orthra-engine.workspace = true
        "#,
        // Tree-sitter anywhere but the hosts.
        r#"
            [package]
            name = "orthra-checks"
            [dependencies]
            tree-sitter-rust = "0.24"
        "#,
        // A dictionary engine anywhere but the dictionary crate.
        r#"
            [package]
            name = "orthra-segment"
            [dependencies]
            spellbook = "0.4"
        "#,
        // The harness in a crate that owns no pipeline.
        r#"
            [package]
            name = "orthra-dict"
            [dev-dependencies]
            orthra-harness.workspace = true
        "#,
        // lspkit is policed too: the runtime belongs to the server, not to the engine.
        r#"
            [package]
            name = "orthra-engine"
            [dependencies]
            lspkit-runtime.workspace = true
        "#,
    ]
    .map(Manifest::parse);
    expect![[r#"
        orthra-text -> orthra-engine (Normal) is not an allowed edge. If it is genuinely right, add it to the rules with the reason in the commit; otherwise route the fact through a layer that is already allowed
        orthra-checks names tree-sitter-rust, which only orthra-hosts may: grammars are built for one tree-sitter ABI range, and a mismatch fails at runtime; one crate to re-pin
        orthra-segment names spellbook, which only orthra-dict may: spellbook calls itself alpha, and the default dictionary is decided by measurement; one crate to swap
        orthra-dict -> orthra-harness (Dev) is not an allowed edge. If it is genuinely right, add it to the rules with the reason in the commit; otherwise route the fact through a layer that is already allowed
        orthra-engine -> lspkit-runtime (Normal) is not an allowed edge. If it is genuinely right, add it to the rules with the reason in the commit; otherwise route the fact through a layer that is already allowed
    "#]]
    .assert_eq(&(EDGES.violations(&manifests).join("\n") + "\n"));
}
