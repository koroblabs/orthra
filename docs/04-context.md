# The context tier

This is the second head. It is the one part of orthra that is an experiment, and an experiment needs
a written contract before it runs. Otherwise it quietly becomes the product, and the product quietly
becomes slow.

---

## 1. The contract

The lexical tier answers **"is this a word?"** The context tier answers two different questions:

- **"Is this the right word here?"** That is a real-word error: `form` for `from`, `компанія` for
  `кампанія`.
- **"Which of these corrections did you mean?"** That is re-ranking.

Seven rules, each enforced by a test or the dependency graph:

1. **It never answers the first question.** It cannot accept or reject a word.
2. **It never proposes a word.** It reorders candidates the lexical tier produced and judges words
   from a confusion set. It never generates spellings, so a model cannot introduce a non-word.
3. **It never blocks.** No diagnostics wait for it, and no code-action response waits longer than
   its budget, 80 ms (`CLAUDE.md` §3), or at all when the client asks on its own. Past the budget
   the lexical order stands. The one exception is the explicit "rewrite" action, which the user
   chose.
4. **It is off by default.** With no model on the machine, orthra is complete.
5. **Its findings carry `source = "orthra/context"`.** A user can tell a dictionary's certainty from
   a model's opinion, and can turn one off without losing the other.
6. **`context-off ⊆ context-on`.** Turning it on never removes, moves or rewords a lexical finding
   (`02-testing.md` level 4).
7. **Text stays on the machine.** Endpoints must be loopback unless the user writes
   `allow_remote = true`. A model is downloaded only when the user asks: `orthra model install`, or
   the settings turning the tier on, which is the consent to fetch it, as enabling a language is
   (the user's choice, 2026-11-06). `auto-install = false` keeps it to the command. Every download
   is pinned by revision and checked by SHA-256, and what is fetched is the model, never text.

---

## 2. Three uses, ranked by value per unit of cost

### A. Re-ranking suggestions (first)

In `textDocument/codeAction`, for each unknown word in the range: its line, cut to 400 bytes around
it (`Window`), and its corrections. A masked LM scores each candidate in the slot, weighing the
lexical order in by a λ tuned on the GitHub Typo Corpus and recorded with the model.

- **Built (2026-11-06).** `orthra-context`'s `Scorer`, `Window` and `rerank`, the server's
  `actions::Plan::rerank`, and a flow test with `FakeScorer`
  (`the_context_tier_reorders_what_a_word_offers`).
- **Only when asked.** It runs when somebody asks for the actions (`triggerKind` 1, or none from an
  older client), never when a client asks on its own as the cursor moves (`triggerKind` 2), which
  Zed does to know whether to show any.
- **Off the server's lock**, on a blocking thread, waited for at most 80 ms.
- **Failure leaves the lexical order.** A scorer that is slower, fails or panics does that, and is
  said once a session in the log.
- **The same corrections, every time.** `rerank` refuses an answer that does not fit them: the wrong
  count, or a score that is not finite.

This is the cheapest use, at one sentence per user action. It is also the most visible, because the
first suggestion becomes the right one more often. It is worth building only if `just eval-context`
shows a top-1 gain that justifies its latency.

### B. Confusables in prose (second)

A pack's `confusables.tsv` lists sets of real words that get swapped:

```
their	there	they're
then	than
loose	lose
affect	effect
компанія	кампанія
ефектний	ефективний
```

**The model is asked only about words in a set**, and only in prose spans (comments, doc comments,
Markdown, commit messages), never identifiers. That makes the work proportional to how often
confusable words appear, not to document size: phrost's "prefilter the index can hold" shape. It
runs in idle time, batched per sentence, and reports `confusable` findings with severity hint.

### C. Rewrite a sentence (third)

This is an explicit code action, "Rewrite sentence (local model)", or "Rewrite selection (local
model)" with text selected, when the selection starts in text meant to be read (2026-11-08,
`what_is_selected_is_offered_and_rewritten_as_selected`). The edit is computed in
`codeAction/resolve` through a localhost endpoint. It is never a diagnostic, and never automatic. It
is also the only place a generative model is used, and the only place grammar reaches orthra.

