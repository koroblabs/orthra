# CLAUDE.md — orthra (fast, quiet spell-checking language server)

orthra is a spell-checking language server in Rust — for Zed first, and for every LSP client after.
The **one rule everything is judged by** has two halves. It must be **fast and resource-cheap**, and
it must be **quiet**: a spell checker that cries wolf gets switched off, and then it catches
nothing. A change that adds hot-path latency, makes memory grow with workspace size or with a
dictionary nobody is using, or adds a false positive to the zero corpus is wrong by default, however
clever the feature is.

The name comes from Orthrus, the two-headed dog. One head is the **lexical tier**: dictionaries,
deterministic, always on. The other is the **context tier**: local models, opt-in, never on the hot
path. They do not share a mouth — see §1.5.

## Project state — read this first

Phases 0 to 3 are done, and 0.1.0 is the first release: `orthra serve` runs in Zed through
`orthra-zed`, which downloads the release binary for the platform. Phases 4 and 5 are done in part:
uk-UA is installed on first use and compiled to an FST, and the context tier, opt-in, reorders an
asked word's corrections with a model in the process and rewrites a sentence through an endpoint on
the machine. orthra checks words in 55 hosts: prose (plain text, Markdown, reStructuredText, commit
messages), thirty programming languages, the web's (HTML, Vue, Svelte, CSS, SCSS, LESS), data and
configuration (TOML, YAML, JSON, XML, Protocol Buffers, GraphQL, Nix, HCL, INI, properties) and
builds (Make, CMake, Dockerfile), listed in the README, in the editor and from the command line
(`orthra check`, `explain`, `perf`). Every number in §3 and in `docs/02-testing.md` level 9 is
measured, with its date. On the zero corpus orthra is 16 to 23 times quieter than codebook on the
same files, and `just zero` ratchets its counts so they only go down. What is still a target says
so.

orthra exists because the incumbent for Zed, codebook, is reported by the people who use it as not
good enough. "Not good enough" is a feeling. Phase 0 turns it into a list of failing fixtures, and
that list — not this plan — decides what orthra must do better. See `corpora/incumbent/README.md`.

The plan is these documents. They are authoritative until code contradicts them:

| Doc                       | Owns                                                                                                                                                                |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `docs/01-architecture.md` | the pipeline, the crate graph, the two seams (host languages, natural languages), the check registry, the dictionary boundary, why projections are the unit of cost |
| `docs/02-testing.md`      | the test levels — **the most important document**                                                                                                                   |
| `docs/03-roadmap.md`      | phases with measured exit criteria, open questions, and what is refused and why                                                                                     |
| `docs/04-context.md`      | the local-model tier: what it may do, what it may never do, how it is measured                                                                                      |

`../phrost` is the template: the same process, the same kind of instruments, and the same agent
rules. Read its `CLAUDE.md` §8.1 once. Those bug shapes come from index-shaped software, and a spell
checker is index-shaped software.

`../lspkit` is what the two share: whatever a language server needs that is not about its language,
moved out of phrost as code that was proven there. orthra depends on it by git, from the public
`github.com/koroblabs/lspkit`, **pinned to a `rev`**: lspkit is released by tag, and orthra moves to
a new one in a commit of its own. In use today:

- `lspkit-arch`, the engine behind §1.1 and §1.2;
- `lspkit-runtime`, phrost's panic containment and buffered transport, under `orthra serve`, and the
  protocol's types, re-exported as `lspkit_runtime::protocol`;
- `lspkit-text`, lines, columns, encodings and the client's edits;
- `lspkit-flow`, the harness the flow tests drive the server through, shared with phrost;
- `lspkit-perf`, the percentiles and the budget verdict of `orthra perf`;
- `lspkit-fixture`, the `//- /path`, `$0` and `@name@` format every fixture is written in; what its
  flags mean is `orthra-testkit`'s.

**What both servers need goes to lspkit, not into one of them twice:** a second copy is a second
place to fix the same bug. Code arrives there once a consumer has proven it and a second one needs
it, and leaves orthra in the same change. A change to lspkit reaches orthra as a `rev` bump in its
own commit, with `just pr-ready` green. lspkit's `CLAUDE.md` §2 says how.

If the code and the docs disagree, fix whichever one is wrong, and say which one you changed and
why.

---

## 1. The load-bearing rules

### 1.1 Boundaries are the dependency graph, not the docs

phrost uses the same rule for the same reason: a rule with no CI check does not hold. From the first
commit, `crates/tools/arch/src/dependency_rules.rs` holds an explicit allowed-edges matrix for the
workspace. It also polices **which crate may name a foreign engine**:

