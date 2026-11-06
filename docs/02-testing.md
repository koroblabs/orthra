# Testing

This is the most important document. The others describe what to build; this one describes the only
thing that stops it rotting. For a spell checker there is a second reason it matters: **quietness
can only be proven by a corpus.** Every false positive is a correct word that some test failed to
contain.

The levels are ordered so that each one is cheap only because the one before it exists. Nothing is
built yet. When a level is built, mark it **built**. If it is built differently from what is planned
here, say how and why — phrost's `no-fallback` level is the model for that.

---

## Level 0. One fixture format

`orthra-testkit` reads one format, and every harness in the workspace uses it. It is borrowed from
phrost.

```
//- /orthra.toml
languages = ["en-US"]

//- /.orthra/words.txt
frobnicate

//- /src/lib.rs
/// Retruns the frist element; frobnicate it later.
fn get_frist_item(s: &str) -> &str { "Hello\nworld" }
```

**Built** (`crates/lib/testkit/src/fixture.rs`).

- `//- /path` starts a file. The host comes from the extension; `host=<id>` overrides it. A path is
  spelled with its leading slash everywhere, in the header and in every lookup.
- The **subject is the last file**. One subject per test (`CLAUDE.md` §8.3).
- `$0` is a cursor, and `@name@` is a named marker. Both are needed only for code-action queries.
  Every marker must be consumed, so an unused one fails with its name. **A marker's safety is scoped
  to the hosts it was checked against**: `$0` collides with shell and PHP. The `raw` flag turns
  marker reading off for one file, and each host's fixture suite has one test proving that its
  syntax and the markers coexist.
- Common indentation is trimmed and trailing blank lines are dropped, so a fixture can be indented
  with the test that holds it.
- A duplicate path, an unknown flag, content before the first header or a duplicate marker is a
  panic.

### The rendering: annotated text

**Built** (`crates/lib/testkit/src/annotate.rs`). Findings render as the document, each line
prefixed `> `, with carets under the words, so a snapshot can be read without a decoder:

```
> /// Retruns the frist element; frobnicate it later.
      ^^^^^^^ unknown-word
                  ^^^^^ known-typo -> first
> fn get_frist_item(s: &str) -> &str { "Hello\nworld" }
         ^^^^^ known-typo -> first (declaration)
```

`frobnicate` is silent because the words file says so. `nworld` is silent because the Rust host
knows `\n`. **The silence is part of the snapshot.** A line with no caret under it is a claim as
well.

Because the text is always rendered, a clean result is never an empty string. An `expect![""]` can
only mean an empty document, which removes the accepted-empty snapshot hazard (`CLAUDE.md` §8.3) by
construction rather than by care. The `> ` prefix also gives every rendering a line that starts in
column 0, which keeps expect-test and `cargo fmt` from fighting over indentation.

---

## Level 1. Layered fixtures

Each layer renders **its own** data and builds the pipeline only as far as itself. When an answer is
wrong, the first layer that disagrees is where the bug is.

| layer            | renders                                                                                                                  |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `orthra-hosts`   | spans: range, kind, the text it covers                                                                                   |
| `orthra-segment` | words: range, shape (`lower`/`Title`/`UPPER`/`camelPart`), script, and anything skipped with the pattern that skipped it |
| `orthra-dict`    | verdicts per pack, for a list of words                                                                                   |
| `orthra-checks`  | findings (the annotated text above)                                                                                      |
| `orthra-suggest` | ranked candidates with their score components                                                                            |
| `orthra-engine`  | findings plus the offers for each finding                                                                                |

---

## Level 2. `expect-test`, never a substring

**Every result check is a snapshot of the whole result.** That never means "`receive` is in the
list". A substring assertion lets nine junk suggestions through. `UPDATE_EXPECT=1` rewrites the
snapshots, and the diff is where a reviewer sees "this change re-ranked forty suggestion lists".

Snapshots live **inline**, next to the test that owns them. Never use a shared `expected.txt`.

---

## Level 3. N queries per fixture

One setup runs several checks that must agree with each other:

```rust
check(FIXTURE, &[
    Query::diagnostics("all"),
    Query::actions("actions at frist", "frist"),
    Query::explain("why", "frist"),
], expect![[r#" ... "#]]);
```

This catches features drifting apart. Examples: a diagnostic whose range differs from the range its
code action replaces, or an `explain` that names a different pack from the one that produced the
finding.

---

## Level 4. Invariants as harness dimensions

These are not tests. They are **dimensions that every fixture runs through**, so one new fixture
buys all of them. Build them in Phase 0, while there is nothing to break.

