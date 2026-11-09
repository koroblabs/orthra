# Roadmap

phrost's first roadmap had eight phases with week estimates. It was stale by day six, because the
estimates were the only ordering it had and they were guesses. phrost's lesson was to order the work
by **evidence**.

On day zero there is nothing to measure, so orthra starts with phases anyway. Each phase has **exit
criteria that are measurements**, and none has a date. After Phase 5 the phases end. The roadmap
becomes a ranked list, re-derived by running the five instruments (`CLAUDE.md` §9).

**Never run two phases at once.**

---

## Where this stands

**0.1.0 is the first release.** Phases 0 to 3 are done, and Phases 4 and 5 in part; each phase's own
"where it stands" says how far.

**Phase 0 is done** (2026-11-05), apart from one input that comes from outside: the reported cases.
Its exit criteria hold: `just pr-ready` is green, `dependency_rules` fails on a planted edge, there
are fifteen incumbent cases, and the dimensions run. On top of that, three corpora have been
measured against codebook.

**Built:**

- The workspace: 15 crate skeletons, toolchain, lints, `deny.toml`, `Justfile` and CI.
- `orthra-arch`:
  - `dependency_rules` — the allowed-edges matrix and the foreign-engine table. Renamed,
    target-specific and build dependencies are caught too, and so are cycles in the rules
    themselves.
  - `no_fallbacks` — filesystem reads on the request path, each with its written reason.
  - The tables are orthra's, and the engine is `lspkit-arch`, shared with phrost and proven there on
    planted violations.
- `orthra-text`: the line index for UTF-8, UTF-16 and UTF-32, including `\r\n` and lone `\r`.
- `orthra-testkit`: the fixture format and the annotated rendering.
- `orthra-harness`: four of the five dimensions, each proven on a planted defect (`02-testing.md`
  level 4).

`just pr-ready` is green.

- `corpora/incumbent/`: **fifteen probe cases** for H1–H3, each written as the snapshot of the right
  answer, with the format checked by `orthra-testkit` (`02-testing.md` level 7). Measured against
  codebook 0.3.42, which agrees on 9 and differs on 6. H1 (noise) is partly confirmed, H2
  (suggestions) is not, and H3 (multilingual) is. See `corpora/incumbent/README.md`.

- `tools/zero-corpus.py`: phrost and ripgrep at pinned versions. Codebook makes 31.5 and 62.7
  findings per 1 000 lines on them.
- `tools/ua-gec.py`: 1 000 Ukrainian spelling errors from UA-GEC. Codebook puts the fix first in 42
  % of them.
- `tools/typo-corpus.py`: 2 000 real typos sampled from the GitHub Typo Corpus. They stay local,
  because of the corpus's licensing. Codebook's bar on them is 84–90 % flagged and 59–63 % top-1.

**Missing:**

- **The reported cases.** These come from the people who called codebook not good enough.
- **A JetBrains number.** RustRover's headless inspector reports no typos at all, even obvious ones:
  RustRover 2026.3's `inspect.sh`, with only the Typo inspection enabled, said nothing about
  `Teh recieved value is definately wrong.` in a `.txt` (2026-11-05), likely because the dictionary
  loads after the batch run ends. So JetBrains stays a bar for qualities: real declarations, rename
  as the fix, quiet by default. To get a number, somebody would have to read the 15 cases in the IDE
  by hand.

---

## Phase 0 — Instruments before features

Nothing in this phase checks a word. It builds everything that will judge the words later.

- Workspace, the crate skeletons, `rust-toolchain.toml` (the same pin as phrost), lints,
  `deny.toml`, `Justfile` and CI (`fmt`, `clippy -D warnings`, `nextest`, `deny`).
- `dependency_rules` with the allowed-edges matrix and the foreign-engine table (`CLAUDE.md` §1.1).
- `orthra-testkit`: the fixture format and the annotated-text renderer (`02-testing.md` level 0).
- `orthra-harness`: the four dimensions. Each one runs the identity until there is something to run.
- **Corpora, gathered and converted.** This is haiku work, run in parallel:
  - **the incumbent's failures** — ask the people who said codebook is not good enough for ten
    concrete cases (a file, a word, what was wrong), and turn each one into a failing fixture in
    `corpora/incumbent/`;
  - a GitHub Typo Corpus sample as fixtures;
  - zero-corpus picks with their words files;
  - a Ukrainian tech-prose sample.