- **Built (2026-11-06).** `orthra-context`'s `Rewriter`, `Connector` and `sentence::around`, the
  engine's `Engine::sentence_at`, the server's `rewrite` module, and `orthra-ml-http`. Flow test:
  `a_sentence_is_rewritten_when_asked`, with a pretend connector and `FakeRewriter`.
- **Offered only when asked** (`triggerKind` 1, or none from an older client; never 2), with the
  cursor in a sentence of text meant to be read, and only to a client that asked for no kinds or for
  `refactor`. The action, `refactor.rewrite`, carries no edit: its `data` names the document, its
  version and the sentence's range.
- **The sentence** is found in two steps, each a pure function with its own tests:
  - The engine finds the passage from the host's spans. In prose (Markdown, plain text, a commit
    message) it is one paragraph's text, through inline code and markup and across a line that
    wraps, from the start of its first line to the end of its last. In a comment or a doc comment it
    is the comment on the cursor's line, because the next line starts with the comment's marker.
    Code, strings, names and values have no passage.
  - `sentence::around` cuts the sentence out of it: from the end of the one before to its own `.`,
    `!`, `?` or `…` followed by whitespace, or to a blank line. Its edges are its first and last
    words, with at most two quotes, brackets or emphasis marks that stand apart. So `**Bold**` and
    `("quoted")` are kept, and `//`, `*/` and a docstring's `"""` are left out. It reads 1 000 bytes
    either side of the cursor and refuses a longer sentence, so it is never a whole file.
  - `e.g.` ends a sentence. A heading followed by a paragraph with no blank line between them is
    read as one sentence. A wrapped `//` comment is rewritten one line at a time.
- **Resolved off the server's lock**, on a blocking thread, waited for at most 30 s. The language is
  the first enabled pack for the script most of the sentence's letters are in (`und` when none). The
  document's version is checked before the model is asked and again after it answers: a document
  that changed gets no edit.
- **Failure:** no edit. The first failure of a session is shown (`window/showMessage`), and every
  later one is logged. A rewrite that changes nothing is shown as such.
- **The HTTP route** (`orthra-ml-http`) posts to `<endpoint>/chat/completions` at temperature 0,
  with a 20 s timeout and a 256 KiB limit on the answer. The prompt asks for spelling and grammar
  only, with meaning, wording, markup and line breaks kept, and names the language. It says the text
  may be part of a sentence and must not be completed.
  - The answer loses, only when they wrap all of it, a thinking model's `<think>` block, a code
    fence, and quotes the sentence did not have.
  - It is refused when it is empty, cut short, more than 3× the sentence plus 40 characters, or has
    more lines than the sentence.
- **Text stays on the machine** (rule 7). The guard runs when the settings name the endpoint and
  again before every request. It reads the URL with the parser the request is sent with, and the
  request goes to the URL it returns. The host must be in 127.0.0.0/8, `::1` or the literal
  `localhost`.
  - No other name is resolved to decide: `localhost.evil.com`, `127.1` and `0.0.0.0` are refused.
  - Credentials in the URL are refused even when remote is allowed.
  - No proxy is used and no redirect followed.
  - With `allowRemote`, the action's title names the endpoint instead of "local model": a project's
    `.zed/settings.json` can set the editor's settings.
- **Not yet measured with a real model.** Latency and quality are the user's model's. The tests run
  against a server on a local socket.

---

## 3. The seam