**Built** in `orthra-harness`, apart from `host-agnostic prose`, which needs hosts and is an engine
test (`crates/engine/src/tests/host_agnostic.rs`). The harness is generic over a `Subject` trait and
knows nothing about spelling, which keeps it below the engine in the crate graph. `check_all` runs
every dimension over one fixture and fails with every violation at once.

**A dimension that cannot fail is a comment.** So each one was proven before anything real depended
on it: `tests/planted.rs` runs a test double with one defect planted at a time, and checks that the
dimension built for that defect catches it.

| planted defect                                | caught by                                                                          | missed by                                             |
| --------------------------------------------- | ---------------------------------------------------------------------------------- | ----------------------------------------------------- |
| a memo an edit forgets to drop                | `incremental == from-scratch`                                                      | the other three                                       |
| ranges in characters instead of bytes         | `shift`                                                                            | the other three, because every ASCII fixture hides it |
| a cache keyed by the word alone               | `cold == warm`, and `context-off ⊆ context-on` (which also asks one session twice) | `shift`                                               |
| a context tier that rewords a lexical finding | `context-off ⊆ context-on`                                                         | the other three                                       |

### `incremental == from-scratch`

A fixture can carry a sequence of edits. The harness applies them to a live document (an incremental
reparse, cache reuse) and compares the findings against a fresh check of the final text. The two
must be **byte-identical**. Findings are asked for after every edit, so a cache has every chance to
go stale. `check_all` also runs 24 seeded random edits per fixture (`dimensions::fuzzed`,
lspkit-fuzz's sweep). They cut words in half, join words, and insert typos, newlines, quotes,
comment markers, Cyrillic and emoji; some are same-length, which leave every later range where it
was and so find a stale cache. A failure names the seed and the edit, and is shrunk to the edits
that still fail.

### `cold == warm`

Check the fixture with an empty verdict cache, then with a warm one, then again after a no-op epoch
bump. If a compiled dictionary exists, check against it and against the source `.aff`/`.dic`. All
results must be identical. A verdict cached across a word-list change is the bug this exists to
catch.

### `shift`

Prepend a neutral line such as `// Привіт 👋\n` to the subject. Every finding must be identical, one
line down, at the same column **in every position encoding** (UTF-8, UTF-16, UTF-32). It costs
nothing per fixture, and it is the UTF-16 test for the whole suite.

The subject supplies the line, because what is inert depends on the host: `//` is a comment in Rust
and a syntax error in Python. JSON has no comments, but its grammar reads JSONC's `//`, so it runs
this dimension too. The harness refuses a line that lacks a multi-byte character and an astral one,
since such a line shifts nothing an encoding bug could see.

### `context-off ⊆ context-on`

Run with the context tier off, then on with `FakeScorer` (deterministic and adversarial: it scores
the _worst_ candidate highest). Every lexical finding must appear unchanged. Only findings under the
`orthra/context` source and the order of suggestions may differ.

### `host-agnostic prose`

One body of prose is written in every host's comment syntax: 22 of them, from Rust's `//`, `///`,
`//!`, `/* */` and `/** */` to Python's docstring, PHP's `#`, YAML's `#`, JSONC's `//`, a Markdown
paragraph and a commit message. The findings, word and check, must be those of the same lines in a
plain text file. This catches a host that strips decorations wrongly, for example the `*` inside a
block comment, or that reads the same words by other rules.

It caught one when it was built (2026-11-05): Markdown dropped inline code, so `` `recieve` `` was
reported in every comment and never in a README. Inline code is now held to known typos there too.
On the zero corpus that added four findings, all deliberate typos quoted in phrost's docs.

---

## Level 5. Hosts and segmentation

- **Inline tests beside the rule.** A `spell.scm` and its fixtures share a directory.
- **A host corpus.** Real files per host. Every span lies within the document. Spans are ordered and
  do not overlap. A span never starts or ends inside a UTF-8 sequence.
- **Splitter parity.** The identifier splitter runs against typos' splitter over a list of real
  identifiers, and every disagreement is reviewed once and then pinned.
- **Grammar loading.** Each host has a test that loads its grammar and compiles its query, because
  an ABI mismatch or a renamed node type fails at runtime (`CLAUDE.md` §8.1). A second test checks
  that every grammar in `Cargo.toml` is pinned with `=`: the incumbent was broken for anybody who
  ran `cargo install` without `--locked`, because its manifest allowed `tree-sitter-dart` `<1`.

---

## Level 6. Flow tests at the LSP boundary

`crates/lsp/src/tests/flow/` drives the server through the protocol, in process, against a project
written to a directory of its own. **Built so far:** a pushed client hears nothing until the project
is learned and then everything open; a change is published once the buffer is quiet; a client that
could pull is pushed to, and opening and closing a file never clears its findings; positions are in
the encoding negotiated; an incremental change lands where the client said, after multi-byte text; a
project's settings are read; every problem in the project is published, open files and closed; a
finding offers its fixes, the words files and a directive; adding a word writes the words file and
checks the project again; a changed settings file is a new pass. The flows are:

- the document lifecycle;
- diagnostics pushed, whatever the client could pull;
- a config change, a words-file change and a newly installed pack, each followed by a re-check;
- cancellation;
- `codeAction/resolve`;
- **a context tier that never answers**: the lexical diagnostics still arrive within budget, and the
  stalled job is dropped when the version moves on.

---

## Level 7. Differential: the incumbent and the others

**`just compare <dir>`** runs codebook, typos and cspell over the same files as black boxes. It
reports each finding as `both`, `orthra-only` or `other-only`. A sample of each bucket is labeled by
hand (true or false positive), which turns "codebook is not good enough" into precision and recall,
measured on the same files, for both.

**Built** (2026-11-06): `tools/compare.py`, with `just compare <dir>` and `just compare-zero`.

- **The same files.** Every tool is asked about the files `orthra check --files` lists, the ones
  orthra has a host for.
- **Matching.** A finding is matched by file, line and word (lower-cased, without a possessive
  `'s`), never by column.
- **Labels.** `corpora/compare/<project>.tsv` holds the labeled sample of each bucket, read line by
  line: `real` for a misspelling the author would fix, `noise` for anything else, by whom. A bucket
  with none real among n labeled is reported as at most 3/n real, at 95 %.
- **typos and cspell** are run when they are on `PATH`, and said to be missing otherwise. Nothing is
  installed by the tool.

**The zero corpus, 2026-11-06,** with the languages codebook is given, and 40 to 48 findings labeled
per bucket per project (565 in all, by Claude, to be reviewed):

|                                             | findings | about real |  precision | recall of what either found |
| ------------------------------------------- | -------: | ---------: | ---------: | --------------------------: |
| orthra                                      |    2 481 |       44.6 | **1.80 %** |                   **100 %** |
| codebook                                    |   27 504 |       43.6 |     0.16 % |                      97.8 % |
| orthra with SCOWL's rare words (2026-11-06) |    2 248 |       43.6 | **1.94 %** |                       100 % |

The real ones are five typos: `extractiong` (ripgrep), `textojbects` and `heirarcihical` (helix),
and `prerequesite` and `internat` (starter workflows). orthra found all five, and codebook missed
`internat`. With the rare words orthra misses `internat` too: SCOWL size 70 knows it as a word. No
labeled finding of codebook's alone was real, which bounds what orthra misses on these projects at
about 7 % of that bucket, not at zero. The direct measure of recall is the typo sample (level 9).
There codebook flags 3 points more non-word typos in prose and 7 more in code, which is the price of
known-typos-only in code, and orthra's first suggestion is right 7 to 10 points more often.

orthra's own noise in these samples is mostly real words missing from its en-US dictionary:

- plurals the dictionary does not inflect (`integrations`, `customizations`, `parsers`);
- derived forms (`vanishingly`, `reproducibly`, `decrypting`);
- British spellings in an en-US check (`behaviour`, `cancelled`, `analyse`).

**`corpora/incumbent/`** holds the concrete cases collected in Phase 0 (its README has the
hypotheses they test). They start out failing. Phase 3 does not exit while one still fails.

**Built** as a format (`orthra-testkit::corpus`). A case is a fixture under a short header —
`hypothesis`, `source`, `incumbent`, `why` — and its subject carries the `expect` flag: it is
_written as the annotated rendering of the right answer_, which `Annotation::parse` reads back into
a text and its expected findings. So a case reads exactly as its failing test will print, and
`the_incumbent_corpus_is_well_formed` refuses a case that is not in canonical form. Labels are the
finding's code, then ` -> fix` when there is one exact fix: `known-typo -> receive`.

---

## Level 8. Real projects, ratcheted

**`orthra check <dir> --ratchet`** counts findings **by code** against `orthra-baseline.toml` beside
the project. It counts by code and not by total, because ten closed `unknown-word` findings against
ten new `mixed-script` ones are the same total and not the same project. A code that is absent from
the baseline also fails the ratchet, because a new check is a decision, not an accident.

**Built** (2026-11-05). A count that falls is written back on its own, so the baseline only goes
down. A rise fails until `--update-baseline` records it, and the commit that does says why.
`--baseline <file>` names a baseline kept elsewhere: `just zero` ratchets every project of the zero
corpus against `corpora/zero/<project>.toml`, committed here because the projects live in `.cache/`,
which is not. A planted rise fails it; a planted slack is lowered and written back.

---

## Level 9. Labeled corpora — quietness and recall, as numbers

- **The zero corpus.** Real, well-kept repositories whose prose is believed to be correct: phrost,
  ripgrep, and a few chosen per host. **The expectation is zero findings** once each project's words
  file is in place. Every finding is reviewed. It is either fixed upstream (a real typo, which is
  worth a PR to that project) or traced to a cause in orthra and fixed there. The number reported is
  **false positives per 10 000 words**. `just zero` ratchets the findings by code (level 8), and
  `just zero-orthra` puts them per 1 000 lines beside codebook's, on the same files. They are the
  numbers this project is judged by first.

  **Built, as data:** `tools/zero-corpus.py`, with `just zero-corpus` and `just zero-codebook`.
  - phrost at a pinned commit, taken from the sibling checkout.
  - ripgrep 15.2.0 from its tarball, digest-checked.
  - composer 2.10.3 from its tarball, digest-checked: PHP, for phrost's users.
  - helix 25.07.1 from its tarball, digest-checked: TOML, in a table of languages and 172 themes.
  - GitHub's starter workflows at a pinned commit, digest-checked: YAML, as a developer edits it.
  - Nothing is committed. Each project counts the extensions it is there for: `.rs` and `.md` for
    phrost and ripgrep, `.php` and `.json` for composer, `.toml` for helix, `.yml`, `.yaml` and
    `.json` for the workflows. Every file is still read, so what a project declares counts as it
    would in an editor.

  **The bar, from codebook-lsp 0.3.42** on 2026-11-05, before any words file. phrost was given
  `en_gb` as well, because it is written in British English.

  |         |   lines | findings | per 1 000 lines | comments | identifiers | Markdown | strings |
  | ------- | ------: | -------: | --------------: | -------: | ----------: | -------: | ------: |
  | phrost  | 321 055 |   10 108 |            31.5 |    3 819 |       3 254 |    2 005 |   1 030 |
  | ripgrep |  58 188 |    3 649 |            62.7 |      718 |       1 206 |      841 |     884 |

  In Markdown that is about one finding every six lines. Read by word, the findings fall into a few
  causes, and each cause is a rule for orthra (`01-architecture.md` §6):
  - **the project's own name**: `phrost` 980 times, `ripgrep` 789;
  - **its dependencies and its domain**: `mago`, `phpstan`, `pcre`, `termcolor`, `mmap`;
  - **identifiers it declares**: `hir` 986, `fqn` 463, `interner`;
  - **possessives of all of these**: `laravel's`, `ripgrep's`, 13 % of phrost's findings;
  - **abbreviations inside identifiers**: `rdr`, `wtr`, `irst`, `fnid`;
  - **test data**: `watsons`, `holmeses`, ripgrep's Sherlock Holmes fixture.

  **orthra, first measured 2026-11-05, Markdown only** (the only host it has for these files;
  `just zero-orthra`): **noisier than codebook.**

  | Markdown                 | orthra (en-US) | codebook |
  | ------------------------ | -------------: | -------: |
  | phrost, per 1 000 lines  |            193 |      142 |
  | ripgrep, per 1 000 lines |            179 |      163 |

  The causes can be read off orthra's words:
  - **No technical vocabulary yet** (`goto`, `namespace`, `enum`, `nullable`, `config`, `stdin`,
    `readme`, `changelog`, `repo`), where codebook ships one.
  - **No en-GB pack** for phrost's `judgement` and `behaviour`.
  - **The project's own and its dependencies' names**, which both checkers report and declared
    vocabulary will not (`ripgrep` 523 times).

  That list is the first work of Phase 2.

  **After technical vocabulary and en-GB** (2026-11-05, the same files, the same languages as
  codebook: en-US + en-GB for phrost):

  | Markdown                 |                       orthra | codebook |
  | ------------------------ | ---------------------------: | -------: |
  | phrost, per 1 000 lines  |                    **114.6** |    142.4 |
  | ripgrep, per 1 000 lines | **29.4** (5.5 times quieter) |    162.9 |

  The rest of phrost's findings are names: `laravel`, `phrost`, `symfony`, `mago`, `phpantom`.
  phrost declares them in its own code, so declared vocabulary needs the Rust host first.

  The typo sample's price was about one point of recall. It is now 88.0 % (code) and 81.0 % (prose);
  top-1 is 72.5 / 67.6 %, and top-5 is 78.2 / 72.6 %.

  **Measured again on 2026-11-06**, with a line from a Markdown file written as `.md`, so its links
  and code spans read as an editor reads them (`corpuslib.case_file`). The 811 Markdown rows had
  been plain text.

  |                 |   n | flagged |  top-1 |  top-5 |
  | --------------- | --: | ------: | -----: | -----: |
  | code, non-word  | 852 |  83.1 % | 70.1 % | 75.4 % |
  | prose, non-word | 762 |  81.4 % | 69.8 % | 74.5 % |
  - Reading Markdown as Markdown moved prose by 0.4 points (69.4 / 74.1 % as plain text), and
    codebook not at all.
  - **The vocabulary as corrections (built the same day).** A pack now offers its vocabulary's words
    within an edit of a short word or two of a longer one, after the dictionary's as near:
    `clodebase` is `codebase`, which no dictionary has. Top-1 went to 70.7 / 70.6 % (code / prose)
    and top-5 to 76.4 / 75.6 %, at 0.3 ms more per correction asked for (p50 0.16 → 0.49 ms, p95 0.5
    → 0.8 ms on ripgrep).
  - **cspell's 107 lists as the vocabulary (the same day; the zero corpus below).** Top-1 71.4 /
    71.4 %, top-5 77.1 / 76.2 %. Asking for a correction got faster, not slower, at 6.7 times the
    entries: p50 0.57 → 0.37 ms and p95 0.87 → 0.79 ms on phrost, because an entry is passed over by
    a sketch of its letters and its length before its edits are counted. Two rules keep the larger
    list from crowding the dictionary out: the lists of names come before the languages' words among
    corrections as near (`Githb` is `GitHub` before the Python package `gitdb`), and a word the
    dictionary accepts is offered from the vocabulary only when nothing nearer is (`04-context.md`
    §7 has what that did to the model).
  - **What did not help English, measured on the same candidates:**
    - ordering them by SCOWL size, which is how common a word is: +0.7 top-1, for 1–2 MB of data to
      carry;
    - an error model that makes neighboring keys, swaps and doubled letters cheaper: ±0.3;
    - FrequencyWords' English counts, for comparison only, since its license keeps it out of the
      binary: +0.1. The lexical order is already right for 93 % of the typos whose fix is among the
      candidates; the rest is detection and candidates, and the context tier (`04-context.md`).
  - Code recall is 5 points under the 2026-11-05 figure. The 123 non-word typos orthra finds alone
    and not in their line are mostly in code: backticks, an identifier at its use site
    (`resyscInterval`), or what reads as a path (`Inflate/Defalte`). `04-context.md` §4 has the
    count.

  **What the 247 missed non-word typos are**, read row by row (2026-11-05):

  | cause                         | rows | the reading                                                                                                |
  | ----------------------------- | ---: | ---------------------------------------------------------------------------------------------------------- |
  | accepted by the dictionary    |   89 | mostly not typos: `creates → create` is grammar, and `/usr/share/dict/words` has no inflections to know it |
  | inside backticks              |   55 | real: `` `cloude` ``, `` `jonh` ``, which code-in-backticks skips whole                                    |
  | inside a path, URL or address |   55 | real, but a path names a file that may really be spelled that way                                          |
  | in the technical vocabulary   |   26 | not typos: `https → http`, `regexp → regex`, `Telegraf` is a product                                       |
  | the acronym rule              |   17 | `GPS → GPU`: a real word in another field                                                                  |
  | glued to a digit              |    5 | `aouth`, `pcks`                                                                                            |

  **After the code policy and declared vocabulary** (2026-11-05):
  - **Code is held to known typos only.** That covers backtick code, identifier parts in prose, and
    declarations.
  - **The manifests' names are accepted:** the package name and every dependency's.

  |          | Markdown noise per 1 000 lines | typo recall, code / prose |
  | -------- | -----------------------------: | ------------------------: |
  | before   |     phrost 114.6, ripgrep 29.4 |             88.0 / 81.0 % |
  | after    |  **phrost 80.4, ripgrep 27.9** |             83.9 / 82.5 % |
  | codebook |    phrost 142.4, ripgrep 162.9 |             90.1 / 84.4 % |

  Prose recall went up, because backtick code is now checked for known typos. Code recall went down
  four points; that is the price of not reporting unknown words in identifiers, and open question 7
  weighs it against the zero corpus's identifiers once the Rust host exists.

  **With the Rust host, on every `.rs` and `.md` file** (2026-11-05). This is the comparison the
  zero corpus exists for: the same files as codebook's baseline, and the same languages.

  | per 1 000 lines              |               orthra |            codebook | quieter by |
  | ---------------------------- | -------------------: | ------------------: | ---------: |
  | phrost, all (`.rs` / `.md`)  | **2.5** (1.6 / 21.9) | 31.5 (26.4 / 142.4) |      12.6× |
  | ripgrep, all (`.rs` / `.md`) | **4.6** (3.7 / 13.6) | 62.7 (53.0 / 162.9) |      13.6× |

  The rules that got it there, each general and each measured:
  - **Code is held to known typos only.**
  - **Declared vocabulary,** from the manifests (workspace members included) and from what the files
    declare. `laravel` and `symfony` went because phrost declares them.
  - **Each host's own vocabulary:** `async`, `usize`, `serde` in Rust.
  - **Unknown words from four letters.**
  - **Not words:** file extensions after a dot, a chunk that one dot starts (`.gitattributes`,
    `.rgignore`, and ripgrep's table of file types: 106 findings), runs glued to `@`, and globs.
  - **Test data is not checked:** strings in test files, in `#[cfg(test)]` modules and in `#[test]`
    functions.
  - **Code in doc comments is not prose:** examples between ` ``` ` fences.

  What is left is each project's own lexicon: `phpantom`, `intelephense`, `drush` in phrost;
  `ugrep`, `pdftotext` and `submatch` in ripgrep. That is what a project's words file is for. Typo
  recall, after all of it: code 83.3 %, prose 81.5 %.

  **With the PHP host, on composer 2.10.3's `src/`** (309 files, 74 926 lines, en-US; 2026-11-05).
  Composer is the project phrost's users depend on, and its release tarball holds the code without
  the tests or the docs.

  | per 1 000 lines |  orthra | codebook | quieter by |
  | --------------- | ------: | -------: | ---------: |
  | composer `.php` | **3.4** |     50.6 |      14.9× |

  It started at 29.8. What took it down, in order of size:
  - **The authors are declared** by the manifest. `Nils Adermann` and `Jordi Boggiano`, in the
    license header of every file and in `@author` lines, were 1 814 of the first 2 235 findings.
  - **PHPDoc tags are the host's data.** `@author`, `@since` and `@see` lines are not prose, and the
    type after `@param`, `@return` or `@var` is code, multi-line array shapes included.
  - **PHP's vocabulary:** the words its functions glue together (`strtolower`, `fwrite`), its
    extensions (`mbstring`, `pgsql`) and the words it coined (`autoloading`, `isset`).

  What is left: config keys in one-token strings (`'cache-files-maxsize'`), dotfiles and domains
  (`.gitattributes`, `getcomposer.org`), and British spellings in an en-US run.

  **With the TOML host, on helix 25.07.1's `.toml`** (195 files, 24 068 lines, en-US; 2026-11-05).

  | per 1 000 lines |  orthra | codebook | quieter by |
  | --------------- | ------: | -------: | ---------: |
  | helix `.toml`   | **5.9** |     87.3 |      14.8× |
  - **A string of one token is a value**, held to known typos: 536 of the first 808 findings were
    names in strings, `"ocaml"`, `"lldb"`, `"ocamllsp"`. A key is a declaration, so `[dependancies]`
    is still reported.
  - **A script unlike Latin ends a word** where it meets Latin, as Chinese is written against it
    with no space: `preview窗口` is `preview` and `窗口`, not a `mixed-script` finding. Latin,
    Cyrillic and Greek stay in one word when they meet, because their letters look alike.
  - **A comment line labeled with people is names:** `# Author: Matous Dzivjak <...>`,
    `# Original author: rebelot`. The labels are the pack's data, because `Author` is English: 121
    of the 262 left.
  - **What is left:** theme and product names (`monokai`, `everforest`), people named in prose
    (`by Protesilaos Stavrou`), and British spellings in an en-US run. Among them is a real typo,
    `fucused`.

  **With the YAML host, on GitHub's starter workflows** (187 files, 10 427 lines, en-US;
  2026-11-05).

  | per 1 000 lines                   |   orthra | codebook | quieter by |
  | --------------------------------- | -------: | -------: | ---------: |
  | starter workflows `.yml`, `.yaml` | **16.7** |    217.8 |      13.0× |
  - **A string of one token is a value**, as in TOML: `ubuntu-latest`, `stable`.
  - **What a step runs is code:** the values of `run:`, `script:` and `command:` are held to known
    typos, so `cargo biuld` is reported and `gradlew` is not. 76 of the first 250 findings.
  - **What is left:** vendors and their products, which these templates exist to name (`frogbot`,
    `zimperium`, `nexploit`). Among them are four real typos: `recieve` twice, `Prerequesite`,
    `Analaysis`.

  **With the JSON host** (2026-11-05): composer's three JSON schemas, whose `description` fields are
  prose, and the starter workflows' 184 `properties.json`.

  | per 1 000 lines                 |   orthra | codebook | quieter by |
  | ------------------------------- | -------: | -------: | ---------: |
  | composer `.json`                |  **4.6** |     45.3 |       9.8× |
  | starter workflows `.json`       | **25.1** |    151.3 |       6.0× |
  | composer, all (`.php`, `.json`) |  **3.2** |     50.4 |      15.8× |
  | starter workflows, all          | **17.8** |    206.4 |      11.6× |
  - **A string is decided whole:** an escape does not split `"Hello\nworld"` into two values.
  - **npm's and composer's `scripts` are code,** like YAML's `run:`.
  - **A schema's keys are declarations,** so composer's PHP gained from them: `cache-files-maxsize`
    in its schema accepts `maxsize` in its code.
  - **What is left:** the vendors that the workflows' properties name, `octicon` icon names, and
    British spellings in an en-US run.

  So the vocabulary cost almost nothing real. The recall to win back is in code inside prose. The
  fix is the identifier policy (`03-roadmap.md` open question 7): check backtick code for known
  typos only, as identifiers will be, instead of skipping it. That way `qzx` stays silent and
  `cloude` does not.

  **With cspell's software vocabulary** (2026-11-06). The vocabulary grew from one cspell-dicts
  package, `software-terms` (3 995 words), to 107 of its lists: software terms, tools, companies,
  cloud services, and the words of 40 programming languages (`packs/en-US/SOURCES.md`), 26 838
  words. `just zero`'s `unknown-word` counts:

  |                   | before | after |
  | ----------------- | -----: | ----: |
  | phrost            |    629 |   529 |
  | ripgrep           |    345 |   282 |
  | composer          |    228 |   173 |
  | helix             |    679 |   510 |
  | starter workflows |    199 |   143 |

  Per 1 000 lines on the same files, codebook is now 16 to 23 times noisier (`just zero-orthra`:
  phrost 1.8 against 31.5, helix's TOML 4.8 against 87.3).
  - **Kept out, so that no typo hides behind a list:** cspell's forbidden words, every known typo,
    British spellings, and 12 000 API names (`assertTrue`, `CNodeBase`), the mixed-case entries
    outside the lists of names. In prose a mixed-case token is an identifier, checked by its parts,
    so an API name accepted almost nothing a plain word did not; as a correction it came first,
    `clodebase` getting `CNodeBase` before `codebase`.
  - **phrost's `parm` is a known typo again,** 42 → 43: its list of misspelled tags writes it beside
    `retrun` and `pram`, which were reported already. cspell had it as a word.
  - **Four findings are new, and real:** `statuscode` three times and `nextline` once, two words
    written as one in prose, which only the API names `statusCode` and `nextLine` had accepted,
    whatever their case. The terms that are words in lower case too (`changelist`, `fastpath`,
    Blade's `endunless`, Perforce's `modtime`) went into `vocabulary.orthra.txt`.

- **GitHub Typo Corpus.** Real typo fixes from commits: code, comments and docs. `just typo-eval`
  reports recall (did we flag the typo?), suggestion top-1 and top-5 (was the fix our first, or in
  our first five?), and false positives on the corrected side of each edit.

  **Built, as data:** `tools/typo-corpus.py`, with `just typo-corpus` and `just typo-codebook`.
  - **Nothing is committed.** The corpus texts keep their own repositories' licenses.
  - **Fetched from the Internet Archive,** because the README's S3 link is dead, and checked against
    the digest the archive recorded.
  - **The sample is 2 000 rows, deterministic:** 1 000 prose and 1 000 code. Each row is an edit
    where exactly one word part changed, both sides have three or more letters, the two sides are at
    most two edits apart, and the change is not only case. `is_typo` in the corpus also marks
    whitespace edits, so it cannot be trusted alone.
  - **Real-word typos (`form` for `from`) are scored apart.** They are the context tier's question,
    not a dictionary's. The tag comes from `/usr/share/dict/words`, which has no inflected forms, so
    a few real words are tagged as non-words.

  **The bar, from codebook-lsp 0.3.42** on 2026-11-05. Every line is checked as plain text, and each
  percentage is of all rows in its group:

  |                  |   n | flagged |  top-1 |  top-5 |
  | ---------------- | --: | ------: | -----: | -----: |
  | prose, non-word  | 762 |  84.4 % | 59.1 % | 73.2 % |
  | code, non-word   | 852 |  90.1 % | 63.3 % | 79.1 % |
  | prose, real-word | 238 |   4.6 % |  0.8 % |  2.1 % |
  | code, real-word  | 148 |   6.8 % |  2.7 % |  4.7 % |

  **orthra, first measured 2026-11-05** (Phase 1's vertical slice, en-US, `just typo-orthra`):

  |                 | flagged |      top-1 |  top-5 |
  | --------------- | ------: | ---------: | -----: |
  | prose, non-word |  81.6 % | **68.6 %** | 73.8 % |
  | code, non-word  |  89.1 % | **73.8 %** | 80.9 % |

  Against codebook, the first suggestion is right about ten points more often. The known-typo list
  puts its exact fix first, ahead of the Hunspell suggestions. Recall is 1–3 points lower; that is
  the price of the quiet rules (short words, acronyms, tokens with digits), and the missed rows are
  worth reading one by one. All 2 000 lines with suggestions take 17.6 s, against codebook's 44 s;
  without suggestions, about a second.

  The same run measured the cost of a suggestion. Checking the 1 896 files took 0.16 s; asking for
  suggestions for the 1 921 words found took 44 s more. That is about 23 ms per word, roughly 275
  times the cost of checking. This is why orthra computes suggestions for one word, on request, and
  never while producing diagnostics (`01-architecture.md` §1).

- **UA-GEC.** Ukrainian, measured on the spelling subset in the same way. **Built, as data:**
  - `tools/ua-gec.py` (`just ua-gec`, `just ua-gec-codebook`) samples 1 000 of UA-GEC's one-word
    `Spelling` corrections. Like the English sample, nothing is committed, and the filters are the
    same.
  - A real-word error is recognized by the corpus itself: the error appears, correctly used,
    somewhere in the corrected texts.
  - The bar, from codebook with only `uk`, on 2026-11-05:

    |           |   n | flagged |  top-1 |  top-5 |
    | --------- | --: | ------: | -----: | -----: |
    | non-word  | 599 |  78.3 % | 42.2 % | 63.8 % |
    | real-word | 401 |  13.0 % |  6.7 % |  9.5 % |

    Codebook is clearly weaker in Ukrainian than in English: the fix comes first 42 % of the time
    here, against 59–63 % in English.

  - orthra with uk-UA, from dict_uk 6.8.6 compiled (`just ua-gec-orthra`, on 2026-11-06):

    |           |   n | flagged |  top-1 |  top-5 |
    | --------- | --: | ------: | -----: | -----: |
    | non-word  | 599 |  81.3 % | 49.1 % | 67.4 % |
    | real-word | 401 |  23.2 % |  9.0 % | 19.7 % |

    The corrections are every form of the compiled dictionary within two edits, nearest first and
    the most written first among those as near (`01-architecture.md` §7). With spellbook's
    corrections ordered by frequency the same day, non-word top-1 was 47.1 % and top-5 64.6 %.

    In the dictionary's own order the fix came first 41.7 % of the time and in the first five 63.1
    %, under codebook's bar. The dictionary's corrections come in the order of the edits that made
    them, so `сюда` got `юда, слюда, сода, сюди`. Ten of them, ordered by how often each word is
    written (FrequencyWords' 50 000 Ukrainian forms from OpenSubtitles 2018), put `сюди` first. The
    ten cost 1 % more time than five: suggestions for UA-GEC's 744 unknown words in its 166 test
    texts took 4.47 s against 4.42, and RSS rose by less than 1 MB, read on the first correction
    asked for.

  - What is still missed, of the 599 non-word errors: 112 are not flagged, 11 of them words of three
    letters, which are not checked; 15 get no correction at all (51 with spellbook's); for 67 the
    right one is not among the first five; and for 110 it is second to fifth, which is the context
    tier's to try (`04-context.md`).

- **Tech prose.** A sample of README and doc-comment text, mixed English and Ukrainian, for the
  routing and apostrophe rules.

---

## Level 10. Performance and memory as tests

- `just perf <dir>` measures every budget against real files and exits non-zero when one is
  exceeded. `crates/cli/src/perf/budget.rs` holds the numbers.
- Memory is attributed **by type**, not by RSS, using phrost's `memsize` approach. "uk_UA affix
  table — 31 MB" is a task. "RSS 90 MB" is a feeling.
- A perf-touching change with no measurement is incomplete. Report the p95 and RSS deltas in the
  commit.

---

## Level 11. The context tier, separately

Models are not in CI, and the CI suite never loads one. Every other level uses `FakeScorer`.
`just eval-context` is a separate, opt-in run. It reports:

- the change in top-1 and top-5 suggestions when re-ranking is on (GitHub Typo Corpus);
- `confusable` precision on a held-out set;
- latency p95;
- RSS while the model is loaded, and RSS after the idle unload.

A model change is judged by that report and nothing else. See `04-context.md`.
