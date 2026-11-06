#!/usr/bin/env python3
"""orthra beside the other spell checkers, on the same files (`docs/02-testing.md` level 7).

    python3 -I tools/compare.py <dir> [--lang en-US]...   orthra, codebook-lsp, and typos and
                                                          cspell when they are installed
    python3 -I tools/compare.py <dir> --sample            and a sample of each bucket to label
    python3 -I tools/compare.py --zero                    every zero-corpus project, and totals

Every tool is a black box run over the same list of files: the files under <dir> that orthra has a
host for, so each is asked about exactly what orthra is. A finding is matched across tools by its
file, its line and its word, lower-cased and without a possessive `'s` (codebook keeps it, orthra
drops it), never by column, since each counts columns its own way. Each finding then falls into a
bucket against codebook: `both`, `orthra only` or `codebook only`.

Labeled findings turn the buckets into precision and recall. `corpora/compare/<project>.tsv` holds
`path`, `line`, `word`, `label` (`real` or `noise`) and `by`, one finding a row, labeled by reading
the line it is on. Precision is real findings over findings, per tool, from the labeled rows of its
buckets; recall is a tool's real findings over all the real findings either tool made. Until a
bucket's sample is labeled its numbers say so, rather than guess.

Nothing is installed by this script: typos and cspell are run if they are on PATH, and said to be
missing otherwise.
"""
import collections
import os
import random
import re
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import corpuslib  # noqa: E402

LABELS = os.path.join(corpuslib.ROOT, "corpora", "compare")
# How many findings of each bucket `--sample` prints for labeling: enough for a precision to a few
# points, few enough to read each one.
SAMPLE = 40


def key(word):
    word = word.lower()
    for possessive in ("'s", "’s"):
        if word.endswith(possessive):
            word = word[: -len(possessive)]
    return word


def typos(root, files):
    if not shutil.which("typos"):
        return None
    run = subprocess.run(["typos", "--format", "brief", *files], cwd=root, capture_output=True, text=True)
    pattern = re.compile(r"^(.+?):(\d+):(\d+): `([^`]+)` ")
    return [(os.path.normpath(m.group(1)), int(m.group(2)), m.group(4))
            for m in map(pattern.match, run.stdout.splitlines()) if m]


def cspell(root, files):
    if not shutil.which("cspell"):
        return None
    run = subprocess.run(["cspell", "lint", "--no-progress", "--no-summary", "--no-color", *files],
                         cwd=root, capture_output=True, text=True)
    pattern = re.compile(r"^(.+?):(\d+):(\d+) - Unknown word \((.+?)\)")
    return [(os.path.normpath(os.path.relpath(m.group(1), root) if os.path.isabs(m.group(1)) else m.group(1)),
             int(m.group(2)), m.group(4))
            for m in map(pattern.match, run.stdout.splitlines()) if m]


def buckets(ours, theirs):
    """Each of `ours` and `theirs` as (path, line, word) triples, split into what both found, what
    only the first did and what only the second did. A word twice on one line is two findings."""
    a = collections.Counter((p, l, key(w)) for p, l, w in ours)
    b = collections.Counter((p, l, key(w)) for p, l, w in theirs)
    both, only_a, only_b = [], [], []
    for k in sorted(set(a) | set(b)):
        shared = min(a[k], b[k])
        both += [k] * shared
        only_a += [k] * (a[k] - shared)
        only_b += [k] * (b[k] - shared)
    return both, only_a, only_b


def read_labels(project):
    path = os.path.join(LABELS, f"{project}.tsv")
    if not os.path.exists(path):
        return {}
    return {(row["path"], int(row["line"]), key(row["word"])): row["label"] for row in corpuslib.read_tsv(path)}


def precision(rows, labels):
    labeled = [labels[k] for k in rows if k in labels]
    if not labeled:
        return None, 0
    return sum(1 for label in labeled if label == "real") / len(labeled), len(labeled)


def line_of(root, path, line):
    with open(os.path.join(root, path), encoding="utf-8", errors="replace") as f:
        for i, text in enumerate(f, 1):
            if i == line:
                return text.rstrip("\n")
    return ""