```rust
pub trait Scorer: Send + Sync {
    /// Log-scores for each candidate in `slot` of `sentence`. Higher is more likely.
    fn score(&self, sentence: &str, slot: TextRange, candidates: &[&str]) -> Result<Vec<f32>, ScoreError>;
}

pub trait Rewriter: Send + Sync {
    /// `lang` is the tag of the enabled pack the sentence is written in, `en-US`, or `und`.
    fn rewrite(&self, sentence: &str, lang: &str) -> Result<String, ScoreError>;
}

/// What makes a Rewriter from the settings' endpoint: `orthra-ml-http`'s, handed to the server
/// by the binary as the `Runtime` is.
pub trait Connector: Send + Sync {
    fn connect(&self, endpoint: &Endpoint) -> Result<Arc<dyn Rewriter>, ScoreError>;
}
```

`orthra-text` has no `LangTag` type, so the language is a tag as a string.

There are three implementations:

| impl                         | crate                                  | for                                                                                                                                                                                                                 |
| ---------------------------- | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `FakeScorer`, `FakeRewriter` | `orthra-context` (test-only)           | every test except the eval. They are deterministic and **adversarial**: the scorer scores the worst candidate highest, and the rewriter upper-cases, so a test that passes cannot be relying on a model being right |
| in-process MLM               | `orthra-ml-local` (feature `ml-local`) | A and B                                                                                                                                                                                                             |
| OpenAI-compatible endpoint   | `orthra-ml-http` (feature `ml-http`)   | C, and A/B when the endpoint returns logprobs                                                                                                                                                                       |

**The HTTP route** speaks the OpenAI-compatible API that Ollama, llama.cpp's server and LM Studio
all expose. That costs nothing in binary size, and the user manages the model. Re-ranking through it
needs token logprobs, and support for them varies by runtime. Measure it; don't assume it.

**The in-process route** has two runtime candidates, and Phase 5 decides between them by measurement
on the same model:

- **candle** is pure Rust. It has no native library, which makes a Zed extension's single binary
  simpler, and it supports Metal on macOS.
- **ort** uses ONNX Runtime, which is the broadest model support and a fast CPU path, at the cost of
  a native library to ship or download.

---

## 4. Models — candidates to measure, not choices

| need                              | candidates                                                |
| --------------------------------- | --------------------------------------------------------- |
| English MLM re-ranking            | DistilRoBERTa-base (~82M params), ModernBERT-base (~150M) |
| Multilingual, including Ukrainian | XLM-RoBERTa-base (~270M), multilingual MiniLM variants    |
| Rewrite                           | whatever the user runs locally; orthra does not care      |

The numbers that decide are latency p95 per sentence on CPU, RSS while loaded, and the top-1 gain.
**If a 270M multilingual model buys nothing over the lexical ranking on UA-GEC, Ukrainian gets no
context tier.** That is an acceptable outcome.

**Measured for English, 2026-11-06**, by a candle probe outside the repository (not kept).

- **Method.**
  - The GitHub Typo Corpus sample was used, with orthra's first ten corrections for each flagged
    non-word typo.
  - Each candidate was scored in its line by a masked LM: one pass with the candidate's tokens
    masked, and pseudo-log-likelihood (PLL).
  - The score is `model + λ·(-ln(position + 1))`, with λ tuned on half the cases by id and reported
    on the other half.
  - Weights were f32. Latency is per correction asked for, all ten scored in one batch, on a machine
    busy with builds: provisional.

| model                          | weights | top-1, held-out half | CPU p50 / p95 | Metal p50 / p95 |
| ------------------------------ | ------: | -------------------: | ------------: | --------------: |
| lexical order alone            |       — |               68.6 % |             — |               — |
| ELECTRA-small generator, 13.5M |   54 MB |         70.6 % (PLL) |    39 / 94 ms |               — |
| BERT L-4 H-512, 29M            |  116 MB |         70.7 % (PLL) |    21 / 45 ms |     6.9 / 14 ms |
| DistilRoBERTa-base, 82M        |  331 MB |    72.5 % (one pass) |   53 / 118 ms |      12 / 26 ms |
| ModernBERT-base, 150M          |  599 MB |    71.9 % (one pass) |  116 / 648 ms |      30 / 70 ms |

