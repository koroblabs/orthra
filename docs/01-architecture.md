# Architecture

This document covers how a document becomes findings, where the crate boundaries run, and why each
boundary sits where it does. Nothing here is built yet. Where a decision waits on a measurement, the
document says which measurement, and `03-roadmap.md` says when it is taken.

The rule everything is judged by is in `CLAUDE.md`: **fast, resource-cheap, and quiet**.

---

## 1. The pipeline

```
text ─► Host ─────────► Span { range, kind }        where does prose live in this file?
     ─► Segment ──────► Word { range, shape, script } what are the words?
     ─► Route ────────► Word + the packs for its script
     ─► Checks ───────► Finding { range, code, data }  what is wrong?
     ─► LSP ──────────► Diagnostic                     UTF-16 / UTF-8 positions, one place

on request (code action / resolve), for ONE finding:
        Finding ─► Suggest ─► [context re-rank] ─► Offer ─► CodeAction
```

Two properties of this shape matter more than any single stage.

- **Diagnostics and suggestions are different paths.** Suggesting is the expensive operation:
  Hunspell-style suggestion costs milliseconds per word, and a check costs microseconds. Diagnostics
  never suggest. A user who looks at one finding pays for one word. The diagnostic carries what the
  code action needs (`Finding::data`), so the action does not re-run the pipeline.
- **Every stage is a pure function of the one before it plus configuration.** That is what makes
  `incremental == from-scratch` checkable at each stage rather than only at the end (`02-testing.md`
  level 1).

---

## 2. The crate graph

Dependencies point downward. Nothing reaches back up, and the matrix is a test
(`crates/tools/arch/src/dependency_rules.rs`).

```
orthra-cli        check / explain / dict build / pack install / perf / compare
orthra-lsp        protocol only: transport, documents, versions, cancellation, scheduling
orthra-project    a project's files, manifests, declarations and config; what the CLI and the
                  server both set their engine up from, so `orthra check` reports what the editor would
orthra-engine     document -> findings, finding -> offers; the ONLY API the server sees
orthra-checks     the Check registry; one directory per check
orthra-suggest    candidates for one word, ranked
orthra-context    the Scorer and Rewriter contracts, the sentence   (no runtime)
orthra-ml-local   in-process model runtime                          (feature, leaf)
orthra-ml-http    OpenAI-compatible localhost client                (feature, leaf)
orthra-lexicon    interning, verdict cache, project lexicon
orthra-dict       Dictionary trait; spellbook / fst live here and only here
orthra-segment    spans -> words
orthra-hosts      file -> spans; tree-sitter lives here and only here
orthra-packs      natural-language packs: data loading, script routing
orthra-text       TextRange, LineIndex, Script, LangTag, ids — nothing else
orthra-testkit  orthra-harness
```

`orthra-engine` plays the role `phrost-view` plays in phrost. The server cannot reach a host, a
dictionary or a cache directly; it asks the engine and gets DTOs back. That keeps protocol code free
of spelling semantics, and the inverse holds too: no crate below `orthra-lsp` names an LSP type.

The model runtimes are **leaves behind features**. A build without `ml-local` and `ml-http` is the
complete lexical server. The default build has both, because neither runs unless the editor's
settings ask for it. `ml-local` is 2.8 MB of a 27 MB binary. `ml-http` adds no dependency, because
ureq is in the binary already for installing packs. The Zed extension may ship either build; that is
a distribution decision, made in Phase 5 from measured binary sizes.

---

## 3. The host seam — where prose lives

"Language" means two different things in this project, and conflating them is the first design bug a
spell checker makes. A **host** is a programming or markup language: Rust, Markdown, a git commit
message. A **pack** is a natural language: en-US, uk-UA. They are orthogonal: a Rust file contains
English and Ukrainian, and Ukrainian appears in Rust and in Markdown. So they get two separate
seams.

```rust
pub trait Host: Sync {
    /// LSP language ids this host answers for ("rust"; "markdown"; "git-commit").
    fn language_ids(&self) -> &'static [&'static str];

    /// Facts about the host's text, not behavior: escape syntax, format placeholders,
    /// comment decorations to strip (`///`, `//!`, ` * `, `#`), directive comments to skip
    /// (`// eslint-disable`, `# type: ignore`), identifier conventions.
    fn conventions(&self) -> &'static Conventions;

    /// Words this host makes legal everywhere it is used: `impl`, `usize`, `std`, `fn`.
    fn vocabulary(&self) -> &'static [&'static str];

    /// Per-document state, so a host can parse incrementally.
    fn open(&self, text: &str) -> Box<dyn HostDoc>;
}