| foreign engine                                   | the only crate that may name it                                                                                                   |
| ------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------- |
| `tree-sitter`, `tree-sitter-*`, `pulldown-cmark` | `orthra-hosts`                                                                                                                    |
| `spellbook`, `fst`                               | `orthra-dict`                                                                                                                     |
| `tower-lsp-server`, `ls-types`                   | no orthra crate: `orthra-lsp` names them through `lspkit_runtime::protocol`, and `lspkit-arch` holds every crate to it            |
| a model runtime (`candle-*` or `ort`)            | `orthra-ml-local`, feature-gated                                                                                                  |
| an HTTP client                                   | `orthra-ml-http`, feature-gated; and `orthra-install`, which fetches a pack's pinned archive by its tag and never sees a document |

Each of these will be swapped or upgraded. Tree-sitter grammars break ABI between core versions,
spellbook calls itself alpha, and the model runtime is an open question. The boundary is what keeps
each swap a one-crate job. Adding an edge is a deliberate allow-list edit with a reason in the
commit, never an incidental `use` during a refactor.

### 1.2 Findings are a function of their inputs

A document's findings depend on its text, its language id, the config and the installed packs.
Nothing else feeds in. That rules out:

- trying a regex because tree-sitter failed;
- accepting every word because a dictionary is missing;
- depending on edit history, the cursor, timing, or the order in which files were opened.

Choosing an extractor per language id is a table, not a fallback. It is decided once, from the
language id, and never re-decided because the first choice answered badly.

The context tier is the one named exception: a model also feeds its findings. That is why it is
quarantined (§1.5).

This is enforced, not requested. `incremental == from-scratch` and `cold == warm` are harness
dimensions (§1.4). `crates/tools/arch/src/no_fallbacks.rs` holds an allow-list of the filesystem
reads on the request path, scanned by `lspkit-arch` the way phrost scans its own. A new read fails
the test until somebody writes down what it is for.

### 1.3 Minimize the conflict surface

This is copied from phrost §1.3 whole, because it is about how work parallelizes, not about PHP. Two
people, or two agents, working on different features must not need to edit the same file.

- **No shared expected-output file.** Snapshots live inline as `expect![[...]]`, next to the test
  that owns them.
- **No central `match` over checks, hosts or packs.** Checks and hosts live in a registry of trait
  objects. Packs are data directories. Adding one is at most **one line** in a shared list.
- **One check, one host, one pack per directory**, each with its own tests.
- **Prefer `pub mod` over a growing `pub use` list.**
- **A test file per behavior, not per crate.**
- **No `utils`.** A helper belongs in the crate that owns the concept.

The rule of thumb doubles as the acceptance test for the seams. Three people adding Go, Polish, and
a `repeated-word` check should touch three disjoint directories, plus at most one registry line
each.

The rule applies to users' repositories too. The project word list is one word per line, kept
sorted, so two branches that each add a word merge without a conflict.

### 1.4 Invariants are harness dimensions, not tests

**Every fixture runs through every dimension**, so one new fixture buys all of them:

- **`incremental == from-scratch`.** Apply a sequence of edits. The findings must be byte-identical
  to a fresh check of the final text.
- **`cold == warm`.** Compare an empty verdict cache with a warm one, and a compiled dictionary with
  one parsed from `.aff`/`.dic`.
- **`shift`.** Prepend a line of Cyrillic and an emoji. The findings must be identical, one line
  down. This is the UTF-16 test, and it costs no extra fixture.
- **`context-off ⊆ context-on`.** Turning the context tier on may add findings under their own
  source and may reorder suggestions. It may never remove, move or reword a lexical finding.

Build the dimensions in Phase 0. They cannot be added retroactively: phrost-old's parity check was
_one file_ instead of a dimension, and invalidation bugs shipped because of it.

### 1.5 The model is never on the keystroke path

The two tiers answer different questions:

- The lexical tier answers **"is this a word?"**
- The context tier answers **"is this the right word here?"** and **"which correction did you
  mean?"**

The context tier never answers the first question. It never blocks a response, never runs unless
enabled, and never proposes a word the lexical tier did not. Its findings carry their own LSP
`source`, so a user can tell a dictionary's certainty from a model's opinion. The server starts,
runs, and passes every test with no model on the machine. The full contract is in
`docs/04-context.md`.

---

## 2. How to work: parallel agents + model per task

The process every server in this family is worked on by -- delegating to agents with disjoint write
sets, the hazards that leak through worktrees, the gate, measuring, code quality -- is lspkit's,
shared with phrost and imported here whole:

@../lspkit/docs/working.md

What is orthra's own is below: where its write sets split, which work goes to which model here, and
the recipes that fit it. **Disjoint write sets** here are a crate, a host directory, a pack
directory or a check directory.

### Model tiers — pick by the risk the work creates, not by how big it looks

- **haiku** for reading:
  - analysis sweeps, code search and summaries;
  - doc edits;
  - **fixture conversion**: turning a typo corpus or the incumbent's failures into fixtures;
  - adding words to a technical vocabulary;
  - mechanical moves and renames, and `UPDATE_EXPECT` acceptance passes.
- **sonnet** for bounded work:
  - a new host language (grammar dependency + `spell.scm` + fixtures);
  - a new language pack;
  - a new check over projections that already exist;
  - LSP handler plumbing, tests, single-module refactors.
- **opus** for anything that creates a long-lived boundary or touches core semantics:
  - the crate graph, and the `Host`, `Pack`, `Check`, `Dictionary` and `Scorer` traits;
  - dictionary storage, the verdict cache and its invalidation;
  - the concurrency and cancellation model, and the context-tier contract;
  - **every perf/memory trade-off**;
  - cross-crate debugging: UTF-16 ranges, stale findings after an edit, a cache that survives a
    config change.

### Fan-out recipes that fit this repo

- **Mechanical bulk work → parallel haiku or sonnet.** One agent per host language, one per language
  pack, one per corpus conversion. This repo parallelizes best here, because hosts and packs are
  disjoint **by construction**. A design that makes them share a file has broken §1.3.
- **Adversarially verify** a perf, quietness or boundary claim with a verifier prompted to refute
  it: "prove this allocates per word", "find a correct word in the zero corpus that this change now
  reports", "prove the shifted fixture reports a different column".

---

## 3. Performance contract

Budgets will live in `crates/cli/src/perf/budget.rs`. Once that file exists, it is the source of
truth, not this table. `just perf <dir>` measures them against real files and exits non-zero when
one is exceeded.

**Measured baseline, end of Phase 1** (2026-11-05, release build, `orthra perf .cache/corpora/zero`:
34 Markdown files, 21 360 lines):

| budget (`perf/budget.rs`)                            |         measured |  limit |
| ---------------------------------------------------- | ---------------: | -----: |
| pack load (most of `initialize` → first diagnostics) |         17–25 ms | 100 ms |
| check per 2 000 lines, empty verdict cache           |           2.6 ms |  10 ms |
| check per 2 000 lines, warm                          | **2.3 ms, over** |   2 ms |
| one word's suggestions, p95                          |          0.52 ms |  15 ms |
| RSS after the run                                    |            25 MB |  40 MB |

The warm row stands in for a keystroke until checking is incremental (Phase 3), when a keystroke
re-checks the span it touched rather than the file. It is over by 15 %, and is recorded as over
rather than loosened. Segmentation is most of the cost: 102 → 38 ms across the run, after the
plain-word fast paths and buffers shared per document.

**With the tree-sitter hosts**, one project per run as an editor serves one workspace (`just perf`;
2026-11-06). Pack load is 16 ms and suggestions p95 at most 0.52 ms everywhere. A grammar loads on
its language's first document and is timed apart from the check: TypeScript's is the slowest at 14
ms, against a budget of 50.

| project                        |   lines | cold / 2 000 | keystroke p95 (limit 2) | slowest keystroke |   RSS |
| ------------------------------ | ------: | -----------: | ----------------------: | ----------------: | ----: |
| phrost (Rust, Markdown)        | 321 657 |       5.5 ms |                 0.71 ms |            4.8 ms | 28 MB |
| ripgrep (Rust, Markdown)       |  60 740 |       6.5 ms |                 1.10 ms |            3.5 ms | 26 MB |
| composer (PHP, JSON)           |  77 115 |       8.4 ms |                 0.41 ms |            0.8 ms | 22 MB |
| helix (Rust, Markdown, TOML)   | 326 242 |       2.8 ms |                 0.63 ms |            5.2 ms | 30 MB |
| starter workflows (YAML, JSON) |  12 239 |       6.4 ms |                 0.14 ms |            0.3 ms | 21 MB |

One run over all five holds every project's words at once and reaches 41 MB, over the 40 a server is
held to. A server holds one project.

