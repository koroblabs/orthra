"""What the corpus tools share: fetching a file and checking its digest, running codebook-lsp, and
scoring typo rows.

The tools run as `python3 -I tools/<tool>.py`. `-I` keeps the current directory and the data they
read off the import path, so each tool puts this one directory back on its own, deliberately.
"""
import base64
import collections
import hashlib
import os
import re
import subprocess
import tempfile
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, ".cache", "corpora")


def fetch(url, path, sha256=None, sha1_base32=None):
    """Download `url` to `path` once, then check it against whichever digest is given."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    if not os.path.exists(path):
        print(f"downloading {url}")
        urllib.request.urlretrieve(url, path + ".part")
        os.replace(path + ".part", path)
    with open(path, "rb") as f:
        data = f.read()
    if sha256 is not None:
        got = hashlib.sha256(data).hexdigest()
        expected = sha256
    else:
        got = base64.b32encode(hashlib.sha1(data).digest()).decode()
        expected = sha1_base32
    if got != expected:
        raise SystemExit(f"{path}: digest {got}, expected {expected}; delete it and fetch again")
    print(f"{path}: verified")


FINDING = re.compile(r"^\s+(\S+?):(\d+):(\d+)\s+(\S+)(?:\s+->\s*(.*))?$")


def codebook(root, dictionaries, files, suggest=False):
    """Run codebook-lsp's lint over `files` under `root`, with only `dictionaries`, and return its
    findings as (path relative to root, line, column, word, suggestions)."""
    with open(os.path.join(root, "codebook.toml"), "w") as f:
        quoted = ", ".join(f'"{d}"' for d in dictionaries)
        f.write(f"dictionaries = [{quoted}]\nuse_global = false\n")
    command = ["codebook-lsp", "--root", root, "lint"] + (["--suggest"] if suggest else []) + files
    run = subprocess.run(command, capture_output=True, text=True, timeout=3600)
    if run.returncode not in (0, 1):
        raise SystemExit(f"codebook-lsp exited {run.returncode}:\n{run.stderr[-2000:]}")
    found = []
    for line in run.stdout.splitlines():
        m = FINDING.match(re.sub(r"\x1b\[[0-9;]*m", "", line))
        if m:
            path = os.path.relpath(os.path.join(root, m.group(1)), root) if not os.path.isabs(m.group(1)) else os.path.relpath(m.group(1), root)
            suggestions = [s.strip() for s in (m.group(5) or "").split(",") if s.strip()]
            found.append((path, int(m.group(2)), int(m.group(3)), m.group(4), suggestions))
    return found


ORTHRA = os.path.join(ROOT, "target", "release", "orthra")
ORTHRA_FINDING = re.compile(r"^(.+?):(\d+):(\d+): ([a-z-]+): (\S+?)(?: -> (.*))?$")


def orthra(files, languages=("en-US",), suggest=0, context=False):
    """Run the release `orthra check` over `files`, and return its findings as (path, line, column,
    code, word, fixes). Build it first: `cargo build --release -p orthra-cli`. With `context`, the
    suggestions are reordered by the installed context model, as the server does when a code
    action is asked for; what it says about that is printed."""
    if not os.path.exists(ORTHRA):
        raise SystemExit(f"{ORTHRA}: build it first with `cargo build --release -p orthra-cli`")
    # Isolated: built-in settings only, so the numbers do not depend on whose machine this is.
    command = [ORTHRA, "check", "--isolated"] + [a for tag in languages for a in ("--lang", tag)]
    if suggest:
        command += ["--suggest", str(suggest)]
    if context:
        command += ["--context"]
    run = subprocess.run(command + files, capture_output=True, text=True, timeout=3600)
    if run.returncode not in (0, 1):
        raise SystemExit(f"orthra exited {run.returncode}:\n{run.stderr[-2000:]}")
    if context:
        for line in run.stderr.splitlines():
            if "context tier" in line:
                print(line)
    found = []
    for line in run.stdout.splitlines():
        m = ORTHRA_FINDING.match(line)
        if m:
            fixes = [s.strip() for s in (m.group(6) or "").split(",") if s.strip()]
            found.append((m.group(1), int(m.group(2)), int(m.group(3)), m.group(4), m.group(5), fixes))
    return found


def orthra_files(root, languages=("en-US",)):
    """The files `orthra check` reads under `root`, relative to it, in the order it reads them."""
    command = [ORTHRA, "check", "--isolated", "--files"] + [a for tag in languages for a in ("--lang", tag)]
    run = subprocess.run(command + [root], capture_output=True, text=True, timeout=600, check=True)
    return [os.path.relpath(line, root) for line in run.stdout.splitlines() if line]


# The extensions a line keeps when it is written to a file of its own: a line of Markdown reads as
# Markdown, its links and code spans as an editor sees them. Every other line is plain text, since a
# line of code alone is a fragment no grammar parses: written as `.py`, it found 19 % of the code
# typos (2026-11-06).
KEPT_EXTENSIONS = {"md"}


def case_file(work, row):
    """Writes `row`'s line to a file of its own under `work`, named by its id, and returns the path."""
    ext = row.get("ext") if row.get("ext") in KEPT_EXTENSIONS else "txt"
    path = os.path.join(work, f"{row['id']}.{ext}")
    with open(path, "w", encoding="utf-8") as f:
        f.write(row["context"] + "\n")
    return path