- **The ceiling is the lexical tier's.** A re-ranker cannot add a correction, so top-5 does not
  move, and the best top-1 can reach is 73.9 %: the right word is among the ten for 91 % of the
  flagged typos. DistilRoBERTa recovers about three quarters of what is left.
  - It fixed 71 cases: `throught` thought → through, `wheather` weather → whether.
  - It broke 14 others, each time picking a word that also fits the context: `failes` fails → files,
    `compuler` compiler → computer.
- **Size buys little.** 13.5M parameters get half of what 82M get, and 150M get no more. The lexical
  prior adds a point at most.
- **Quantized, the same day.** The probe's own encoder was used, bit-identical to candle's in f32,
  with every linear layer in f16 or candle's `QMatMul`, weights in GGUF, and 4 CPU threads, as
  orthra runs. Top-1 is the one-pass scorer with the prior, on the held-out half. Latency is one
  correction asked for, ten candidates in one batch, at load average 4–8 (provisional).

| model, weights      |     on disk |      top-1 |     CPU p50 / p95 | Metal p50 / p95 | RSS after load |
| ------------------- | ----------: | ---------: | ----------------: | --------------: | -------------: |
| ELECTRA-small, f32  |     54.2 MB |     71.0 % |    13.5 / 25.4 ms |    5.7 / 7.1 ms |        +100 MB |
| ELECTRA-small, Q8_0 | **14.8 MB** | **71.1 %** | **6.3 / 17.3 ms** |    4.3 / 6.1 ms |     **+16 MB** |
| ELECTRA-small, Q4_0 |     10.1 MB |     70.5 % |     7.4 / 18.3 ms |    4.1 / 5.8 ms |         +11 MB |
| DistilRoBERTa, f16  |      164 MB |     72.6 % |    17.9 / 37.8 ms |    4.8 / 7.4 ms |        +314 MB |
| DistilRoBERTa, Q8_0 | **88.8 MB** | **72.5 %** | **8.8 / 25.3 ms** |    4.0 / 7.0 ms |    **+121 MB** |

- **Q8_0 costs no accuracy and halves CPU time.** f16 and Q8_0 are within 0.3 points of f32; Q4_0
  costs half a point and is no faster.
- **Reading the file needs no `unsafe`.** mmap saves only the read buffer, and candle copies the
  tensors anyway.
- **The peak is the LM head, not the weights.** ELECTRA Q8_0 peaks at 94 MB while scoring, because
  the probe computes `[rows, 30 522]` logits for every position. Only the masked positions need
  them.
- **A 32-token window costs nothing, and saves only 10–15 % of p95.** Lines are short (median 18
  tokens), and the p95 is ten candidates times their tokens.
- **A real word in the wrong place is not a finding a small model can make quietly.**
  ELECTRA-small's discriminator, given each real-word typo in its line and its fix in the same line,
  separates them with an AUC of 0.76.
  - At a threshold that 1 % of fixes cross, it catches 13.5 % of the typos, and 0.6 % of the words
    of the zero corpus's Markdown fire: about 400 alarms in 71 000 words.
  - Held to 0.01 % of clean words, it catches about 1 %.
  - The generator prefers the fix to the typo in 77–79 % of these lines, which is no quieter.
- **Decided for English:** a context tier built on ELECTRA-small, Q8_0, as the default model (+2.5
  points top-1, 15 MB, CPU p95 17 ms). DistilRoBERTa-base, Q8_0, is the larger choice (+4.0 points,
  89 MB). There is no confusables finding (B) from either.
- **What the lexical tier misses is mostly not ranking.** 289 of the 1 614 non-word typos are not
  flagged at all:
  - 135 are real words: `creates` → create, `serves` → servers. Those are the confusables of B, not
    unknown words.
  - 123 sit in code: backticks, identifiers (`resyscInterval`), or what reads as a path
    (`Inflate/Defalte`).
  - 31 are shorter than four letters.