**The server** learns what a project declares at startup, off the request path, on four threads, and
nothing is reported before it has. An open file's diagnostics arrive at 0.27–0.32 s on phrost
(0.82–0.93 on one thread), 0.18 s on helix (0.48), 0.14 s on composer (0.57) and 0.11 s on ripgrep
(0.31). Then it checks every file on the same four threads and publishes the closed ones' findings:
phrost's 345 by 0.52–0.58 s (1.66–1.77 on one). RSS is 35 MB after it against 24 MB on one thread,
but most of that is freed memory the system takes back when it needs it: the footprint a minute
later is 21 MB against 17. `initialize` answers at 7.4 MB, whether en-US alone or en-US and en-GB
are enabled: a pack's dictionaries are read on the first word of its script, not when it is enabled.
Read, en-US is 5.6 MB of that and en-GB another 2.7: packs loaded together share the files they both
name, and en-GB names en-US's known typos and vocabulary. Before they shared them, en-GB cost 12 MB.

- **2.4 MB of en-US is the rare words:** what SCOWL size 70 adds to the size-60 dictionary
  (`en_US-rare.dic`, 34 304 lines; 2.1 MB of live heap).
- **uk-UA, from dict_uk, was 37 MB and half a second to read** in spellbook (353 000 entries;
  measured 2026-11-06), and is now compiled when it is installed: an FST of every form, 3.9 MB, read
  in 0.02 s (`01-architecture.md` §7). That is why a pack is read on its script's first word: a
  project that enables uk-UA and opens English files pays nothing for it. They took the zero corpus
  from 2 481 findings to 2 248 (precision 1.80 → 1.94 % by `just compare-zero`) for 0.2 to 0.4
  points of recall on the typo sample.

- **English is compiled when orthra is** (2026-11-09, `01-architecture.md` §7): FSTs read where they
  are embedded, 0.7 MB of live heap after the packs where spellbook's tables were 12.8, and RSS
  after `perf` 25, 27 and 21 MB on ripgrep, helix and composer where it was 34, 37 and 30. Its
  frequencies too: parsed on the first correction asked for, they took 20 ms of its 24 and held 2
  MB; compiled, the first takes 4 ms, and RSS after `perf` is 3 MB lower again (helix 24.3). Linux
  also gives back what tree-sitter frees only when glibc is asked (`release_freed` after a pass and
  a close); macOS 26's allocator gives back nothing, which is why the compiled dictionaries mattered
  there. On macOS, mimalloc is the process's `malloc` (`lspkit-heap`'s `override`, lspkit 0.1.1),
  and `release_freed` gives back what tree-sitter frees. `just perf` on the Mac (2026-11-09) puts
  the footprint after the run at 6 to 9 MB on every project of the zero corpus: 8.6 on helix, 8.4 on
  ripgrep, 8.7 on phrost, 7.2 on composer and 6.1 on starter workflows, with the frequencies
  compiled; the slowest correction is 2.4 to 3.1 ms, where it was 16 to 18 before they were. Every
  budget was met; on one run of an earlier build, Ruby's query loaded in 53 ms against 50, where
  every other run took 25. Not on Linux, where glibc already gives it back and mimalloc's defaults
  held 4.5 MB more. 0.2.1 on the Mac (2026-11-10) met every budget too, with the footprint where it
  was (8.6 helix, 8.1 ripgrep, 8.8 phrost, 7.2 composer, 6.0 starter workflows): cold 3.2 to 9.4 ms
  per 2 000 lines, composer's the nearest its 10; keystroke p95 0.15 to 1.38 ms; the slowest grammar
  Ruby's, 27 ms.

- **The known typos are read where they are embedded,** as an offset per entry, and never copied. As
  a hash map of owned strings, en-US's 96 000 entries were 9 MB of heap in 288 000 allocations, and
  `initialize` sat at 22.8 MB.

- **A keystroke is parsed, queried and segmented again only where it changed.** The server keeps,
  for the one document being typed in, its tree, what the query captured in each part of it, and
  every span's words, all told of every edit: about 3 MB of live heap for the tree of a 4 000-line
  file, and 0.34 MB for its words. A keystroke went from a whole check, 2.6 to 7.7 ms per 2 000
  lines, to the column above. The query is cut at nodes of a sixteenth of the text
  (`tree_sitter_host.rs`, `capture_reusing`): a node larger is asked only for the patterns that
  start at it or its children, which keeps `#[cfg(test)]` and the `mod tests` after it together, and
  a smaller node the old tree shares is not asked again.
- **A keystroke is one keystroke per file, and the budget is its p95,** since the target is per
  keystroke. The budget used to be the keystrokes' total per 2 000 lines, which multiplied a small
  file's fixed cost: starter workflows' 55 µs keystrokes in 34-line files read as 3.3 ms, over.