pub trait HostDoc: Send {
    fn edit(&mut self, edit: &TextEdit, new_text: &str);
    fn spans(&self, out: &mut Vec<Span>);
}

pub struct Span { pub range: TextRange, pub kind: SpanKind }
pub enum SpanKind { Comment, DocComment, String, Declaration(DeclKind), Prose }
```

**A tree-sitter host is data.** It consists of a grammar, a `spell.scm` query, `Conventions` and a
vocabulary. One generic `TreeSitterHost` runs them all, using captures such as `@comment`,
`@comment.doc`, `@string`, `@decl.function`, `@decl.type`, `@decl.variable` and `@decl.field`.
Incremental parsing comes for free from tree-sitter's `InputEdit`.

**Hosts that are not tree-sitter** are a handful of hand-written ones:

- `PlainText`: the whole document is prose.
- `Markdown`: pulldown-cmark, measured against tree-sitter-md 0.5.4 on the zero corpus's Markdown
  (76 files, 26 383 lines; 2026-11-05) and kept. The two extract the same prose to within 0.1 %, the
  difference being tree-sitter-md reading link reference labels as text. pulldown-cmark does the
  whole job in 0.46 ms per 2 000 lines; tree-sitter-md's block parse alone takes 20 ms, and 54 ms
  with its inline grammar, and its one advantage, incremental reparsing, has nothing to win back
  from a full parse that cheap. Inline code is a value, held to known typos as between backticks in
  a comment.
- `GitCommit`: lines starting with `#` and trailers such as `Signed-off-by:` are skipped.
- `LineComments { marker }`: for simple config formats.

Which host handles a language id comes from a table, decided once. An unknown language id is not
checked. That is deterministic and quiet. The config can map a language id to an existing host
(`[hosts] nix = "line-comments:#"`).

**A tree-sitter host's captures** (`crates/hosts/src/tree_sitter_host.rs`):

- `@comment`, `@doc`, `@string` and `@declaration` are spans;
- `@value` is code that is not a declaration, held to known typos: YAML's `run:` and `script:`;
- `@skip` claims a range for nothing: TS module paths, PHP's `require` paths and shell commands;
- `@test` marks test code whose strings are data: Rust's `#[cfg(test)] mod` and `#[test] fn`.

A data format's grammar can say that a string of one token is a value (`SpanKind::Value`): a name, a
version or a path, held to known typos like code. TOML's `"derive"` and `"source.ocaml"` are values,
and its text has spaces. Code hosts leave it off until it is measured for them: there, a one-word
string can be a label someone reads.

Doc comments are Markdown, and their fenced examples are dropped. Their tags are the host's data
(`DocTags`): PHPDoc's `@author`, `@since` and `@see` lines are not prose, and the type after
`@param`, `@return` or `@var` is code, even when its brackets run over several lines
(`array{dev: int, ...}`). A host also has a `vocabulary.txt` beside its `spell.scm`: the words of
the language (`async`, `usize`, `serde`), accepted in its files. A file on a test path has its
strings left out the same way: a `tests/` or `test/` directory in any language, and beyond that
whatever the host names as data (`*_test.rs`, `*.spec.ts`, `testdata/`). A file a tool writes, such
as `package-lock.json`, is named by its host as generated, and no host claims it.

**Identifiers are checked only at declarations** (`Declaration(_)`). A use of `serde_json::from_str`
is not yours to fix. A typo that is declared once is reported once, not at every one of its fifty
uses.

**The acceptance test for this seam:** adding Go should take a new directory, `hosts/go/` (with
`spell.scm`, `conventions.rs` and fixtures), plus one line in the registry, one grammar dependency
and one line in the Zed extension's language list. Nothing under `segment/`, `checks/`, `suggest/`
or `engine/` should change.

---

## 4. The pack seam — natural languages

**A pack is a directory of data, not code:**

