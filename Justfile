# Run `just --list` for the full set. `just pr-ready` is the gate before asking for review.

# Full test suite.
test:
    cargo nextest run --workspace

# Fast inner loop for one crate.
test-one crate filter="":
    cargo nextest run -p {{crate}} {{filter}}

# Accept new snapshot output, for the workspace or one crate. Read the diff before committing it.
# Plain `cargo test`, never nextest: nextest runs one process per test, and several of them
# accepting into one file clobber each other (`CLAUDE.md` §8.2).
accept crate="":
    UPDATE_EXPECT=1 cargo test {{ if crate == "" { "--workspace" } else { "-p " + crate } }}

[doc("The boundaries: allowed edges, foreign engines, filesystem reads on the request path.")]
arch:
    cargo test -p orthra-arch

# Markdown's formatter, at the version CI pins: lines wrapped at 100 (`.prettierrc.json`), as
# lspkit's are. Zed formats a Markdown file with it on save (`.zed/settings.json`).
prettier := "npx --yes prettier@3.8.0"

[doc("Formatting and clippy, the way CI runs them.")]
lint:
    cargo fmt --all --check
    {{prettier}} --check "**/*.md"
    cargo clippy --workspace --all-targets -- -D warnings

[doc("Format every crate, and every Markdown file.")]
fmt:
    cargo fmt --all
    {{prettier}} --write "**/*.md"

[doc("Licenses, bans, sources and advisories.")]
deny:
    cargo deny check

[doc("The gate before asking for review.")]
pr-ready: lint test deny

[doc("Fetch the GitHub Typo Corpus, verify it, and write the 2 000-typo sample. Local only, never committed.")]
typo-corpus:
    python3 -I tools/typo-corpus.py fetch
    python3 -I tools/typo-corpus.py sample

[doc("Score codebook-lsp on the typo sample: recall, top-1, top-5.")]
typo-codebook:
    python3 -I tools/typo-corpus.py codebook

[doc("Fetch the zero corpus (phrost at a pinned commit, ripgrep 15.2.0, composer 2.10.3, helix 25.07.1, starter workflows). Local only.")]
zero-corpus:
    python3 -I tools/zero-corpus.py fetch

[doc("orthra's findings on each project of the zero corpus, by code, against corpora/zero/. Exit 1 if one rose.")]
zero *args:
    cargo build --release -p orthra-cli
    python3 -I tools/zero-corpus.py ratchet {{args}}

[doc("What codebook-lsp reports on the zero corpus, by place and by word.")]
zero-codebook:
    python3 -I tools/zero-corpus.py codebook

[doc("Fetch UA-GEC at a pinned commit and sample 1 000 Ukrainian spelling errors. Local only.")]
ua-gec:
    python3 -I tools/ua-gec.py fetch
    python3 -I tools/ua-gec.py sample

[doc("Score codebook-lsp, with only `uk`, on the UA-GEC sample.")]
ua-gec-codebook:
    python3 -I tools/ua-gec.py codebook

[doc("Score orthra, with only uk-UA, on the UA-GEC sample. Builds the release binary first.")]
ua-gec-orthra:
    cargo build --release -p orthra-cli
    python3 -I tools/ua-gec.py orthra

[doc("Score orthra on the typo sample. Builds the release binary first.")]
typo-orthra:
    cargo build --release -p orthra-cli
    python3 -I tools/typo-corpus.py orthra

[doc("The context tier judged (docs/04-context.md §7): the typo sample scored with its model off and on, its latency and its size. Needs `orthra model install small`.")]
eval-context:
    cargo build --release -p orthra-cli
    python3 -I tools/typo-corpus.py orthra
    python3 -I tools/typo-corpus.py context

[doc("orthra beside codebook on the zero corpus, for the files orthra has a host for.")]
zero-orthra:
    cargo build --release -p orthra-cli
    python3 -I tools/zero-corpus.py orthra

[doc("orthra beside codebook (and typos and cspell, when installed) on the files under <dir> orthra reads: both / orthra only / other only, and precision and recall from the labels in corpora/compare/. `--sample` prints what to label.")]
compare dir *args:
    cargo build --release -p orthra-cli
    python3 -I tools/compare.py {{dir}} {{args}}

[doc("`just compare` on every project of the zero corpus, with the totals.")]
compare-zero:
    cargo build --release -p orthra-cli
    python3 -I tools/compare.py --zero

[doc("Every budget in crates/cli/src/perf/budget.rs, measured on each project of the zero corpus. Exit 1 if one is over.")]
perf:
    #!/usr/bin/env bash
    set -u
    cargo build --release -p orthra-cli || exit 1
    # One project per run, as an editor serves one workspace: a run over all of them would hold
    # every project's words at once, and measure a server nobody runs.
    status=0
    for project in .cache/corpora/zero/*/; do
        echo "== $project"
        ./target/release/orthra perf "$project" || status=1
    done
    exit $status

[doc("This machine's release archive in dist/, packed as the release workflow packs one. Needs cargo-about 0.9.3 (`cargo install cargo-about --locked --features cli`).")]
dist:
    #!/usr/bin/env bash
    set -eu
    target=$(rustc -vV | sed -n 's/^host: //p')
    cargo build --release --locked -p orthra-cli --target "$target"
    sh tools/dist/smoke.sh "target/$target/release/orthra"
    cargo about generate --locked --manifest-path crates/cli/Cargo.toml -o target/THIRD-PARTY-LICENSES.md tools/dist/third-party-licenses.hbs
    sh tools/dist/pack.sh "$target" "target/$target/release/orthra" target/THIRD-PARTY-LICENSES.md dist
