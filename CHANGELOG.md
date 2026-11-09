# Changelog

A release's section is its notes on GitHub: the release workflow publishes the section whose heading
names the tag's version.

## 0.2.0

- checks 45 more languages, each in its comments, its strings and the names a file declares:
  JavaScript and JSX, Scala, Groovy, Swift, Objective-C, C, C++, C#, F#, Dart, Zig, Kotlin, Java,
  Ruby, Lua, Elixir, Erlang, Haskell, OCaml, Elm, R, Julia, PowerShell, shell scripts, SQL, HTML,
  Vue and Svelte components, CSS, SCSS, LESS, XML, reStructuredText, Protocol Buffers, GraphQL, Nix,
  HCL and Terraform, INI, Java properties, Makefiles, CMake and Dockerfiles; a property name in a
  stylesheet is held to known typos (`widht`);
- leaves a template's code out of an HTML page (`{{ }}`, `{% %}`, `{# #}`, `<% %>`): Django's admin
  templates went from 268 findings to none;
- rewrites the selection, not only the sentence at the cursor, and rephrases it, with a model you
  run through a local OpenAI-compatible endpoint, now described in the README; the prompts can be
  your own, and an API key is sent to a server that asks for one;
- **settings:** the built-in model is `context.rank` and a model you run is `context.rewrite`, each
  on its own. 0.1.0's `context.enabled` and `context.http` are no longer read;
- offers to change every same finding in a file at once, and to ignore one line with an
  `orthra: ignore-next-line` comment above it;
- applies every exact fix in a file at once, a known typo's only correction or a word in one script,
  from the menu or when the editor saves: `source.fixAll.orthra`;
- reads a doc comment's markup as markup, whatever the language: `<pre>`, `<code>`, `{@code}`,
  `{@link}`, odoc's `{!refs}`, Scaladoc's `{{{ }}}`, doctests (`>>>`, `iex>`, `scala>`), Haddock's
  `>` lines, and a code block indented in Markdown's way, in a docstring or heredoc too;
- reads a handle or a name as a name: `TODO(nweiz)`, code review's `CR rgrinberg:`, and what follows
  `(c)`, `©`, `Copyright` anywhere in a line, `Created by` and `Written by:`;
- leaves a name qualified by its module unchecked, `os.path.join`, `Vec::new`: it refers to code;
- English's dictionaries are compiled to FSTs when orthra is built: 12.8 MB less memory, a faster
  start, and corrections ranked by how common a word is, so `leter` gets `later` and `letter` before
  `leer` and `lefter`; the right correction comes first a point more often on real typos;
- gives back on macOS what its parsers free, with mimalloc as the process's allocator there: 7 to 10
  MB after checking a large project;
- English is either spelling until a setting names one: en-US and en-GB are built in, and
  `languages = ["en-US"]` makes a project American only;
- test data is not checked: `fixtures/`, `*-fixtures/`, `__fixtures__/` and `testdata/` in every
  language;
- a translation into a language no enabled pack speaks has its text left unchecked:
  `locales/de.json`, `config/locales/devise.de.yml`, `_locales/zh_CN/messages.json`, Android's
  `res/values-de/`, `de.lproj/`, `Strings.de.resx` and `messages_de.properties`;
- more of software English is a word: 86 000 uncommon words from SCOWL size 80, 5 000 names of
  commands from tldr-pages, `un-`, `re-`, `sub-` and six more prefixes on a known word, and plurals
  of the technical vocabulary (`datagrams`). On the projects beside orthra, 13 827 findings became 1
  923, and the typo sample's recall moved by 0.2 points.

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
