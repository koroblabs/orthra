//! The allowed-edges matrix for the workspace, and who may name a foreign engine.
//!
//! Only `orthra-*` and `lspkit-*` dependencies, and the engines in [`FOREIGN`], are policed. Every
//! other third-party crate is a per-crate choice. The graph between our own layers, and the reach
//! of the engines we expect to swap, are not.
//!
//! Adding an edge is a deliberate edit here, with its reason in the commit. It is never a side
//! effect of a `use` added during a refactor. The engine is `lspkit_arch::edges`.

use lspkit_arch::edges::{Edges, Foreign, Pattern, Rule};

#[cfg(test)]
mod tests;

/// Test-only dependencies every crate may take. The fixture format is not a layer; it describes
/// inputs, and without it every layer reinvents the same scaffolding.
const DEV_EVERYWHERE: &[&str] = &["orthra-testkit", "orthra-text"];

/// The harness builds sessions over the whole pipeline, so only the crates that own one may test
/// with it.
const WITH_HARNESS: &[&str] = &["orthra-harness"];

const RULES: &[Rule] = &[
    Rule::new(
        "orthra-cli",
        &[
            // The global allocator: only the binary installs one.
            "lspkit-heap",
            "lspkit-perf",
            "lspkit-ratchet",
            "orthra-install",
            "orthra-lsp",
            // The context tier's model runtime, behind the `ml-local` feature: the binary is what
            // hands it to the server and the model command, which never name it.
            "orthra-ml-local",
            // The context tier through an endpoint the user runs, behind the `ml-http` feature:
            // the binary hands its connector to the server, which never names an HTTP client.
            "orthra-ml-http",
            "orthra-project",
            "orthra-engine",
            "orthra-dict",
            "orthra-packs",
            "orthra-text",
        ],
    )
    .dev(WITH_HARNESS),
    // The panic containment and the transport phrost measured its way to (`01-architecture.md` §9).
    Rule::new(
        "orthra-lsp",
        &[
            // To give the context tier's freed memory back, and count what its model costs; the
            // allocator itself is installed by the binary.
            "lspkit-heap",
            "orthra-project",
            "orthra-engine",
            "orthra-install",
            "orthra-text",
            "lspkit-runtime",
            "lspkit-text",
        ],
    )
    // Driven in process by its flow tests, through lspkit's harness, which phrost's are too.
    .dev(&["orthra-harness", "lspkit-flow"]),
    // What a project is -- its files, manifests and declarations -- in one place that both the
    // command line and the server set their engine up from, so `orthra check` reports what the
    // editor would (`CLAUDE.md` §8.3).
    Rule::new(
        "orthra-project",
        &[
            // To give back what a long pass's parses free as it goes, not only once it is done.
            "lspkit-heap",
            "orthra-engine",
            "orthra-text",
            "lspkit-walk",
        ],
    ),
    Rule::new(
        "orthra-engine",
        &[
            "orthra-checks",
            "orthra-suggest",
            "orthra-context",
            "orthra-lexicon",
            "orthra-dict",
            "orthra-segment",
            "orthra-hosts",
            "orthra-packs",
            "orthra-text",
        ],
    )
    .dev(WITH_HARNESS),
    Rule::new(
        "orthra-checks",
        &[
            "orthra-lexicon",
            "orthra-segment",
            "orthra-hosts",
            "orthra-packs",
            "orthra-text",
        ],
    ),
    Rule::new(
        "orthra-suggest",
        &[
            "orthra-lexicon",
            "orthra-dict",
            "orthra-packs",
            "orthra-text",
        ],
    ),
    Rule::new("orthra-context", &["orthra-text"]),
    // The context tier's model in process: a leaf, the only crate that knows candle exists, and
    // the scorer it makes is the contract's.
    Rule::new("orthra-ml-local", &["orthra-context", "orthra-text"]),
    // The context tier through an endpoint the user runs: a leaf, the only crate that sends a
    // document's text anywhere, and the rewriter it makes is the contract's.
    Rule::new("orthra-ml-http", &["orthra-context"]),
    Rule::new("orthra-lexicon", &["orthra-text"]),
    Rule::new("orthra-packs", &["orthra-dict", "orthra-text"]),
    // Installing a pack orthra does not carry, or the context tier's model: an archive, a hash, and
    // the files written. A model is built by the contract's `Runtime`, handed in, never named.
    Rule::new("orthra-install", &["orthra-packs", "orthra-context"]),
    Rule::new("orthra-dict", &["orthra-text"]),
    Rule::new("orthra-segment", &["orthra-text"]),
    Rule::new("orthra-hosts", &["orthra-text"]),
    // Lines, columns and encodings come from lspkit, shared with phrost; with its `lsp` feature
    // only in the server, which is the one crate that may name a protocol type.
    Rule::new("orthra-text", &["lspkit-text"]),
    // The fixture format is lspkit's, which phrost writes too; what its flags mean is orthra's.
    Rule::new("orthra-testkit", &["orthra-text", "lspkit-fixture"]),
    // lspkit-fuzz is the edit sweep, which this harness's `incremental == from-scratch` dimension
    // draws its seeded edits from; the harness is itself test support, so the edge is a normal one.
    Rule::new(
        "orthra-harness",
        &["orthra-testkit", "orthra-text", "lspkit-fuzz"],
    ),
    Rule::new("orthra-arch", &[]).dev(&["lspkit-arch"]),
];