- **The slowest keystrokes are in the largest files:** 4.8 ms in phrost's 3 960-line `skeleton.rs`,
  5.2 ms in helix's 4 523-line `languages.toml`. What such a keystroke still does over the whole
  file is the verdicts and the checks, one pass over every word. The next cut is to keep the
  verdicts of the spans an edit did not touch, as their words are kept.
- **RSS is the engine:** packs, grammars and the lexicon of the project's words. `orthra perf`
  reads, checks and drops each file, as a server holds only open documents. Holding every text had
  put it at 50 MB.

Still targets, not yet measurable:

| what                                                | target |
| --------------------------------------------------- | -----: |
| diagnostics after one keystroke (incremental)       |   2 ms |
| code actions with context re-ranking (opt-in)       |  80 ms |
| `orthra check` over ripgrep's repository, 8 threads | 300 ms |

**Memory targets:** RSS ≤ 40 MB with en-US, and ≤ 80 MB with en-US + uk-UA. A pack whose script
never appears in an open document costs nothing, because it is loaded on first use. A model's memory
is reported on its own line. It exists only while the context tier is on, and it is released after
the idle timeout.

**Both spellings of English by default, and CSS** (2026-11-08, on Workspace, a VM about three times
slower than the machine the tables above were taken on, so only its deltas compare). On lspkit, RSS
after `orthra perf` went from 22 to 28.5 MB: en-GB's dictionary is about 4 MB of it, read on the
first word en-US refuses, and the pages of the rare-word lists a lookup reads about 1. Cold and
keystroke did not move. A CSS keystroke costs 3.4 ms per 2 000 lines, Rust's 5.4 on the same machine
(`bootstrap.css`, 12 056 lines, against phrost's `skeleton.rs`); a 467-line stylesheet's, 1.1 ms.
The grammar loads in 1.3 ms.

**The hosts added on 2026-11-08,** on the same VM, beside Rust's 16 to 21 ms cold per 2 000 lines
and 0.6 to 0.8 ms keystroke p95: JavaScript, shell, HTML, Lua, Java, Kotlin, C and LESS cost about
what Rust does. Dense SQL costs four to five times Rust per line (13 ms for a keystroke in
pgvector's 918-line `vector.sql`). SCSS's old grammar recovers from errors on every `!default`:
twice Rust cold, six times on a keystroke, 35 ms at worst in bootstrap's `_variables.scss`.
tree-sitter-cpp is slow on template-heavy headers, about 90 ms a keystroke in fmt's `format.h`
whatever the query. Ruby's query loads in 63 ms and Swift's in about 45, against a 50 ms budget,
after merging patterns that tree-sitter's query analysis made expensive (a pattern rooted at Swift's
`function_declaration` alone cost 200 ms).

**The hosts added on 2026-11-09,** on the same VM with agents building beside it, so their ratios to
Rust's run of the moment stand and their absolutes do not. Most cost about what Rust does, or up to
three times it cold: C#, Scala, Groovy, Dart, Zig, Objective-C, OCaml, Elm, Elixir, Erlang, R, Nix,
HCL, INI, XML, Protocol Buffers, GraphQL, Make. Over a budget, and measured again in the perf stage:

- **A keystroke:** tree-sitter-fsharp and tree-sitter-haskell parse slowly, 19 to 27 ms a keystroke
  p95 against 2, 60 ms at worst in containers' 5 000-line `Data/Sequence/Internal.hs`. CMake's
  `#[=[ ]=]` docs are one node each, 17 ms p95. PowerShell's parse puts it at 5 ms.
- **A query's load:** Julia's at 92 to 105 ms after it was cut from 404, Elixir's at 78 (a pattern
  under its `arguments` node costs about 30 ms of tree-sitter's analysis; split into its five
  alternatives it cost more), and Dart's grammar at 25 to 97, against 50. On 2026-11-09 a
  declaration's child written `(_)` and kept when it is a plain name, rather than `(identifier)`,
  took Objective-C from 104 to 52 ms, Scala from 70 to 44, C# from 57 to 34 and Julia from 98 to 83:
  the analysis of a named node type the grammar reaches from many states is what costs.
- **reStructuredText parses whole on every keystroke** (`TreeSitterHost::parsed_whole`, §8.1): about
  10 ms for an 836-line page.

**Memory rule:** RSS is dominated by _the dictionaries in use plus the open documents_. If RSS
scales with workspace size, that is the bug. The project lexicon stores hashed words, not text, for
the same reason as phrost's `WordSet`.