def case_id(path):
    return os.path.splitext(os.path.basename(path))[0]


def score_orthra(rows, languages, groups, context=False):
    """`score_typos`, for orthra: each line as a file of its own (`case_file`), recall and top-k from
    its fixes and, for an unknown word, its suggestions; with `context`, those reordered by the
    context model."""
    with tempfile.TemporaryDirectory() as work:
        files = [case_file(work, row) for row in rows]
        found = collections.defaultdict(list)
        for path, _, _, _, word, fixes in orthra(files, languages, suggest=5, context=context):
            found[case_id(path)].append((word, fixes))
    tier = ", the context tier on" if context else ""
    tally(rows, found, groups, f"orthra on {len(rows)} typos, languages {list(languages)}{tier}, each line as a file of its own")


def score_typos(rows, dictionaries, groups):
    """Score codebook on typo rows (`id`, `typo`, `fix`, `context`), each line as a file of its own
    (`case_file`), and print recall, top-1 and top-5 by `groups(row)`. Percentages are of all rows in
    a group."""
    with tempfile.TemporaryDirectory() as work:
        for row in rows:
            case_file(work, row)
        found = collections.defaultdict(list)
        globs = [os.path.join(work, "*.txt")] + [os.path.join(work, f"*.{ext}") for ext in sorted(KEPT_EXTENSIONS)]
        for path, _, _, word, suggestions in codebook(work, dictionaries, globs, suggest=True):
            found[case_id(path)].append((word, suggestions))
    tally(rows, found, groups, f"codebook on {len(rows)} typos, dictionaries {dictionaries}, each line as a file of its own")


def tally(rows, found, groups, title):
    score = collections.Counter()
    for row in rows:
        group = groups(row)
        score[group + ("n",)] += 1
        hits = [s for word, s in found[row["id"]] if word.lower() == row["typo"].lower()]
        if not hits:
            continue
        score[group + ("flagged",)] += 1
        suggestions = [s.lower() for s in hits[0]]
        if suggestions[:1] == [row["fix"].lower()]:
            score[group + ("top1",)] += 1
        if row["fix"].lower() in suggestions[:5]:
            score[group + ("top5",)] += 1
    keys = sorted({k[:-1] for k in score})
    width = max(len(" ".join(k)) for k in keys)
    print(title)
    print(f"{'group':<{width}} {'n':>5} {'flagged':>9} {'top-1':>8} {'top-5':>8}")
    for key in keys:
        n = score[key + ("n",)]
        pct = lambda k: f"{100 * score[key + (k,)] / n:5.1f} %"
        print(f"{' '.join(key):<{width}} {n:>5} {pct('flagged'):>9} {pct('top1'):>8} {pct('top5'):>8}")


def write_tsv(path, header, rows):
    ids = [row[0] for row in rows]
    assert len(set(ids)) == len(ids), "two rows share an id, so one would overwrite the other"
    with open(path, "w", encoding="utf-8") as f:
        f.write("\t".join(header) + "\n")
        for row in rows:
            assert not any("\t" in str(cell) or "\n" in str(cell) for cell in row), row
            f.write("\t".join(str(cell) for cell in row) + "\n")


def read_tsv(path):
    if not os.path.exists(path):
        raise SystemExit(f"{path}: not there yet; run the tool's `sample` first")
    with open(path, encoding="utf-8") as f:
        header = f.readline().rstrip("\n").split("\t")
        return [dict(zip(header, line.rstrip("\n").split("\t"))) for line in f]