const TREE_SITTER: &str = "grammars are built for one tree-sitter ABI range, and a mismatch fails at \
    runtime; one crate to re-pin";
const DICTIONARY: &str = "spellbook calls itself alpha, and the default dictionary is decided by \
    measurement; one crate to swap";
const RUNTIME: &str = "the model runtime is an open question, and lives in a leaf behind a feature";
const NETWORK: &str = "text leaves the machine only through the loopback-guarded model client; \
    orthra-install fetches a pinned archive by a pack's tag and never sees a document";

const INSTALL: &str = "an archive and its hash are an install's, which never sees a document";

const FOREIGN: &[Foreign] = &[
    Foreign {
        pattern: Pattern::Exact("zip"),
        owners: &["orthra-install"],
        why: INSTALL,
    },
    Foreign {
        pattern: Pattern::Exact("sha2"),
        owners: &["orthra-install"],
        why: INSTALL,
    },
    Foreign {
        pattern: Pattern::Exact("tree-sitter"),
        owners: &["orthra-hosts"],
        why: TREE_SITTER,
    },
    Foreign {
        pattern: Pattern::Prefix("tree-sitter-"),
        owners: &["orthra-hosts"],
        why: TREE_SITTER,
    },
    Foreign {
        pattern: Pattern::Exact("pulldown-cmark"),
        owners: &["orthra-hosts"],
        why: "where prose lives in Markdown is the host's question",
    },
    Foreign {
        pattern: Pattern::Exact("spellbook"),
        owners: &["orthra-dict"],
        why: DICTIONARY,
    },
    Foreign {
        pattern: Pattern::Exact("fst"),
        owners: &["orthra-dict"],
        why: DICTIONARY,
    },
    Foreign {
        pattern: Pattern::Prefix("candle-"),
        owners: &["orthra-ml-local"],
        why: RUNTIME,
    },
    Foreign {
        pattern: Pattern::Exact("ort"),
        owners: &["orthra-ml-local"],
        why: RUNTIME,
    },
    Foreign {
        pattern: Pattern::Exact("reqwest"),
        owners: &["orthra-ml-http", "orthra-install"],
        why: NETWORK,
    },
    Foreign {
        pattern: Pattern::Exact("ureq"),
        owners: &["orthra-ml-http", "orthra-install"],
        why: NETWORK,
    },
    Foreign {
        pattern: Pattern::Exact("hyper"),
        owners: &["orthra-ml-http", "orthra-cli"],
        why: NETWORK,
    },
    Foreign {
        pattern: Pattern::Prefix("hyper-"),
        owners: &["orthra-ml-http", "orthra-cli"],
        why: NETWORK,
    },
];

const EDGES: Edges<'static> = Edges {
    policed: &["orthra-", "lspkit-"],
    rules: RULES,
    dev_everywhere: DEV_EVERYWHERE,
    foreign: FOREIGN,
};