Measure before and after any change that touches extraction, segmentation, the dictionary, the
verdict cache or the server loop, and report the p95 and RSS deltas. "It compiles" is not "it's
fast." Run sweeps sequentially on an idle machine. A concurrent `cargo build` reads exactly like a
regression.

---

## 4. Definition of done

1. `cargo check` and the relevant `cargo test` are green, and `cargo fmt` and `cargo clippy` are
   clean.
2. New behavior has a snapshot test at the **layer that owns it** (see §6), not only end-to-end.
3. Snapshots are of the **whole result**, never a substring. A test for "is `receive` among the
   suggestions" lets nine junk suggestions through.
4. Hot path touched → perf measured and the deltas reported.
5. Anything that can add a finding → `just zero` re-run. The count must be unchanged, or every new
   finding must be explained.
6. Invalidation touched (edits, config, word lists, packs) → `incremental == from-scratch`
   confirmed.
7. A new host or pack → its fixtures run through all four dimensions.

---

## 5. Hard NEVERs

- Never let a tree-sitter, spellbook, fst, LSP or model-runtime type escape its crate (§1.1).
- Never compute suggestions while producing diagnostics. Suggestions are computed for one word, when
  asked: a code action, or `codeAction/resolve`.
- Never wait for the context tier on any response the lexical tier can answer. Context results
  arrive later and are merged.
- Never send text off the machine. The HTTP model client refuses a non-loopback endpoint unless the
  user writes `allow_remote = true`.
- Never vendor a dictionary, word list or model whose license has not been recorded. Dictionaries
  are data under their own licenses, and `cargo deny` cannot see them.
- Never add a fallback: no regex after a failed tree-sitter parse, no accept-all after a missing
  dictionary, no "try the other pack".
- Never check an identifier at a use site. Check it where it is declared. A use of somebody else's
  API is not yours to fix, and a typo that is declared once is reported once.
