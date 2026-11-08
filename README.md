# orthra

A spell-checking language server that is fast and quiet. Written in Rust, for Zed first and any LSP
client after.

orthra checks the prose in your project: Markdown, plain text, commit messages, and the comments,
doc comments, strings and declared names in code. It checks them against real dictionaries, stays
out of the way of everything else, and runs entirely on your machine.

## Why another one

A spell checker that cries wolf gets switched off, and then it catches nothing. orthra is built
around that: every change is measured against a corpus of well-kept projects that should produce no
findings at all.

| measured 2026-11-05/06                          |            orthra | codebook 0.3.42 |
| ----------------------------------------------- | ----------------: | --------------: |
| findings on well-kept projects, same files      | 16–23 times fewer |               — |
| right correction first, real typos from GitHub  |        **71.4 %** |         59–63 % |
| right correction first, with the optional model |        **72.2 %** |               — |
| right correction first, Ukrainian (UA-GEC)      |        **49.1 %** |          42.2 % |
| re-check after a keystroke, p95                 |        0.1–1.1 ms |               — |
| memory for a whole project                      |          25–37 MB |               — |

## What it finds

- **`unknown-word`**: a word no dictionary of its script accepts, in prose. In code, a name you did
  not declare is somebody else's choice, so orthra checks a name where it is declared and only for
  known typos where it is used.
- **`known-typo`**: one of 96 000 known misspellings (from crate-ci/typos), with its exact fix,
  anywhere.
- **`mixed-script`**: a word with a Cyrillic `с` in place of the Latin `c` looks right and is not.
  The fix is the same word in one script.
- **`wrong-layout`**: `руддщ цщкдв` is `hello world` typed with the Ukrainian layout still on. The
  fix is the same keys in the other layout. It only fires when every word of the run is another
  layout's word, so Ukrainian prose stays Ukrainian.

Things that are never words are never checked:

- URLs, paths and emails;
- hashes, UUIDs and versions;
- CLI flags and language tags (`uk-UA`);
- inline code;
- Markdown link targets and front matter;
- test data: `fixtures/`, `*-fixtures/`, `__fixtures__/` and `testdata/`;
- the strings of a translation into a language you have not enabled (`locales/de.json`).

**Languages:**

- `en-US` and `en-GB` are built in, and both are on until a setting names one. They come with 27 000
  names and terms of software from
  [cspell-dicts](https://github.com/streetsidesoftware/cspell-dicts) (`rustup`, `kubectl`, `Gitea`,
  `PostgreSQL`), 5 000 names of commands from [tldr-pages](https://github.com/tldr-pages/tldr)
  (`journalctl`, `wipefs`), SCOWL's uncommon words, and the prefixes and plurals software English
  makes (`unconfigured`, `datagrams`).
- `uk-UA` (from [dict_uk](https://github.com/brown-uk/dict_uk)) is installed on first use. orthra
  compiles it to a 3.9 MB index of all its 3.6 million word forms.
- Words are routed by script, word by word, so Ukrainian comments next to English names need no
  setting.

**Formats:** plain text, Markdown, Git commit messages, Rust, TypeScript and TSX, Python, Go, PHP,
TOML, YAML, JSON and CSS.

## Install

- **Zed:** install the **Orthra** extension. It runs `orthra` from your `PATH` when there is one,
  and otherwise downloads the latest release for your platform, once.
- **A release:** each [release](https://github.com/koroblabs/orthra/releases) has an archive for
  macOS (Apple silicon and Intel) and Linux (x86_64 and arm64, glibc 2.35 or later). Put the
  `orthra` in it on your `PATH`. On macOS, a binary downloaded by a browser is quarantined until
  `xattr -d com.apple.quarantine orthra`.
- **From source,** with Rust 1.99 or later:

  ```sh
  cargo install --locked --git https://github.com/koroblabs/orthra orthra-cli
  ```

## Configure

Everything is optional.

- **`orthra.toml`** at the root of a project:

  ```toml
  languages = ["en-GB", "uk-UA"]   # default: en-US and en-GB; naming one replaces both
  ignore = ["vendor/", "*.lock"]   # gitignore patterns
  severity = "information"         # error | warning (default) | information | hint
  auto-install = false             # never fetch a pack; say how to instead
  ```

- **Words orthra should accept:** one per line, kept sorted so that branches merge cleanly.
  - `.orthra/words.txt` holds the project's words.
  - `~/.config/orthra/words.txt` holds yours.
  - The _Add to words_ code action writes either.
- **One file:** a comment `orthra: ignore word another` makes those words fine in that file.
- **Your machine:** `~/.config/orthra/config.toml` takes the same keys as `orthra.toml`.
- **The editor:** in Zed's settings,

  ```jsonc
  "lsp": { "orthra": { "initialization_options": {
    "languages": ["uk-UA"],                      // added to the project's
    "severity": "hint",
    "context": { "enabled": true }               // the optional model, below
  } } }
  ```

## The optional model

orthra can reorder a word's corrections by the sentence it is in: "Please recieve the **leter**"
offers `letter` first, where the dictionary order puts `leer` first.

- **The model.** It is ELECTRA-small, quantized to 15 MB, and runs in process on the CPU.
- **Off by default.** With `"context": {"enabled": true}` the server downloads it once from a pinned
  revision, checks its SHA-256, and loads it. `orthra model install small` does the same by hand.
- **Only when you ask.** It runs when you open the code-action menu, never as you type. It takes
  about 13 ms, and the dictionary's order stands if it is slower or fails.
- **Memory.** It costs about 20 to 40 MB while it is used. It is unloaded after ten minutes unasked,
  and the memory is given back.
- **What it never does.** It never adds a word the dictionary did not propose, and never changes
  what is reported.

## Command line

```sh
orthra check [--lang en-GB]... [--suggest 5] <path>...   # every file a host claims; exit 1 on findings
orthra check --ratchet <path>                            # CI: fail only when a count rose
orthra explain notes.md:12:7                             # why the word there was or was not reported
orthra pack list | install uk-UA                         # installable languages
orthra model list | install small                        # the optional model
orthra serve                                             # the language server, on stdio
```

## Privacy

- **Nothing you write leaves your machine.**
- **Downloads.** orthra fetches only a language pack or the model, and only when your settings ask
  for it. Each comes from a pinned source, checked by SHA-256, and no text is sent with the request.

## Licenses

- **The code:** MIT or Apache-2.0, at your option.
- **What a release carries:** `THIRD-PARTY-LICENSES.md`, the license of every crate linked into the
  binary, and `licenses/`, each pack's.
- **The data:**
  - Dictionaries and word lists keep their own licenses, recorded beside each pack in
    `packs/<tag>/SOURCES.md`.
  - The Ukrainian dictionary (GPL-3.0) is not carried, only installed.
  - The model is Apache-2.0.

## Working on orthra

- [`docs/`](docs/README.md) is the design: architecture, the test levels, the roadmap and the
  context tier's contract.
- [`CLAUDE.md`](CLAUDE.md) is the operating manual.
- `just pr-ready` is the gate every commit passes.

<!-- The model's example misspells on purpose. orthra: ignore recieve leter -->