def compare(root, languages, sample=False):
    """orthra and codebook on the files orthra reads under `root`: the buckets, printed, and each
    bucket's size with its share of real findings as labeled, for a total across projects."""
    project = os.path.basename(root.rstrip("/"))
    dictionaries = [tag.lower().replace("-", "_") for tag in languages]
    ours = [(os.path.relpath(p, root) if os.path.isabs(p) else os.path.normpath(p), l, w)
            for p, l, _, _, w, _ in corpuslib.orthra([root], languages)]
    files = corpuslib.orthra_files(root, languages)
    claimed = set(files)
    theirs = [(p, l, w) for p, l, _, w, _ in corpuslib.codebook(root, dictionaries, [os.path.join(root, f) for f in files])
              if p in claimed]

    both, orthra_only, codebook_only = buckets(ours, theirs)
    named = (("both", both), ("orthra only", orthra_only), ("codebook only", codebook_only))
    labels = read_labels(project)
    lines = sum(1 for f in files for _ in open(os.path.join(root, f), encoding="utf-8", errors="replace"))
    print(f"== {project}: {len(files)} files orthra has a host for, {lines} lines, languages {languages}")
    print(f"   orthra {len(ours)} findings, codebook {len(theirs)}")
    print(f"   both {len(both)}, orthra only {len(orthra_only)}, codebook only {len(codebook_only)}")
    for name, rows in named:
        words = collections.Counter(w for _, _, w in rows)
        print(f"   {name:<14} " + ", ".join(f"{w} {n}" for w, n in words.most_common(16)))

    # Each bucket's share of real findings, from its labeled sample, scaled to the bucket. With
    # none real among n labeled, the share is at most 3/n at 95 % (the rule of three), said so.
    stats = {"findings": {"orthra": len(ours), "codebook": len(theirs)}, "buckets": {}}
    for name, rows in named:
        share, n = precision(rows, labels)
        stats["buckets"][name] = (share, n, len(rows))
        if share is None:
            print(f"   labeled {name:<14} none of {len(rows):>5}")
        else:
            bound = f" (at most {100 * 3 / n:4.1f} % at 95 %)" if share == 0 and n else ""
            print(f"   labeled {name:<14} {n:>4} of {len(rows):>5}, real {100 * share:5.1f} %{bound}")
    report(stats)

    for name, tool in (("typos", typos), ("cspell", cspell)):
        found = tool(root, files)
        if found is None:
            print(f"   {name}: not installed, so not compared")
            continue
        b, a_only, o_only = buckets(ours, found)
        print(f"   {name}: {len(found)} findings; both {len(b)}, orthra only {len(a_only)}, {name} only {len(o_only)}")

    if sample:
        rng = random.Random(project)
        print(f"\n# a sample of each bucket to label, {SAMPLE} at most: path, line, word, then the line")
        for name, rows in named:
            unlabeled = [k for k in dict.fromkeys(rows) if k not in labels]
            for path, line, word in sorted(rng.sample(unlabeled, min(SAMPLE, len(unlabeled)))):
                print(f"{name}\t{path}\t{line}\t{word}\t{line_of(root, path, line).strip()[:160]}")
    return stats


def report(stats, indent="   "):
    """Precision and recall from bucket shares, or nothing while a bucket has no labels."""
    buckets_ = stats["buckets"]
    if any(share is None and size for share, _, size in buckets_.values()):
        return
    real = {b: (share or 0) * size for b, (share, _, size) in buckets_.items()}
    union = sum(real.values())
    for name, own in (("orthra", "orthra only"), ("codebook", "codebook only")):
        found = real["both"] + real[own]
        total = stats["findings"][name]
        p = found / total if total else 0
        r = found / union if union else 0
        print(f"{indent}{name:<8} {total:>6} findings, about {found:5.1f} real: precision {100 * p:5.2f} %, "
              f"recall of what either found {100 * r:5.1f} %")


def zero():
    """Every project of the zero corpus, with the languages codebook is given there, and the totals."""
    import importlib.util
    spec = importlib.util.spec_from_file_location("zero_corpus", os.path.join(os.path.dirname(__file__), "zero-corpus.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    total = {"findings": collections.Counter(), "buckets": collections.defaultdict(lambda: [0.0, 0, 0])}
    for name, project in module.PROJECTS.items():
        languages = [{"en_us": "en-US", "en_gb": "en-GB"}[d] for d in project["dictionaries"]]
        stats = compare(os.path.join(module.ZERO, name), languages)
        total["findings"].update(stats["findings"])
        for bucket, (share, n, size) in stats["buckets"].items():
            total["buckets"][bucket][0] += (share or 0) * size
            total["buckets"][bucket][1] += n
            total["buckets"][bucket][2] += size
    print("== the zero corpus, all five projects")
    buckets_ = {b: (real / size if size else 0, n, size) for b, (real, n, size) in total["buckets"].items()}
    for b, (share, n, size) in buckets_.items():
        print(f"   {b:<14} {size:>6} findings, {n:>4} labeled, about {share * size:5.1f} real")
    report({"findings": total["findings"], "buckets": buckets_})


def main(argv):
    args, languages, sample = [], [], False
    it = iter(argv)
    for arg in it:
        if arg == "--lang":
            languages.append(next(it))
        elif arg == "--sample":
            sample = True
        else:
            args.append(arg)
    if args == ["--zero"] or argv == ["--zero"]:
        zero()
    elif len(args) == 1:
        compare(os.path.abspath(args[0]), languages or ["en-US"], sample)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