- Never accept a word because the project uses it often. The project lexicon _ranks_ suggestions. It
  never _accepts_ a word — that would hide a typo repeated forty times. A word is accepted for one
  of two reasons:
  - it is in a words file;
  - the project **declares** it: its package name, its dependencies' names, its authors' names, or a
    part of an identifier it defines. A declared word is accepted everywhere _except at the
    declaration itself_, which is still checked, so a typo the project declares is reported once,
    where it was written. A known typo is never accepted this way. (On the zero corpus, declared
    vocabulary is most of codebook's noise: `02-testing.md` level 9.)
- Never put a language-specific branch, natural or host, in generic code. `if lang == "uk"` belongs
  in `packs/uk-UA/pack.toml` as data. `if host == "rust"` belongs in `hosts/rust/`.
- Never store a document's text twice, or keep parallel maps keyed by the same word. Intern words.
- Never suppress a check to make the zero corpus quiet. Find the cause, such as a token shape nobody
  recognized or a missing vocabulary, and fix that.
- Never run two phases, or two agents with overlapping write sets, at once.

---

## 6. Crate map (target — dependency direction is downward)

```
orthra-cli        `orthra check` (CI, pre-commit, exit code), `orthra explain`, `orthra dict build`,
                  `orthra pack install`, the perf and compare sweeps
orthra-lsp        protocol only: transport, document store, versions, cancellation, scheduling
orthra-project    a project's files, manifests, declarations and config: what the CLI and the
                  server both set their engine up from (§8.3)
orthra-engine     the pipeline: document -> findings, finding -> offers. The ONLY API the server sees
orthra-checks     the Check registry and every built-in check, one directory each
orthra-suggest    candidate generation and ranking, for one word at a time
orthra-context    the context tier's contract: the Scorer trait, scheduling, merging. No runtime
orthra-ml-local   an in-process model runtime (feature `ml-local`) — a leaf
orthra-ml-http    a client for an OpenAI-compatible localhost endpoint (feature `ml-http`) — a leaf
orthra-lexicon    word interning, the verdict cache, the project lexicon
orthra-dict       the Dictionary trait; the only crate that knows spellbook and fst exist
orthra-segment    spans -> words: Unicode word bounds, identifier splitting, skip patterns
orthra-hosts      where prose lives in a file; the only crate that knows tree-sitter exists
orthra-packs      natural-language packs: loading pack.toml, routing words by script
orthra-text       TextRange, LineIndex (UTF-8/16/32), Script, LangTag, ids — and nothing else
orthra-testkit  orthra-harness
```

**Naming families:**

- `*Span` — a range of prose that a host found, with its kind.
- `*Word` — a segmented token with its shape and script.
- `*Verdict` — what the packs said about a word.
- `*Finding` — something to report.
- `*Offer` — a fix for a finding.
- `*Facts` — a read-only bundle handed to one computation.
- Also: `*Host`, `*Pack`, `*Check`, `*Scorer`, `*Lexicon`, `*Id`, `*Store`.

Don't invent a new family unless nothing fits, and don't force a bad fit into an old one.

---

## 7. Code quality

lspkit's `docs/working.md` holds the rules every server keeps (imported in §2): `forbid(unsafe)`
through inherited workspace lints, `panic = "unwind"`, `pub(crate)` by default, files under ~600
lines, one feature per directory, no `utils`, tests beside the code, comments that keep their
measurements. `lspkit_arch::baseline` checks the settings among them
(`crates/tools/arch/src/baseline.rs`). What is orthra's own:

- **Prose is American English (en-US):** docs, comments, commit messages, and what the server says.
  Write `behavior`, `license`, `normalize`, `learned`. `orthra.toml` checks the repository as en-US,
  so a British spelling is a finding. A British word that is an example stays, in backticks.

- If a memory-mapped dictionary is ever picked, that single `unsafe` call lives in one leaf module
  of `orthra-dict`, with a comment saying what must not happen to the file while it is mapped, and
  that crate is the baseline's one exception. Reading the file into a `Vec` is the safe alternative,
  so measure the RSS difference first.
- Tests beside the code matter twice here: `no_fallbacks` tells request-path code from test code by
  path, so an inline test module that touches the filesystem would read as a fallback.

---

## 8. Gotchas

### 8.1 Known before day zero — not yet paid for

These are the classic spell-checker traps. None has bitten _this_ repo yet, so each must be proven
by a fixture before it is believed. Each becomes a fixture in Phase 1 or 2.

- **Escape sequences glue letters onto words.** `"Hello\nworld"` reads as `nworld`. The same goes
  for `\t`, `é`, `%s`, `{0}` and `{name}`. Segmentation must know the host's escape and format
  syntax, and that knowledge is host data.
- **Ukrainian apostrophes.** Ukrainian uses the apostrophe _inside_ a word (`м'ята`, `п'ять`), and
  people type it as U+0027, U+2019 or U+02BC. The pack normalizes all three to one form for the
  check, and the finding reports the original range. Separately, a single-quoted string in code
  (`'hello'`) has delimiters, not apostrophes — the host strips those.
- **UTF-16 positions.** LSP positions default to UTF-16 code units. A Cyrillic letter is 2 bytes and
  1 unit; an emoji is 4 bytes and 2 units. Negotiate `positionEncoding` (take UTF-8 when the client
  offers it), convert in exactly one place (`LineIndex`), and let the `shift` dimension find the
  rest.
- **Identifier shapes.** `HTTPServer`, `parseJSON`, `IOError`, `getURLsForIDs`, `utf8`, `base64`,
  `x86_64`, `i18n`, `k8s`, `SHA256`, `md5sum`. Acronym plurals and digit boundaries are where
  splitters disagree with each other. Score the splitter against typos' splitter in Phase 1.
- **A Hunspell dictionary that "loads" can still accept nothing.** Dictionaries carry their own
  encoding (`SET ISO8859-1`), flag format (`FLAG long`, `FLAG num`, `FLAG UTF-8`) and compounding
  rules. Every pack gets a smoke fixture: ten known-good words that must pass and ten known-bad
  words that must not.
- **Case.** `ukraine` vs `Ukraine`, sentence-initial capitals, ALL-CAPS words (Hunspell has rules
  for them), and Title Case headings.
- **Hyphens.** `well-known`, `будь-який`, `по-українськи`. The pack decides whether a hyphen is
  word-internal.
- **Homoglyphs.** A Cyrillic `с о а е р х і` inside a Latin word, or the other way round, passes
  neither dictionary — or, worse, looks right on screen. That is a `mixed-script` finding, not an
  `unknown-word` one.
- **Tree-sitter ABI.** A grammar crate is built for one tree-sitter ABI range. A mismatch fails when
  the language is _set_, at runtime, not at compile time. Pin the core and every grammar together,
  commit `Cargo.lock`, and give each host a test that loads its grammar.
- **Pin a grammar in the manifest, not only in the lockfile.** Observed in the incumbent on
  2026-11-05. codebook 0.3.42 declares `tree-sitter-dart = "<1"` and locks 0.2.0. A plain
  `cargo install codebook-lsp` ignores the lockfile, resolves 0.3.0 (which renamed a node type), and
  the Dart query no longer compiles. codebook compiles every host's query at startup and panics, so
  one grammar takes down Rust, Markdown and every other host along with it. Two rules follow, both
  for orthra:
  - every grammar is `=x.y.z` in `Cargo.toml`, because whoever installs from crates.io gets the
    manifest and not the lockfile;
  - a host's grammar and query load on the first document of that language. A failure is that host's
    error, reported once, and never a panic that stops the other hosts.
- **A grammar's external scanner can keep state in a `static`.** Observed 2026-11-09:
  tree-sitter-properties 0.3.0 keeps `reached_eof` process-wide, so two threads' parses corrupt each
  other. Every file was fine alone; `orthra check` over 238 of them hung five times in six, and one
  run reached 46 GB. A host's test of one file cannot see it: run `orthra check` over a directory of
  the language, several times, before trusting a grammar. Java properties is read by hand for this.
- **A grammar's incremental parse can differ from its fresh parse.** Observed 2026-11-09:
  tree-sitter-rst parsed 9 to 18 % of single edits differently from the edited text parsed alone,
  which `incremental == from-scratch` caught on half its runs. Such a host parses every version
  whole (`TreeSitterHost::parsed_whole`); never mark the dimension `#[ignore]` to ship one.
- **Markdown.** Only prose is checked: not code spans, fenced blocks, link destinations, autolinks,
  raw HTML or front matter.
- **Things that are never words:** URLs, emails, paths, UUIDs, commit hashes (`a3f9c2e`), hex
  colors, base64 blobs, semver, CLI flags (`--no-verify`). They are skip patterns, kept as ordered
  data.

### 8.2 Agents, worktrees and the build

The hazards every server shares -- a patch path per agent, the one `git stash` every worktree writes
to, `UPDATE_EXPECT` under nextest and inside a worktree, one cargo command at a time -- are in
lspkit's `docs/working.md`, imported in §2. orthra's own:

- **Pin foreign engines with `=` and commit `Cargo.lock`.** orthra is a binary, so the lockfile
  belongs in git. Tree-sitter and its grammars have to move together (§8.1).

### 8.3 Fixtures and snapshots

These are taken from phrost §8.4, keeping the ones that are not about PHP:

- **An `expect![""]` that arrives from `UPDATE_EXPECT` is a result to explain before accepting it.**
  An empty expectation is the one snapshot that can be right for the wrong reason. For a spell
  checker it is the most common snapshot of all, which makes this rule matter more here than it did
  in phrost.
- **A test whose fixture does not match its own name pins a gap instead of a rule.**
- **One subject per test, last in its fixture.** A negative control placed last pins its own
  emptiness.
- **Snapshot lines that begin with padding fight `cargo fmt` forever.** Render the first character
  of every line as content.
- **A harness that treats silence as failure scores a correct silence as a miss.** When the CLI and
  the suite disagree, suspect the suite's reading of the expectation first.
- **A format's stated safety is scoped to the language it was designed against.** A `$0` marker is
  safe in Rust and collides in shell, PHP and Perl. A fixture written in a new host language is
  outside the marker format's guarantee until a test says otherwise.
- **A `check`-style CLI must mirror the server's setup** — the same config layering, the same packs,
  the same hosts — or it reports phantom findings and you chase them for a day.

New notes go here until an area outgrows this file. Then split it into `docs/08-*.md`, as phrost
did.

---

## 9. Tooling

- Use `rg` instead of `grep`, and `fd` instead of `find`.
- `just` is the task runner: `just test`, `just lint`, `just deny`, `just arch` (the boundary checks
  alone), `just accept [crate]`, `just pr-ready`.
- Run tests with `cargo nextest run`. Accept snapshots with `UPDATE_EXPECT=1`, minding the hazards
  in lspkit's `docs/working.md`.
- **`orthra explain <file>:<line>:<col>`** is the debugging tool. It shows which span the position
  is in, which word it is part of, which packs were asked, what each one said, and which check
  fired. It is phrost's `query` for this domain. Build it in Phase 1, before you need it.
- **Five instruments, five questions** — all built in Phase 0–1:
  - `just zero` — did a change add a false positive to the zero-diagnostics corpus?
  - `orthra check <dir> --ratchet` — did the count change, _by code_?
  - `just typo-eval` — recall, and suggestion top-1/top-5, on labeled real typos.
  - `just compare <dir>` — codebook, and typos and cspell when installed, over the same files: both
    / orthra-only / other-only, and precision and recall from `corpora/compare/`'s labels.
    `just compare-zero` runs it on the zero corpus.
  - `just perf <dir>` — did it cost anything?

  Measure, let the data rank the work, build the top, then re-measure with all five.

- Commit your own work, by the gate rules in lspkit's `docs/working.md`.