```
packs/uk-UA/
  pack.toml
  uk_UA.aff, uk_UA.dic      (or uk_UA.fst — see §7)
  typos.tsv                 optional: known misspelling <TAB> correction
  confusables.tsv           optional: real-word confusion sets, for the context tier
  LICENSE                   required — a pack without one does not load
```

```toml
tag = "uk-UA"
script = "Cyrl"
dictionary = { hunspell = "uk_UA" }
vocabulary = ["vocabulary.txt"]             # accepted whatever the case, beside the dictionary
word-internal = ["'", "’", "ʼ", "-"]          # characters that do not end a word
normalize = { "'" = "ʼ", "’" = "ʼ" }          # checked in one form, reported in the original
name-labels = ["автор", "автори"]            # `# Автор: …` starts a comment line of names
```

A pack names its files relative to its own directory, and may name a neighbor's: en-GB's
`typos = "../en-US/typos.tsv"` shares English's known typos and vocabulary rather than copying 2 MB
of them.

Packs are found in three places: built in (every directory under `packs/`, embedded by
`orthra-packs`'s build script, so a fresh install works with zero setup; en-US and en-GB today),
`$XDG_DATA_HOME/orthra/packs/` (installed with `orthra pack install uk-UA`, verified by checksum),
and `.orthra/packs/` in a project.

**The acceptance test for this seam:** adding Polish should take a directory and fixtures, with no
change to any Rust code. A haiku agent runs this test for real in Phase 4. **en-GB passed it on
2026-11-05:** `tools/packs.py en-GB` wrote a directory, and no Rust line names it.

### Routing by script

A word's script comes from the Unicode Script property of its letters: Latin, Cyrillic, Greek.
Letters of those three stay in one word when they meet, because they look alike (`сount`); any other
script ends a word where it meets them, as Chinese is written against Latin with no space. The word
is checked by the **enabled packs for that script**, and it is accepted if **any** of them accepts
it.

- `languages = ["en-US", "uk-UA"]`: a Ukrainian comment over an English identifier needs no
  switching. Each word goes to the dictionary that can judge it.
- A word in a script no enabled pack covers is not checked. Greek letters in a math comment are not
  flagged. Silence here is deterministic and correct.
- A word whose letters span two scripts goes to the `mixed-script` check, never to a dictionary.

Packs load lazily, on the first word of their script. An enabled pack whose script never appears
costs nothing.

---

## 5. Checks, and why projections are the unit of cost

phrost learned this the hard way, so it is designed in from the start here. A check never walks the
document. It reads **projections** that the engine computes lazily, once per document revision:

```rust
pub trait Check: Sync {
    fn code(&self) -> CheckCode;          // "unknown-word", "known-typo", ...
    fn run(&self, facts: &DocFacts<'_>, out: &mut Vec<Finding>);
}
// DocFacts: words(), prose_words(), declarations(), sentences(), verdicts() — each memoised.
```

The cost of a pass is **the number of distinct projections, not the number of checks**. A new check
that reuses `words()` and `verdicts()` costs almost nothing. A check that needs a new projection has
to say so in its commit and report a measured delta.

**The built-in checks:**

| code                    | fires when                                              | confidence                           | default severity |
| ----------------------- | ------------------------------------------------------- | ------------------------------------ | ---------------- |
| `known-typo`            | a pack's `typos.tsv` lists the word                     | high; the fix is exact and preferred | warning          |
| `unknown-word`          | no enabled pack for the word's script accepts it        | medium                               | information      |
| `mixed-script`          | the word mixes letters from two scripts                 | high                                 | warning          |
| `repeated-word`         | "the the", in prose spans only                          | high                                 | information      |
| `inconsistent-spelling` | the project writes one word two ways (`colour`/`color`) | medium; needs the lexicon            | hint             |
| `confusable`            | the context tier only (`04-context.md`)                 | an opinion; separate `source`        | hint             |

**One finding per site.** When two checks claim the same range, the first one in the registry's
order wins. `known-typo` comes before `unknown-word`, so a known typo is never reported twice. That
order is data in the registry, not a `match`.

---

## 6. The lexicon and the verdict cache

Words repeat. A 2 000-line file has thousands of tokens and a few hundred distinct words. So:

- **Interning:** a word is a `WordId`. Text is stored once.
- **Verdict cache:** `FxHashMap<(WordId, PackSetEpoch), Verdict>`. When a config, words-file or pack
  change bumps the epoch, the whole cache is invalidated at once, so a stale verdict cannot survive
  an edit to the word list. That is the bug class `cold == warm` exists to catch. The hit rate is a
  number Phase 1 measures; we expect > 90 % but have not measured it.
- **Project lexicon:** hashed word counts from declarations in the workspace. Like phrost's
  `WordSet`, it is memory proportional to distinct words and never to text. **By frequency, it ranks
  and never accepts** (`CLAUDE.md` §5). Accepting would hide a typo the project repeated everywhere.
- **Declared vocabulary:** what the project _declares_ is accepted everywhere except at the
  declaration itself:
  - the package name, from `Cargo.toml`, `package.json` or `composer.json`;
  - its dependencies' names;
  - its authors' names, which a license header repeats in every file;
  - the parts of every identifier it defines.

  The declaration is still checked, so `recieveMessage` is reported once, where it was defined, and
  not again in each comment that mentions it. A word on a pack's known-typo list is never accepted
  this way. The rule exists because the zero corpus measured it: the project's own name,
  dependencies and identifiers are most of codebook's 10 108 findings on phrost (`02-testing.md`
  level 9).

- **Possessives:** `laravel's` is checked as `laravel`. Without that, every unknown name is reported
  twice, and it is 13 % of codebook's findings on phrost.

---

## 7. The dictionary boundary

```rust
pub trait Dictionary: Send + Sync {
    fn check(&self, word: &str) -> bool;
    fn suggest(&self, word: &str, limit: usize, out: &mut Vec<Candidate>);
}
```

There are two candidate implementations. **Phase 1 decides the default by measurement**, and the
result is written here.

|         | `HunspellDict` (spellbook) | `FstDict` (compiled)                                                   |
| ------- | -------------------------- | ---------------------------------------------------------------------- |
| input   | any `.aff`/`.dic` as-is    | built by `orthra dict build` from a Hunspell dictionary or a word list |
| check   | affix rules at lookup time | a set membership test                                                  |
| suggest | Nuspell's algorithm        | Levenshtein automaton ∩ FST, ranked by frequency                       |
| load    | parse at startup           | read a file, or mmap it                                                |
| risk    | alpha, by its own README   | expanding a morphology-rich language may be large                      |

**Decided for en-US, 2026-11-05: `HunspellDict`, with edit suggestions only.** Measured on spellbook
alone, because every budget held without the second implementation:

- load 17–25 ms;
- RSS 25 MB;
- the whole check 2.6 ms per 2 000 lines;
- suggestions 0.52 ms p95 once spellbook's n-gram pass is off (25 ms with it, for about one point of
  top-1; the table is on `HunspellDict::suggest`).