**Exit:**

- `just pr-ready` is green.
- `dependency_rules` fails on a deliberately planted bad edge.
- At least 10 incumbent fixtures exist and fail.
- The four dimensions run on an empty pipeline.

---

## Phase 1 — The lexical core, no server

- `orthra-text`, `orthra-segment` (UAX #29 bounds, the identifier splitter, skip patterns), and
  `orthra-dict` with **both** `HunspellDict` and `FstDict`.
- `orthra-lexicon` (interning, the verdict cache) and `orthra-checks` with `unknown-word` and
  `known-typo`.
- The `PlainText` and `Markdown` hosts.
- `orthra check` and `orthra explain` on the CLI.
- **The dictionary decision.** Measure the table in `01-architecture.md` §7 on en-US, write the
  result into that section, and pick the default for English. **English first** (decided
  2026-11-05): Ukrainian waits for Phase 4, and its engine is chosen there, per pack.

**Exit:**

- `CLAUDE.md` §3 holds measured baselines instead of targets.
- `just typo-eval` reports recall, top-1 and top-5.
- `just zero` reports false positives per 10k words on the Markdown part of the zero corpus.
- The dictionary decision is recorded with its numbers.

**Where it stands (2026-11-05): the exit criteria hold, with one budget over.**

- The baselines are in `CLAUDE.md` §3, and `orthra perf` measures them. The warm check is 2.3 ms
  against 2 ms.
- `just typo-orthra` puts the fix first in 72.8 % / 67.6 % of code / prose typos, against codebook's
  63.3 / 59.1, and in the first five as often as codebook (79.0 / 72.7 against 79.1 / 73.2).
- `just zero-orthra` measures the Markdown, and it is **noisier than codebook** (193 and 179
  findings per 1 000 lines against 142 and 163). The causes are measured, and Phase 2 starts with
  them.
- The dictionary is decided: spellbook, edit suggestions only, and no `FstDict` for English
  (`01-architecture.md` §7). Revised 2026-11-09: English is compiled too, for memory.
- `orthra check`, `orthra explain` and `orthra perf` exist.

---

## Phase 2 — Hosts

**Done (2026-11-05).** Every exit below is met. **Where it stands:**

- **Hosts:** Rust, TypeScript, TSX, Python, Go, PHP, TOML, YAML and JSON on tree-sitter. They are
  pinned with `=`, their grammars load on first use, and a failure stays with its host. Each names
  its own test files and doc tags as data.
- **Fourteen more (2026-11-08):** JavaScript, shell, HTML, CSS, SCSS, LESS, SQL, Lua, Java, Kotlin,
  Swift, C, C++ and Ruby, each one agent's directory, measured quiet on two to four well-kept
  projects of its language. SCSS's and LESS's grammars predate the `LANGUAGE` constant and load
  through `LanguageSource::Function`; HTML cuts a template's code out of its text.
- **Thirty-one more (2026-11-09):** Vue, Svelte, Scala, Groovy, C#, F#, Dart, Zig, Objective-C,
  Haskell, OCaml, Elm, Elixir, Erlang, R, Julia, PowerShell, Make, CMake, Dockerfile, Protocol
  Buffers, GraphQL, XML, reStructuredText, Nix, HCL, INI and Java properties, in two waves of agents
  with disjoint directories. Java properties is read by hand, since its grammar's scanner is not
  thread-safe, and reStructuredText parses whole, since its grammar's incremental parse is not its
  fresh one (`CLAUDE.md` §8.1). The doc reader learned doctests, Haddock's bird tracks, odoc's
  references, Scaladoc's fences and indented blocks in docstrings, and the engine handles in
  `TODO(name)`, Android's, Apple's, .NET's and Java's translations by path, and qualified names.
- **Every English incumbent case passes** (13 of 13). The two Ukrainian cases wait for Phase 4.
- **The quiet exit:** on every `.rs` and `.md` file, before any words file, phrost is at **2.5**
  findings per 1 000 lines (target ≤ 3, met) and ripgrep at **4.6** (target ≤ 6, met). What ripgrep
  has left is its own lexicon: `ugrep`, `pdftotext`, `submatch`. On composer's PHP and JSON, orthra
  makes **3.2** against codebook's 50.4 (target ≤ 5, met). On helix's TOML, **5.9** against 87.3
  (target ≤ 8.7, met). On the starter workflows' YAML and JSON, **17.8** against 206.4 (target ≤
  20.6, met).
