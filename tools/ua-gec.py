#!/usr/bin/env python3
"""UA-GEC's spelling errors, as orthra's Ukrainian eval set.

    python3 -I tools/ua-gec.py fetch      download and verify UA-GEC at a pinned commit
    python3 -I tools/ua-gec.py sample     write .cache/corpora/ua-gec/typos.tsv (deterministic)
    python3 -I tools/ua-gec.py codebook   score codebook-lsp on the sample, with only `uk`
    python3 -I tools/ua-gec.py orthra     score orthra on the sample, with only uk-UA (release
                                          build; installs uk-UA first if it is not installed)

UA-GEC (Grammarly, CC BY 4.0) marks each correction in place as `{error=>edit:::error_type=Tag}`.
The sample keeps `Spelling` corrections where one word became another: three or more letters on
both sides, at most two edits apart, and not only a change of case. `в=>у` and `Я=>я` are spelling
in UA-GEC's sense, and not a dictionary's question, so they fall out. Documents marked sensitive
are left out.

A Ukrainian error that is itself a word (`погоджуйтеся` for `погоджуєтеся`) needs the context tier,
as `form` for `from` does in English. With no Ukrainian word list on hand, such a word is
recognized by the corpus itself: an error that appears somewhere in the corrected texts is a word
somebody used correctly.

Nothing from UA-GEC is committed. Its license would allow it with attribution, but every corpus here
is fetched and sampled the same way, so one rule covers them all.
"""
import collections
import csv
import hashlib
import io
import os
import re
import sys
import tarfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import corpuslib  # noqa: E402

COMMIT = "4757f72f192c4a41e4c8fb1d9690a948f87cf6d6"
URL = f"https://github.com/grammarly/ua-gec/archive/{COMMIT}.tar.gz"
SHA256 = "94693ae6a97c87fc327a17bfd088b05f921a0288a62f7ba71b49ee3691290411"
HOME = os.path.join(corpuslib.CACHE, "ua-gec")
ARCHIVE = os.path.join(HOME, f"ua-gec-{COMMIT[:12]}.tar.gz")
SAMPLE = os.path.join(HOME, "typos.tsv")
SIZE = 1000

ANNOTATION = re.compile(r"\{([^{}]*?)=>([^{}]*?):::error_type=([^{}]*?)\}")
WORD = re.compile(r"[\w'’ʼ-]+")
# Letters only, with an apostrophe or a hyphen between them: `80хх` is not a spelling of a word.
LETTERS = re.compile(r"[^\W\d_]+(?:['’ʼ-][^\W\d_]+)*")


def fetch():
    corpuslib.fetch(URL, ARCHIVE, sha256=SHA256)


def documents():
    """(name, annotated text) for each non-sensitive document, read straight from the archive."""
    with tarfile.open(ARCHIVE) as archive:
        names = archive.getnames()
        meta = next(n for n in names if n.endswith("/data/metadata.csv"))
        rows = csv.DictReader(io.TextIOWrapper(archive.extractfile(meta), encoding="utf-8"))
        sensitive = {row["id"] for row in rows if row["is_sensitive"] == "1"}
        for name in sorted(names):
            if "/data/gec-only/" in name and "/annotated/" in name and name.endswith(".ann"):
                if os.path.basename(name).split(".")[0] in sensitive:
                    continue
                yield name, archive.extractfile(name).read().decode("utf-8")


def resolve(text, side):
    """The text with every annotation replaced by its error (side 1) or its edit (side 2)."""
    return ANNOTATION.sub(lambda m: m.group(side), text)


def distance(a, b):
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


def sample():
    docs = list(documents())
    corrected = set()
    for _, text in docs:
        corrected.update(w.lower() for w in WORD.findall(resolve(text, 2)))
    rows, seen, kinds = [], set(), collections.Counter()
    for name, text in docs:
        for line in text.split("\n"):
            source = resolve(line, 1)
            for m in ANNOTATION.finditer(line):
                error, edit, kind = m.group(1), m.group(2), m.group(3)
                kinds[kind] += 1
                if kind != "Spelling":
                    continue
                if not (LETTERS.fullmatch(error) and LETTERS.fullmatch(edit)):
                    continue
                if min(len(error), len(edit)) < 3 or error.lower() == edit.lower():
                    continue
                if distance(error.lower(), edit.lower()) > 2:
                    continue
                # The error's place in the source line: the text before it, resolved to errors.
                at = len(resolve(line[: m.start()], 1))
                context = " ".join(source[max(0, at - 150): at + len(error) + 150].split())
                if (error, edit, context) in seen:
                    continue
                seen.add((error, edit, context))
                key = hashlib.sha1(f"{name}\0{error}\0{edit}\0{context}".encode()).hexdigest()[:12]
                real = "yes" if error.lower() in corrected else "no"
                rows.append((key, error, edit, real, context))
    rows.sort()
    corpuslib.write_tsv(SAMPLE, ["id", "typo", "fix", "real_word", "context"], rows[:SIZE])
    print(f"{SAMPLE}: {min(SIZE, len(rows))} rows of {len(rows)} eligible, from {len(docs)} documents; "
          f"Spelling is {kinds['Spelling']} of {sum(kinds.values())} corrections")


def codebook():
    rows = corpuslib.read_tsv(SAMPLE)
    real = lambda row: "real-word" if row["real_word"] == "yes" else "non-word"
    corpuslib.score_typos(rows, ["uk"], lambda row: ("uk", real(row)))


def orthra():
    rows = corpuslib.read_tsv(SAMPLE)
    real = lambda row: "real-word" if row["real_word"] == "yes" else "non-word"
    corpuslib.score_orthra(rows, ["uk-UA"], lambda row: ("uk", real(row)))


if __name__ == "__main__":
    commands = {"fetch": fetch, "sample": sample, "codebook": codebook, "orthra": orthra}
    if len(sys.argv) != 2 or sys.argv[1] not in commands:
        sys.exit(__doc__)
    commands[sys.argv[1]]()