**Decided for en-US, 2026-11-06: SCOWL size 60, and size 70's additions as rare words.** A pack's
`dictionary = { hunspell = "en_US", rare = "en_US-rare" }` reads a second `.dic` with the same
affixes: the 34 304 lines SCOWL's `en_US-large` (size 70) has and size 60 does not. A rare word is
accepted only where no pack lists it as a known typo. SCOWL's README warns that its larger sizes
hold uncommon words that are common misspellings (`ort`, `calender`), and 137 of crate-ci/typos'
known typos were among them (`incudes`, `algin`, `diaster`). Measured against size 60 alone, and
against size 70 used as the main dictionary:

|                                  |       size 60 |       size 70 |     60 + rare |
| -------------------------------- | ------------: | ------------: | ------------: |
| zero corpus findings             |         2 481 |         2 244 |         2 248 |
| typo sample recall, code / prose | 83.3 / 81.4 % | 83.1 / 80.8 % | 83.1 / 81.0 % |
| real-word typos flagged, code    |        10.8 % |         6.8 % |         9.5 % |
| live heap after `initialize`     |       2.85 MB |       4.93 MB |       4.96 MB |
| pack load                        |         14 ms |         16 ms |         16 ms |

What the rare words silence is real English the smaller list lacks: plurals it does not inflect
(`integrations`, `customizations`, `parsers`), derived forms (`vanishingly`, `reproducibly`,
`decrypting`), and the rare words of a novel's text. Suggestions still come from size 60 alone.

`FstDict` is not built for English, then, because nothing measured asks for it. Two things could
still make it worth building: Ukrainian (Phase 4), and spellbook's MPL-2.0, which `deny.toml`
carries as a named exception.