- **Open question 7 is decided by measurement.** Code is held to known typos only. It costs 4.7
  points of code recall on the typo sample and buys most of the 12×; the rule stays.
- **`host-agnostic prose` is green:** one body in 22 comment syntaxes, the same findings as plain
  text.
- **`just zero` ratchets:** each project's findings by code, against a baseline committed in
  `corpora/zero/` that only goes down.
- **Markdown stays on pulldown-cmark,** reassessed against tree-sitter-md: the same prose to within
  0.1 %, at a fortieth of the cost (`01-architecture.md` §3).

- The generic `TreeSitterHost`.
- Hosts: Rust, Markdown (reassessed against tree-sitter-md), Git commit, TOML, JSON, YAML, Python,
  TypeScript/JavaScript, Go, and PHP (phrost's users).
  - Each is one directory and one agent. They are disjoint by construction, so they run in parallel.
- Declarations-only identifiers, escape and format syntax per host, and technical vocabularies per
  host plus one general software-terms list (license recorded).

**Exit:**

- `just zero` is at or below a threshold set from the Phase 1 measurement, and that threshold only
  ever ratchets down.
- **Before any words file, at least ten times quieter than codebook on the same files:** at most 3
  findings per 1 000 lines on phrost, 6 on ripgrep, 5 on composer, 8.7 on helix's TOML and 20.6 on
  the starter workflows, where codebook makes 31.5, 62.7, 50.4, 87.3 and 206.4.
- `host-agnostic prose` is green.
- The incumbent fixtures about noise (H1) pass.

---

## Phase 3 — The server and Zed

**Where it stands (2026-11-05):**

- **`orthra serve` answers in the editor's terms.** It speaks UTF-8 when the client offers it and
  UTF-16 otherwise. Diagnostics are pushed, to every client: pull made Zed's list of problems empty
  and refill in a loop. Sync is incremental. It runs behind lspkit's panic containment and buffered
  transport.
- **Nothing is reported until the project is learned,** and then everything open at once. A change
  waits for 300 ms of quiet.
- **The command line and the server set up a project in one place,** `orthra-project`.
- **`orthra-zed` runs it in Zed** (`github.com/koroblabs/orthra-zed`): `orthra serve` from
  `lsp.orthra.binary.path`, from `PATH`, or else from the latest GitHub release for the platform,
  downloaded once; for every language with a host, with `lsp.orthra.initialization_options` passed
  through.
- **A tag `v*` makes a release** (`.github/workflows/release.yml`): macOS on Apple silicon and
  Intel, Linux on x86_64 and arm64 (glibc 2.35 or later), each archive with the licenses of what is
  in it: the packs' data and every crate linked in (`about.toml`).
- **The configuration layers are read** by the command line and the server alike: `orthra.toml`
  (`languages`, `ignore`), `.orthra/words.txt`, and the user's `~/.config/orthra/`. orthra checks
  itself with them: 570 findings on its own repository fell to 25, every one a typo its docs quote
  on purpose.
- **A project is what it keeps in git:** the walk honors `.gitignore`. A Drupal project with
  `vendor/` and Drupal core outside git started in 25.2 s at 165 MB, and now in 1.3 s at 35 MB.
- **Code actions and watching are built:** change to a fix or a suggestion, add to the project's
  words or the user's, ignore in this file; and a settings or words file that changes is a new pass
  over the project.
- **The known typos are not copied:** an offset per entry into the embedded list, searched by binary
  search. `initialize` went from 22.8 MB to 10.7 MB, and a server after a pass over phrost from 34
  MB to 25 MB, with the check as fast as before.
- **A keystroke is parsed, queried and segmented again only where it changed,** from what the server
  keeps for the document being typed in: per 2 000 lines, phrost 5.0 → 1.8 ms, ripgrep 6.1 → 1.5,
  composer 7.7 → 1.3, helix 2.6 → 0.7, starter workflows 5.8 → 3.3. `just perf` measures a keystroke
  now, not a second whole check, and its budget is the p95 of one keystroke per file: 0.14 to 1.10
  ms on the five projects, against 2 ms. The slowest are 3.5 to 5.2 ms, in files of 4 000 lines or
  more, where the verdicts still run over every word.
- **`just compare` is built,** and on the zero corpus orthra is ahead of codebook on precision (1.80
  % against 0.16 %, on 2 481 findings against 27 504) at equal recall of the real typos either found
  (`02-testing.md` level 7). On the typo sample codebook still flags 3 to 7 points more.

- `orthra-lsp`: tower-lsp-server, the document store, incremental re-check, pushed diagnostics,
  cancellation, and config layering with file watching.
- Code actions:
  - replace with a suggestion (computed in `codeAction/resolve`);
  - add to project words;
  - add to user words;
  - ignore in this file.
- `orthra-zed`: an extension repo alongside phrost's `phrost-zed`. It downloads the release binary
  for the platform and maps Zed language names to language ids.
- `just compare` against codebook, typos and cspell, with the hand-labeled samples.
- **Not built: a way in from cspell.** Read `words` and `ignoreWords` from a project's
  `cspell.json`, `.cspell.json` or `cspell.config.yaml` as its words, so a project moving from
  cspell or VS Code's Code Spell Checker keeps what it taught it.

**Exit:**

- The flow tests (level 6) are green.
- `just perf` meets every budget.
- **Every English incumbent fixture passes.** The two Ukrainian cases, 06 and 08, belong to Phase
  4's exit.
- `just compare` shows orthra ahead of codebook on precision at equal or better recall, on the same
  files.
- The author uses it daily on phrost.

---

## Phase 4 — Many languages

- The uk-UA pack from dict_uk, with its license recorded.
- Routing by script, apostrophe normalization, hyphen rules, and the `mixed-script` check.
- **Packs are downloaded, not built in, past English** (agreed with the user, 2026-11-06):
  - en-US and en-GB stay in the binary, so a fresh install checks with nothing to set up and no
    network. A built-in pack nobody enables costs disk, not memory: its pages are never touched.
  - Every other pack, and each optional part of one, is installed into
    `$XDG_DATA_HOME/orthra/packs/<tag>/<version>/`, pinned by version and checked by SHA-256:
    - by `orthra pack install <tag>`, or `--from` an archive for a machine with no network;
    - by `orthra check`, for a language its settings enable;
    - by the server, in the background with a progress bar, which checks the project again once the
      pack is in (user's choice, 2026-11-06, over the extension doing it: it works in every editor
      and needs no restart).
  - **Built (2026-11-06)** for uk-UA, from dict_uk's own release: GPL-3.0, so orthra does not carry
    it. `packs/uk-UA/` holds `pack.toml` and `install.toml`, and no dictionary.
  - Only `orthra-install` has network code. It fetches the archive a pack's `install.toml` pins, by
    the pack's tag, and never sees a document. A project's `orthra.toml` cannot make it fetch
    anything else.
  - `auto-install = false` keeps a machine off the network, and then the server and `orthra check`
    say how to install the pack instead. An install that fails is said once a session.
  - **Updates are explicit** (`orthra pack update`), and the installed version is recorded. A pack
    that refreshed itself would change findings with nothing changed on the user's side, which
    breaks §1.2 and every ratchet in CI.
  - An installed pack is read from its file into memory. A built-in one is read where it is
    embedded, which is why the 2 MB of known typos cost no heap. Measure an installed pack's RSS
    against the built-in one before choosing a format.
- **The dictionary's size is a setting:** `dictionary = "small" | "standard" | "large"` in
  `orthra.toml`, project-level, because it changes findings and the team and CI must see the same
  ones.
  - `small` is SCOWL size 60 alone.
  - `standard`, the default, adds size 70's rare words: 13.0 MB at `initialize` against 10.6, and 2
    248 zero-corpus findings against 2 481 (`01-architecture.md` §7).
  - `large` is to be measured.
  - Each size beyond the smallest is an installable part of the pack, so `small` never downloads the
    rest. The default is chosen by measurement, since most people never change it.
- `orthra pack install` with checksums.
- Suggestions ranked by the project lexicon, casing restored, and `inconsistent-spelling`.
- **Suggestions ranked by how often a word is written** (built 2026-11-06): a pack may name a
  `frequency` file, and its dictionary's corrections are ordered by it among those as near. uk-UA
  installs FrequencyWords' list as a second source of its `install.toml`. On UA-GEC this took top-1
  from 41.7 % to 47.1 % (`02-testing.md` level 9).
- **uk-UA compiled to an FST** (built 2026-11-06, `01-architecture.md` §7): 3.9 MB where spellbook
  held 37 MB, the same findings, a Ukrainian word's first check in 0.02 s against 0.14, and
  corrections within two edits. UA-GEC top-1 49.1 %, top-5 67.4 %.
- **The acceptance test of the pack seam, run for real:** a haiku agent adds pl-PL with no Rust
  diff. If it needs one, the seam is wrong, and fixing the seam is the next task.

**Exit:**

- UA-GEC spelling recall and top-k beat codebook's bar. On 2026-11-05 codebook flagged 78 % of the
  non-word errors and put the fix first for 42 % (`02-testing.md` level 9). **Met on 2026-11-06:**
  orthra flags 81.3 % and puts the fix first for 49.1 %, and in the first five for 67.4 % against
  63.8 %.
- The Ukrainian incumbent cases, 06 and 08, pass.
- The mixed-language fixtures pass.
- pl-PL landed as data only.

---

## Phase 5 — The context tier

This phase is fully specified in `04-context.md`. The order is by value per unit of cost:

1. The `Scorer` contract, `FakeScorer`, scheduling and merging, and the `context-off ⊆ context-on`
   dimension made real. **Built (2026-11-06) for re-ranking:** the server reorders an unknown word's
   corrections through a `Scorer` when somebody asks for the actions, within 80 ms and off its lock,
   and keeps the lexical order otherwise (`04-context.md` §2 A). A flow test holds the findings to
   the lexical tier's with the adversarial `FakeScorer` on.
2. **Re-ranking** suggestions in `textDocument/codeAction`. On the fifteen easy probes, codebook's
   first suggestion was always right. On 2 000 real typos it is right 59–63 % of the time. That is
   the bar: orthra's lexical ranking (project words, keyboard distance, frequency) has to beat it
   first, and the context tier has to add something on top of that, or it is not built.
3. **Confusables** in prose, in idle time.
4. **Rewrite sentence**, an explicit action through a localhost endpoint.
5. In-process runtime spike: candle vs ort on the same model. Compare latency, binary size and
   distribution cost, then pick one. **candle, measured (2026-11-06):** ELECTRA-small in Q8_0 is 15
   MB and +2.5 points of English top-1 at 17 ms p95 on 4 CPU threads, with no `unsafe` and no native
   library (`04-context.md` §4). ort was not needed to decide.

**Settings** (agreed with the user, 2026-11-06):

- The model is a user-level switch, in `~/.config/orthra` or the editor's settings, and off by
  default: `[context] enabled = true, model = "..."`. Whether to spend a machine's memory on a model
  is that machine's question, not the project's.
- Its findings only ever add, under `source = "orthra/context"`, so a project's CI never depends on
  it.
- A model is installed only by an explicit `orthra model install`, checked by SHA-256, and unloaded
  after `idle-unload`.

**Exit:**

- Re-ranking improves top-1 on the GitHub Typo Corpus by a margin worth its latency, at p95 ≤ the
  budget.
- `context-off ⊆ context-on` is green.
- RSS returns to the lexical baseline after the idle unload.

---

## After Phase 5

The phases end here. The next task is the top of a list ranked by the instruments:

1. what `just zero` still reports;
2. what `just compare` shows others catching that orthra misses;
3. what `just typo-eval` misses whole;
4. which budgets `just perf` shows close to the edge.

---

## Open questions — each settled by a measurement or a user, not an argument

1. **spellbook vs fst as the default dictionary.** Phase 1, by the §7 table. Settled: compiled for
   uk-UA (2026-11-06) and English (2026-11-09); spellbook stays for a pack whose `.aff` a list of
   forms cannot hold.
2. **What a fix means for an identifier.** "Replace in file", or "name the fix and let the
   language's server rename it". JetBrains does a real rename, with every use, and that is the
   target. orthra has no semantic model, so the question is how close an LSP server can get, for
   example by handing the client a rename request. Ask users in Phase 3.
3. **What counts as a finding inside an identifier.** In prose, an unknown word is a finding. In an
   identifier, `rdr`, `wtr` and `fnid` are abbreviations the author chose. The candidate rule is to
   report a known typo, or a near miss of a dictionary word, and stay quiet about an unknown word.
   Decide it in Phase 2, on the zero corpus and the typo sample together: one rule has to be quiet
   on the first and still catch the second. **Decided (2026-11-05): known typos only in code.** It
   costs 4.7 points of code recall on the typo sample (88.0 → 83.3 %). With the other quiet rules it
   takes the zero corpus's `.rs` noise from codebook's 26.4 and 53.0 per 1 000 lines to 1.6 and 5.7.
4. **Default severity of `unknown-word`.** Information or hint? Zed renders them differently. Ask
   users. **Decided by the first user (2026-11-05): a warning, for every finding.** Zed lists only
   errors and warnings among a project's problems (`diagnostics.rs`, `max_severity`), and an
   information never reached the list. `severity` in the settings lowers it.
5. **A project-wide background sweep**, like phrost's. `inconsistent-spelling` and the project
   lexicon want it, and the memory rule is against it. Measure the RSS, then decide on-or-off by
   default. **Decided for declarations (2026-11-05): on.** The server learns what every file
   declares at startup, off the request path, reading each file and dropping it: phrost in 0.95 s,
   helix 0.52 s, composer 0.35 s, ripgrep 0.23 s, and RSS after it 29 to 35 MB. **And for findings
   (2026-11-05): on, published and not kept.** Every file is checked after the project is learned
   and the closed files' findings published, so the editor lists every problem: phrost's 345 files
   with problems by 2.2 s, RSS 35 MB. The open file is answered first, at 1.3 s.
6. **candle vs ort** for the in-process runtime. Phase 5.
7. **What the Zed extension ships:** the lexical-only binary with models as a separate download, or
   one binary. Decide from measured sizes.

---

## Refused, and why

| refused                                            | why                                                                                                                                                                                         |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Grammar in the lexical tier                        | It is a different product. Harper and LanguageTool do it well and run alongside. Grammar reaches orthra only as the explicit "rewrite sentence" action                                      |
| Checking identifiers at use sites                  | Not the user's to fix, and it reports one typo fifty times                                                                                                                                  |
| Accepting a word because the project uses it often | It hides a typo repeated everywhere. The lexicon ranks. A word is accepted only from a words file or because the project declares it (`CLAUDE.md` §5), and its declaration is still checked |
| Quieting the word under the cursor while typing    | It makes findings depend on edit history, which breaks §1.2. The 300 ms quiet before a change is published already covers it                                                                |
| Word completion from dictionaries                  | In code it is noise in the completion menu, next to the language server's real items                                                                                                        |
| Bundled models, or remote APIs by default          | Binary size, and privacy. Local and opt-in only (`04-context.md`)                                                                                                                           |
| A regex fallback after a tree-sitter failure       | Tree-sitter is error-tolerant, so a fallback would only run on bugs, and it would hide them                                                                                                 |
| Runtime-loaded grammars (WASM tree-sitter)         | One more ABI to pin before anyone has asked for a language we don't ship. Revisit with a user and a measurement                                                                             |
| CJK and Thai word segmentation                     | Needs dictionary-based segmentation. Not until someone brings a corpus and a need                                                                                                           |
