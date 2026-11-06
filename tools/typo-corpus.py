#!/usr/bin/env python3
"""The GitHub Typo Corpus, as orthra's eval set for recall and suggestions.

    python3 -I tools/typo-corpus.py fetch      download and verify the corpus into .cache/corpora/
    python3 -I tools/typo-corpus.py sample     write .cache/corpora/typos.tsv (deterministic)
    python3 -I tools/typo-corpus.py codebook   score codebook-lsp on the sample
    python3 -I tools/typo-corpus.py orthra     score orthra on the sample (release build)
    python3 -I tools/typo-corpus.py context    the same, the context model reordering suggestions
                                               (installed: `orthra model install small`)

Nothing from the corpus is committed. Its texts keep the licenses of the repositories they came
from (Hagiwara & Mita, LREC 2020), so the corpus is fetched, sampled and scored locally, and only
this script and the numbers it prints are in the repository. The README's S3 link is dead; the
file comes from the Internet Archive and is checked against the digest the archive recorded.

`is_typo` in the corpus marks any small edit, whitespace included (`function(){` -> `function()
{`), so the sample keeps only edits where exactly one word part changed: camelCase and snake_case
are split, both sides have at least three letters, the two are at most two edits apart, and the
change is not only case. A typo that is itself a word (`form` for `from`) is kept and marked: no
dictionary can see it, so it is the context tier's question and is scored separately.
"""
import collections
import gzip
import hashlib
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import corpuslib  # noqa: E402

CACHE = corpuslib.CACHE
CORPUS = os.path.join(CACHE, "github-typo-corpus.v1.0.0.jsonl.gz")
SAMPLE = os.path.join(CACHE, "typos.tsv")
URL = (
    "https://web.archive.org/web/20200906123950id_/"
    "https://github-typo-corpus.s3.amazonaws.com/data/github-typo-corpus.v1.0.0.jsonl.gz"
)
# The archive's SHA-1 of the payload, in base32, as its CDX index records it.
DIGEST = "IDRMZ4JVCEGPABJ6FFSXE42INRS2LBPW"
PROSE = {"md", "rst", "txt", "adoc", "markdown", "textile", "org"}
PER_KIND = 1000
WORDS = "/usr/share/dict/words"


def fetch():
    corpuslib.fetch(URL, CORPUS, sha1_base32=DIGEST)


PART = re.compile(r"[A-Z]?[a-z]+|[A-Z]+(?![a-z])")


def parts(text):
    return PART.findall(text)


def distance(a, b):
    """Damerau-Levenshtein (optimal string alignment)."""
    d = [[0] * (len(b) + 1) for _ in range(len(a) + 1)]
    for i in range(len(a) + 1):
        d[i][0] = i
    for j in range(len(b) + 1):
        d[0][j] = j
    for i in range(1, len(a) + 1):
        for j in range(1, len(b) + 1):
            cost = a[i - 1] != b[j - 1]
            d[i][j] = min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + cost)
            if i > 1 and j > 1 and a[i - 1] == b[j - 2] and a[i - 2] == b[j - 1]:
                d[i][j] = min(d[i][j], d[i - 2][j - 2] + 1)
    return d[-1][-1]


def one_word_change(src, tgt):
    a, b = parts(src), parts(tgt)
    if len(a) != len(b):
        return None
    changed = [(x, y) for x, y in zip(a, b) if x != y]
    if len(changed) != 1:
        return None
    typo, fix = changed[0]
    if min(len(typo), len(fix)) < 3 or typo.lower() == fix.lower():
        return None
    if distance(typo.lower(), fix.lower()) > 2:
        return None
    return typo, fix


def sample():
    if not os.path.exists(CORPUS):
        sys.exit("no corpus: run `fetch` first")
    words = set()
    if os.path.exists(WORDS):
        with open(WORDS, encoding="utf-8") as f:
            words = {w.strip().lower() for w in f}
    rows, seen = collections.defaultdict(list), set()
    with gzip.open(CORPUS, "rt", encoding="utf-8") as f:
        for line in f:
            commit = json.loads(line)
            for edit in commit["edits"]:
                src, tgt = edit["src"], edit["tgt"]
                if src["lang"] != "eng" or not edit.get("is_typo"):
                    continue
                change = one_word_change(src["text"], tgt["text"])
                if change is None:
                    continue
                typo, fix = change
                context = " ".join(src["text"].split())[:300]
                if (typo, fix, context) in seen:
                    continue
                seen.add((typo, fix, context))
                path = src["path"]
                ext = path.rsplit(".", 1)[-1].lower() if "." in path.rsplit("/", 1)[-1] else "-"
                kind = "prose" if ext in PROSE else "code"
                key = hashlib.sha1(f"{commit['repo']}\0{commit['commit']}\0{path}\0{typo}\0{fix}\0{context}".encode()).hexdigest()
                real = "?" if not words else ("yes" if typo.lower() in words else "no")
                rows[kind].append((key[:12], kind, ext, typo, fix, real, context))
    out = []
    for kind in ("prose", "code"):
        out.extend(sorted(rows[kind])[:PER_KIND])
    corpuslib.write_tsv(SAMPLE, ["id", "kind", "ext", "typo", "fix", "real_word", "context"], out)
    eligible = {k: len(v) for k, v in rows.items()}
    print(f"{SAMPLE}: {len(out)} rows, from {eligible} eligible single-word edits")


def codebook():
    rows = corpuslib.read_tsv(SAMPLE)
    real = lambda row: "real-word" if row["real_word"] == "yes" else "non-word"
    corpuslib.score_typos(rows, ["en_us"], lambda row: (row["kind"], real(row)))


def orthra():
    rows = corpuslib.read_tsv(SAMPLE)
    real = lambda row: "real-word" if row["real_word"] == "yes" else "non-word"
    corpuslib.score_orthra(rows, ["en-US"], lambda row: (row["kind"], real(row)))


def context():
    rows = corpuslib.read_tsv(SAMPLE)
    real = lambda row: "real-word" if row["real_word"] == "yes" else "non-word"
    corpuslib.score_orthra(rows, ["en-US"], lambda row: (row["kind"], real(row)), context=True)


if __name__ == "__main__":
    commands = {"fetch": fetch, "sample": sample, "codebook": codebook, "orthra": orthra, "context": context}
    if len(sys.argv) != 2 or sys.argv[1] not in commands:
        sys.exit(__doc__)
    commands[sys.argv[1]]()