**Measure both on uk-UA in Phase 4:** load time, RSS, check ns/word, suggest p95, and suggestion
quality (`just ua-gec`). For Ukrainian it is the deciding case. Its morphology is rich enough that
fully expanding it could be big, and big enough that affix rules could be slow. Neither failure is
known in advance. Both implementations may survive, chosen per pack in `pack.toml` — that is a
table, not a fallback.

**Decided for uk-UA, 2026-11-06: `CompiledDict`** (`dictionary = { compiled = "uk_UA" }`). Expanding
dict_uk was neither failure: its 353 211 stems and 92 suffix classes make 3 555 574 forms, and an
FST holds them in 3.9 MB. Measured against spellbook on the same dictionary, with FrequencyWords'
ranks for both:

|                                              |         spellbook |      compiled |
| -------------------------------------------- | ----------------: | ------------: |
| one Ukrainian word checked, whole process    | 0.14 s, 49 MB RSS | 0.02 s, 19 MB |
| UA-GEC's 3 953 texts, 864 000 words, checked |     0.75 s, 85 MB | 0.55 s, 55 MB |
| corrections for UA-GEC's 744 unknown words   |     4.47 s, 59 MB | 1.16 s, 29 MB |
| UA-GEC non-word top-1 / top-5                |     47.1 / 64.6 % | 49.1 / 67.4 % |
| words with no correction, of 599             |                51 |            15 |

- **It accepts what spellbook accepts.** The 15 341 findings on all of UA-GEC are the same, byte for
  byte. Spellbook accepts every compiled form but 28 `+cs=…` entries that `ICONV` makes unreachable
  in both. A differential test holds the two to the same answer on every form of a small dictionary,
  every casing of each, and every form one edit away (`crates/dict/src/compiled/tests.rs`).
- **A correction is every form within two edits** (one, for four letters or fewer), found by walking
  the FST with a bit-parallel Levenshtein automaton. Spellbook found none for 51 of the 599, and its
  n-gram pass, which would, took 150–300 ms a word on this dictionary.
- **It is compiled when the pack is installed**, from the `.aff` and `.dic` the install fetched: 1.9
  s for dict_uk. The forms are made in passes of about 8 MB, so the 86 MB of them are never held at
  once. The FST's type is its format, and a new format comes with a new version of every compiled
  pack, so the next start installs it again.
- **What it cannot hold, it refuses**: prefixes, compounding, a suffix with further affixes, and the
  flags that make a form need an affix, forbid it or fix its case. Such a pack keeps `hunspell`.
  That is why en-US stays on spellbook: SCOWL's `.aff` has prefixes and `COMPOUNDRULE`.

---

## 8. Suggestions

Suggestions are computed for **one word**, on request, and cached per `(WordId, epoch)`.

**Candidates come from three sources, merged:**

1. the exact fix from `typos.tsv`, when there is one;
2. the dictionary's own `suggest`;
3. project-lexicon words within edit distance 2.

**Ranking** combines a weighted edit distance (keyboard adjacency is cheap, a transposition costs
1), word frequency, project frequency and, if enabled, the context score (`04-context.md`). The
context score can reorder candidates; it can never add one.

**Casing and position are restored.** `Recieve` → `Receive`, `RECIEVE` → `RECEIVE`, and the part of
an identifier is replaced in place: `recieveMessage` → `receiveMessage`.

**What a fix means for an identifier is an open question** (`03-roadmap.md`). Replacing a
declaration alone breaks every use. Two options: offer "replace in this file", or name the fix and
leave the rename to the language's own server. That decision needs users, not an argument.

---

## 9. The server

The server uses `tower-lsp-server` + tokio, the same stack as phrost, including its panic
containment, and names neither: both come through lspkit (`lspkit_runtime::protocol` and
`serve_stdio`), so replacing tower is one change in lspkit for every server.

- **Document store:** text, version, `HostDoc`, the last findings. A buffer is never stored twice.
- **Diagnostics are pushed, to every client,** open files and closed ones alike, each publication
  replacing the last. Pull was tried and dropped (2026-11-05). Zed keeps pulled and pushed
  diagnostics apart, so a file that was both had to be cleared on one channel when it opened. Zed's
  list of a project's problems opens a buffer for each file it shows: the open cleared the file, the
  list let it go, the close published it again, and the list emptied and refilled in a loop. One
  channel says everything. phrost pushes too.
