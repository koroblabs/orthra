# orthra — planning documents

orthra is a spell-checking language server in Rust: fast, quiet and modular. It targets Zed first
and every LSP client after that.

Read in order:

1. **[01-architecture.md](01-architecture.md)**: the pipeline, the crate graph, the two seams, the
   check registry, the dictionary boundary, and why projections are the unit of cost.
2. **[02-testing.md](02-testing.md)**: the test levels. **This is the most important document.**
3. **[03-roadmap.md](03-roadmap.md)**: phases whose exit criteria are measurements, open questions
   and what is refused.
4. **[04-context.md](04-context.md)**: the local-model tier. It gets its own document because it is
   the one part that is an experiment, and an experiment needs a written contract before it runs.

`../CLAUDE.md` is the operating manual. It covers the performance contract, how agents split the
work, the hard NEVERs, the crate map and the gotchas.

## The rules everything else follows from

1. **Quiet beats clever.** A false positive costs more than a false negative. A noisy checker gets
   turned off, and then it catches nothing. The zero-diagnostics corpus is a test, not a mood.

2. **A boundary is the dependency graph, not the documentation.** Exactly one crate may name
   tree-sitter, exactly one may name spellbook, and exactly one may name a model runtime. A test
   enforces all of it.

3. **Findings are a function of the inputs.** Same text + language id + config + packs = same
   findings. No fallbacks, no edit-history tricks. The context tier is the single exception, and it
   is labeled as one.

4. **Adding a language is data.** For a natural language, that means a directory with a `pack.toml`
   and a dictionary. For a host language, it means a directory with a tree-sitter query. Neither
   should need a diff to generic code.

5. **An invariant beats a test.** These four are dimensions that every fixture runs through:
   `incremental == from-scratch`, `cold == warm`, `shift` and `context-off ⊆ context-on`.

6. **Measure, then build.** Five instruments ask five different questions (`CLAUDE.md` §9). The top
   of a ranked list beats an idea about what matters.