---

## 5. Scheduling and resources

- **One context worker** (a small pool if measurement says so) with a queue of
  `(document, version, sentence)` jobs. A job whose version is stale is dropped before it runs.
- **Idle time only for B.** It starts after the document has been quiet for the configured delay.
  Each document has a time budget per revision.
- **Lazy load, idle unload.** A model loads on first use and unloads after `idle-unload` (default 10
  minutes). RSS goes back to the lexical baseline, and `just eval-context` checks that it does.
- **Merging.** Results are added to the current version's findings. With pull diagnostics the server
  asks the client to re-pull, and with push it re-publishes.
- **Failure is silence.** A model that errors, times out or is missing produces no context findings
  and one log line. It never produces a lexical change. That is the one place "fail quietly" is
  correct, because the tier is optional by contract.

---

## 6. Configuration

**Built (2026-11-06), split in two (2026-11-08):** the editor's initialization options, as
`.zed/settings.json` writes them. The built-in model and a model the user runs answer different
questions at different speeds, and each has its own key: either, both or neither may be on.

```jsonc
"lsp": { "orthra": { "initialization_options": {
  "context": {
    // Re-ranking by the built-in model. Off unless given: `true`, or "small" by default and
    // unloaded after `idleUnload` seconds unasked (600).
    "rank": { "model": "small", "idleUnload": 600 },
    // "Rewrite sentence", and "Rewrite selection" and "Rephrase selection". Absent means no such
    // action. `endpoint` is Ollama's unless given, `model` is required, `allowRemote` is false
    // unless it is `true` itself, `apiKey` is sent as a bearer token, and `prompts` replace
    // orthra's, `{language}` naming the language.
    "rewrite": {
      "endpoint": "http://127.0.0.1:11434/v1", "model": "qwen3:4b", "allowRemote": false,
      "apiKey": "...", "prompts": { "rewrite": "...", "rephrase": "..." }
    }
  }
} } }
```

- **0.1.0's keys** (`enabled`, `model`, `idleUnload`, `http`) are not read.
- **The rewrite actions** need only `rewrite`. Rephrasing is offered for a selection only, since a
  rephrasing nobody selected would be one more entry in every menu.
- **An API key** is never written to a log: an endpoint prints it as `<api key>`.
  - The endpoint is checked at `initialize`. One that cannot be had is said once: no model named,
    not on this machine, or a binary without `ml-http`.
  - It is read from the editor's settings only, never from a project's `orthra.toml`.
- **The HTTP client** comes from the binary's `ml-http` feature, on by default. It adds no
  dependency the binary does not have already: ureq is there for installing packs. It costs nothing
  at runtime without an endpoint in the settings. It never sends anything until somebody chooses the
  action.

- **Turning it on.** On `initialized` the server, in the background, finds the model under
  `$XDG_DATA_HOME/orthra/models/<name>/<version>/`. It installs the model if it is missing, with a
  progress bar: the sources `models/<name>/install.toml` pins, quantized by the binary's runtime.
  Then it loads the model and logs "the context tier is on". A model that cannot be had is said
  once, and the tier stays off.
- **By hand.** `orthra model list` and `orthra model install small [--from <file>]...`.
- **The runtime.** It comes from the binary's `ml-local` feature (on by default, 2.8 MB of a 27 MB
  binary). A binary without it says so and stays off.

Measured on 2026-11-06, the real server with ELECTRA-small, `orthra serve` driven over stdio:

- **Install:** 0.48 s from the pinned files, at a peak of 121 MB, the 54 MB download included.
- **Load:** 0.02 s.
- **An asked code action:** p50 13.0 ms and p95 13.8 ms, against 0.3 / 0.5 ms with the tier off.
  "Please recieve the leter" now offers `letter` first, where the lexical order put `leer`.