- **Context-tier results arrive later,** and are published again. Results for a stale document
  version are dropped.
- **Code actions** carry their edits whole: a suggestion costs half a millisecond at p95, computed
  for the one word asked about, so nothing is left for `codeAction/resolve`.
- **Positions:** negotiate `positionEncoding`: UTF-8 when the client offers it, UTF-16 otherwise.
  All conversion happens in `orthra-text::LineIndex` and nowhere else.
- **Readiness:** the project is learned off the request path at `initialized`, through
  `orthra-project`, as `orthra check` learns it. Nothing is reported before that, because half a
  project's vocabulary reports the other half's names as unknown words.
- **Every problem in the project, not only the open files':** once the project is learned, what is
  open is answered, and then every file is checked on a fork of the engine, so requests are answered
  meanwhile. The closed files' findings are published, and an editor lists them with the rest. A
  file that opens is published as an open file, one that closes as a closed one, and neither is ever
  cleared in between.
- **Severity:** a warning by default, for every finding. A spelling is not a fault in the code, but
  Zed lists only errors and warnings among a project's problems, and shows an information only where
  it is written. `severity = "information"` in `orthra.toml`, the user's `config.toml` or the
  editor's options says otherwise; the most specific wins.
- **Code actions,** for the word under the cursor: its fixes (a known typo's, or up to five
  suggestions, computed then and for that word only), add it to the project's words or the user's
  (the `orthra.addWord` command, which the server runs), or ignore it in the file (an
  `orthra: ignore` directive, in the file's comment syntax, so the file says it).
- **Watching:** the client is asked to watch `orthra.toml`, `.orthra/words.txt` and the user's
  files. A change is a new pass: the settings read again, the engine rebuilt, the project learned
  and checked, and files that are clean now cleared.
- **Languages** come from `initializationOptions.languages` until the configuration layers exist.

### Configuration layers

| layer    | where                                                        | holds                                            |
| -------- | ------------------------------------------------------------ | ------------------------------------------------ |
| built in | the binary                                                   | en-US, hosts, defaults                           |
| user     | `~/.config/orthra/config.toml`, `~/.config/orthra/words.txt` | personal words and languages                     |
| project  | `orthra.toml`, `.orthra/words.txt`                           | project words, ignored paths, per-glob languages |
| editor   | LSP settings (`lsp.orthra.settings` in Zed)                  | on/off, severities, the context tier             |

Word lists and `languages` are **unions** across the layers. For scalar settings, the most specific
layer wins: editor, then project, then user, then built in. Word files are watched. A change bumps
the epoch and re-checks the open documents.

**Built (2026-11-05)** in `orthra-project`, and read the same way by `orthra check`,
`orthra explain` and the server:

```toml
# orthra.toml, at the project's root
languages = ["en-GB"]                           # added to en-US and the user's
ignore = ["crates/hosts/src/*/vocabulary.txt"]  # gitignore patterns: files that are not prose
```

- **A words file's word is a word everywhere,** a declaration and a known typo included: somebody
  wrote it down on purpose. It is checked before the verdict cache, so nothing cached goes stale.
- **A project's root** is the nearest directory with an `orthra.toml` or a `.git`, then the nearest
  with a manifest. A member of a Cargo workspace has a manifest of its own, and the repository's
  settings are still its settings.
- **A broken settings file is a problem said out loud:** `orthra check` exits 2, and the server
  shows it. A key that is not a setting is one too.
- **The instruments run `--isolated`,** built-in settings only, so the corpora's numbers do not
  depend on whose machine or which checkout they run in.
- **The editor's layer** is `initialization_options`: `languages` and `severity`.

---

## 10. Determinism as an invariant

Every lexical finding is a function of text, language id, config and packs (`CLAUDE.md` §1.2). The
server carries no second path, no timing, no edit-history state and no fuzzy fallback.

The harness checks this from every direction it can:

- `incremental == from-scratch` — edit history;
- `cold == warm` — caches;
- `shift` — encoding;
- `context-off ⊆ context-on` — the one non-deterministic component.

`02-testing.md` level 4 says how each one runs.
