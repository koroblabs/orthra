# The incumbent corpus

These are cases about what the incumbent spell checker for Zed, codebook, is said to get wrong. Each
case is the answer orthra should give, written as the snapshot a test would print
(`docs/02-testing.md` level 0 and level 7). Phase 3 does not exit while one of them still fails.

A case file is a fixture with a short header:

- `hypothesis:` which of H1–H5 below it tests, or `reported` for a case somebody sent.
- `source:` where the case came from.
- `incumbent:` what codebook does with it, as measured.
- `why:` why orthra's answer is the right one.

The subject carries the `expect` flag, so its text and its expected findings are one artifact.
`cargo test -p orthra-testkit corpus` checks that every case parses and is written in canonical
form, and `orthra-engine`'s `incumbent` tests that orthra gives each answer.

## The hypotheses

The people who use codebook report it as not good enough. That is the reason orthra exists, and it
is a feeling, not a specification. These five hypotheses say what the complaint might mean. None is
a claim about codebook; each is a set of cases run against both servers.

| #   | hypothesis                                                                                      | what would show it                                                      |
| --- | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| H1  | **Noise.** Technical words, identifier shapes, escapes and URLs come back as misspellings       | the zero corpus (`docs/02-testing.md` level 9), scored for both servers |
| H2  | **Suggestions.** Candidates are ranked without regard to the project or the sentence            | GitHub Typo Corpus top-1/top-5 for both                                 |
| H3  | **Multilingual.** A Ukrainian comment in an English codebase is noise, or is not checked at all | mixed-language cases                                                    |
| H4  | **Cost.** Latency or memory on large files and workspaces                                       | `just perf` for both, over the same files                               |
| H5  | **Friction.** Config, the word list, the actions on offer                                       | a written list of what a user had to do by hand                         |

If the reported cases do not fit H1–H5, the hypotheses were wrong and this table gets rewritten.

## Where it stands

Cases 01–15 are **probes**, written from the hypotheses. Cases 16–19 are **reported**: Artem, who
uses codebook daily, said it "does not understand apostrophes, contractions" and that "every UUID
lights up like a garland" (2026-11-05). Measured, codebook 0.3.42 reads curly apostrophes and UUIDs
correctly, so the forms he meant are elsewhere: an escaped apostrophe in a single-quoted string
(`didn\'t`), which orthra got wrong too and now reads, and random identifiers of every other kind
(nanoid, ULID, Stripe ids, a JWT, an access key), where codebook makes 24 findings and orthra none.
More reported cases are still wanted: a file, a word, what was wrong.

Measured on codebook-lsp 0.3.42, installed with `--locked`, on 2026-11-05. Without `--locked` it
panics at startup on 14 of 15 cases (`CLAUDE.md` §8.1). It **agrees on 9 of 15** and **differs on
6**:

| case                                | tests    | orthra should say                                    | codebook says                                                                   |
| ----------------------------------- | -------- | ---------------------------------------------------- | ------------------------------------------------------------------------------- |
| 01-escapes-and-placeholders         | H1       | nothing: `\nworld`, `%d ms`, `\tdone`                | nothing ✓                                                                       |
| 02-identifier-shapes                | H1, H2   | `recieve` in a declaration, once                     | the same, receive first ✓                                                       |
| 03-use-sites-are-not-ours           | H1       | nothing about another crate's API                    | nothing ✓                                                                       |
| 04-urls-paths-hashes                | H1       | nothing                                              | **`orthra`, inside a path**                                                     |
| 05-markdown-code-is-not-prose       | H1, H2   | `definately`, and nothing in code                    | `definately`, **plus `qzx` from inline code and `xyzzy` from the fenced block** |
| 06-ukrainian-comment                | H3       | `Будьласка`                                          | the same, «Будь ласка» first ✓                                                  |
| 07-ukrainian-without-the-pack       | H3       | nothing                                              | **all five Cyrillic words, with no suggestions**                                |
| 08-mixed-script                     | H3       | a Cyrillic letter inside a Latin word, named as such | both words, **as ordinary misspellings** (`сount` → count)                      |
| 09-technical-vocabulary             | H1       | nothing                                              | **`dedupe`, `kubectl`**                                                         |
| 10-acronyms-and-brands              | H1       | nothing                                              | **`CLI`**                                                                       |
| 11-known-typos-keep-their-case      | H2       | three known typos, each fix in its case              | the same, right first suggestion, case kept ✓                                   |
| 12-possessives-hyphens-contractions | H1       | nothing                                              | nothing ✓                                                                       |
| 13-commit-message                   | H1, H2   | `teh`                                                | the same ✓                                                                      |
| 14-numbers-and-units                | H1       | nothing                                              | nothing ✓                                                                       |
| 15-camel-case-declarations          | H1, H2   | `recieve` in `recieveTimeoutMs`                      | the same ✓                                                                      |
| 16-curly-apostrophes                | reported | nothing                                              | nothing ✓                                                                       |
| 17-contractions-in-escaped-strings  | reported | nothing                                              | **`didn`, `aren`**                                                              |
| 18-uuids-in-every-form              | reported | nothing                                              | nothing ✓                                                                       |
| 19-random-identifiers               | reported | nothing                                              | **24 fragments: `GXR`, `NDEKTSV`, `Nffr`, `AKIAIOSFODNN`…**                     |

What this says about the hypotheses:

- **H1, noise: partly confirmed by the probes, confirmed at scale.** Five false positives in four of
  the ten H1 cases: a word inside a path, code in Markdown, technical words and an acronym. Escapes,
  identifier shapes, use sites, contractions and units were right. On the zero corpus codebook makes
  31.5 findings per 1 000 lines on phrost and 62.7 on ripgrep, about one every six lines of
  Markdown, most of them the project's own vocabulary: its name, its dependencies, the identifiers
  it declares, and their possessives.
- **H2, suggestions: easy typos fine, real ones mediocre.** The seven probed typos all got the right
  first suggestion. On 2 000 real typos from the GitHub Typo Corpus, codebook flags 84–90 % of the
  non-word ones and puts the fix first for 59 % in prose and 63 % in code (`docs/02-testing.md`
  level 9).
- **H3, multilingual: confirmed.** A Ukrainian comment with only English enabled is five false
  positives with no suggestions, and a Cyrillic letter inside a Latin word is an ordinary
  misspelling, `сount` → `count`.

So quietness on code in prose, paths and vocabulary, and routing by script, were where orthra had to
win, in that order; suggestion quality was a requirement, not a selling point.

## Measuring codebook

Once `codebook-lsp` is installed, run it over each case's subject with the same languages. Write
what it reports into that case's `incumbent:` line, in words: "silent", "flags `nworld` and
`tdone`", "suggests `receive` third". If codebook already gets a case right, keep the case anyway.
It stops being evidence for a hypothesis, and stays a regression test for orthra.
