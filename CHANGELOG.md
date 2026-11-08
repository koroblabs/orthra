# Changelog

A release's section is its notes on GitHub: the release workflow publishes the section whose heading
names the tag's version.

## Unreleased

- checks CSS: its comments, the text of `content:`, the class and id names, custom properties and
  `@keyframes` names a stylesheet declares, and known typos in property names (`widht`).

## 0.1.0

The first release.

**The language server**, `orthra serve`, for Zed and any LSP client:

- checks the prose of plain text, Markdown and commit messages, and the comments, doc comments,
  strings and declared names of Rust, TypeScript and TSX, Python, Go, PHP, TOML, YAML and JSON;
- reports a name where it is declared, never where somebody else's API is used, and accepts what the
  project declares: its name, its dependencies, its authors, the parts of its identifiers;
- skips what is never a word: URLs, paths, emails, hashes, UUIDs, versions, CLI flags, language
  tags, inline code, link targets and front matter;
- re-checks only what a keystroke changed: 0.1 to 1.1 ms at the 95th percentile on real projects;
- offers fixes, adds a word to the project's words or the user's, or ignores it in one file.

**What it finds:**

- `unknown-word`: a word no dictionary of its script accepts;
- `known-typo`: one of 96 000 known misspellings (crate-ci/typos), with its exact fix;
- `mixed-script`: a word with a letter from another script (`сount` with a Cyrillic `с`), fixed to
  one script;
- `wrong-layout`: `руддщ цщкдв` typed with the Ukrainian layout still on, fixed to `hello world`.

**Languages:** en-US and en-GB built in, with 27 000 names and terms of software from cspell-dicts.
uk-UA is installed on first use from dict_uk and compiled to a 3.9 MB index of its 3.6 million word
forms. Words are routed to a language by their script, word by word.

**The optional model,** off by default: ELECTRA-small, 15 MB, run in the process, reorders a word's
corrections by its sentence when the code actions are asked for. The server downloads it from a
pinned revision when the settings turn it on, and unloads it after ten minutes unasked. With a local
endpoint (Ollama, llama.cpp), an action rewrites the sentence at the cursor.

**The command line:** `orthra check` (with `--ratchet` for CI), `orthra explain`,
`orthra pack install`, `orthra model install`.

Measured against codebook-lsp 0.3.42 on the same files: 16 to 23 times fewer findings on well-kept
projects, and the right correction first for 71.4 % of real typos from GitHub against 59–63 %; 72.2
% with the model.