- **Memory.** The model counts 16.5 MB: 15.2 MB of weights and the vocabulary. The server's
  footprint, in the same session:

  |                             | footprint |
  | --------------------------- | --------: |
  | tier off                    |    9.2 MB |
  | model loaded                |     29 MB |
  | after 120 asked actions     |     49 MB |
  | unloaded after `idleUnload` |     22 MB |
  - **The allocator is lspkit-heap's** (`lspkit_heap::Heap`, installed by the binary). It maps a
    large block from the system and gives it back. `release_freed()` runs after a load, which frees
    the file read whole, and after an unload. Without it, the footprint stayed at 56 MB once loaded
    and 76 MB after use.
  - **Two threads.** One, two or four scoring threads made no difference to the footprint, and two
    gave the best p95.
  - **What stays after an unload,** 13 MB above the tier-off footprint, is small blocks the system's
    allocator keeps.

- **Idle unload (built).** A model no code action has asked for in `idleUnload` seconds (600 unless
  set) is unloaded: its weights, and the threads it computes on. The next asked action is answered
  in the lexical order and loads the model again in the background, with no install.

Still to build, as first written:

```toml
[context]
confusables = true                 # not built: a small model is not quiet enough (§4)
```

`[context.rewrite]` in `orthra.toml` is not built, deliberately. A repository's own settings must
not be able to choose where its reader's text is sent.

---

## 7. How it is judged

`just eval-context`, and nothing else (`02-testing.md` level 11):

**Built (2026-11-06).** It scores the typo sample through the product twice:

- `orthra check --suggest 5` as it is;
- with `--context`, which loads the installed model and reorders each unknown word's corrections as
  the server does for an asked code action, in the word's line.

It prints the model's size and the reordering's latency. The first run, on all 1 614 non-word typos,
with the vocabulary already offered as corrections:

|                                                               | top-1 code / prose | top-5 code / prose |
| ------------------------------------------------------------- | -----------------: | -----------------: |
| the tier off                                                  |      70.7 / 70.6 % |      76.4 / 75.6 % |
| ELECTRA-small, Q8_0                                           |  **71.7 / 71.7 %** |      76.4 / 75.6 % |
| the tier off, cspell's 107 lists as vocabulary (the same day) |      71.4 / 71.4 % |      77.1 / 76.2 % |
| ELECTRA-small, Q8_0, the same                                 |  **72.2 / 72.2 %** |      77.1 / 76.2 % |

- **The gain is +0.8 to +1.0 point over the lexical tier as it now is.** The probe measured +2.8
  over the lexical order of the morning, which the vocabulary and the edit order have since
  improved.
- **Ten candidates reordered, and five shown, did no better** (71.6 %), and were slower.
- **The model is only as good as what it is offered.** With the larger vocabulary it first scored
  70.2 / 70.7 %, a point _under_ the tier off: the languages' lists hold plain English, and offered
  from them at two edits, `Schedulor` got `Schedule` beside `Scheduler`. The model prefers the
  shorter, likelier word, and lost 46 typos it had right to win 31. A word the dictionary accepts is
  now offered from the vocabulary only when nothing nearer is (`orthra-packs`, `vocabulary_near`).
- **Latency:** p50 12.6 ms over the 332 words with more than one correction; p50 12.9 and p95 34 ms
  over 343 with the larger vocabulary.
- **Size:** 16.5 MB.

What the table measures:

- top-1 and top-5 with re-ranking off and on, on the GitHub Typo Corpus and on UA-GEC;
- `confusable` precision on a held-out set, where the bar is that a false confusable is rarer than a
  false `unknown-word` on the zero corpus;
- p95 latency per sentence, RSS while loaded, and RSS after unload.

Any change to a model, prompt, λ or confusion set reports all of these numbers before and after. A
context tier that is not measurably better is turned off, not tuned until it feels better.
