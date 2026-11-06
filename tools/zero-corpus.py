#!/usr/bin/env python3
"""The zero corpus: real, well-kept projects whose prose is believed to be correct.

    python3 -I tools/zero-corpus.py fetch      phrost from ../phrost at a pinned commit, ripgrep,
                                               composer and helix from release tarballs, GitHub's
                                               starter workflows from a pinned commit's tarball,
                                               into .cache/corpora/zero/
    python3 -I tools/zero-corpus.py codebook   what codebook-lsp reports on them
    python3 -I tools/zero-corpus.py orthra     orthra beside codebook, on the files orthra has a host for
    python3 -I tools/zero-corpus.py ratchet    orthra's findings on each project, by code, against
                                               corpora/zero/<project>.toml: exit 1 if one rose.
                                               Add --update-baseline to record a rise, and say why

Every finding on this corpus is either a false positive or a real typo in somebody's project, and
`docs/02-testing.md` level 9 says what to do with each. Until orthra checks words, this measures
the incumbent: how many findings it makes per thousand lines, where, and on which words.
"""
import collections
import io
import os
import subprocess
import sys
import tarfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import corpuslib  # noqa: E402

ZERO = os.path.join(corpuslib.CACHE, "zero")
PROJECTS = {
    "phrost": {
        "git": os.path.join(os.path.dirname(corpuslib.ROOT), "phrost"),
        "rev": "de013033dab612c44aee781e50973a0273347676",
        "paths": ["Cargo.toml", "README.md", "CLAUDE.md", "docs", "crates"],
        "license": "MIT OR Apache-2.0",
        # phrost is written in British English: `modelled`, `judgement`, `colour`.
        "dictionaries": ["en_us", "en_gb"],
        "extensions": (".rs", ".md"),
    },
    "ripgrep": {
        "url": "https://github.com/BurntSushi/ripgrep/archive/refs/tags/15.2.0.tar.gz",
        "sha256": "7605249d3eb0d5f170e3414498e3344e26b1e7a147aec518b57090b80036a562",
        "license": "Unlicense OR MIT",
        "dictionaries": ["en_us"],
        "extensions": (".rs", ".md"),
    },
    # PHP, for phrost's users. The release tarball leaves out the tests and the docs, so this is
    # the code of `src/` and nothing else.
    "composer": {
        "url": "https://github.com/composer/composer/archive/refs/tags/2.10.3.tar.gz",
        "sha256": "803a3de1c096cc6ba4b568de3859c6debcfd95a09b80404ee2fb6a19ddd22680",
        "license": "MIT",
        "dictionaries": ["en_us"],
        "extensions": (".php", ".json"),
    },
    # TOML: a table of 250 languages and some 170 themes. The whole project is taken, so what its
    # Rust declares counts, as it would in an editor.
    "helix": {
        "url": "https://github.com/helix-editor/helix/archive/refs/tags/25.07.1.tar.gz",
        "sha256": "27c8bc3eba46bc7bab1e3629c6b28ff94882eeff17366b3ea69cd8ceffba7541",
        "license": "MPL-2.0",
        "dictionaries": ["en_us"],
        "extensions": (".toml",),
    },
    # YAML: GitHub's starter workflows, the YAML a developer edits most. Each is a template for a
    # language or a vendor, so their comments name a great many products.
    "starter-workflows": {
        "url": "https://github.com/actions/starter-workflows/archive/e3c451d60f119b71caebf13c98ac45da6e15b4b7.tar.gz",
        "sha256": "e0736177f6e406e87799fa6a1a42d564b9d6e9aac6ca3e6272f90221747cc2cb",
        "license": "MIT",
        "dictionaries": ["en_us"],
        "extensions": (".yml", ".yaml", ".json"),
    },
}
# `corpuslib.codebook` writes its config into each project's root. It is ours, not the project's.
OURS = "codebook.toml"


def counted(project, path):
    return path.endswith(project["extensions"]) and os.path.basename(path) != OURS


def patterns(project):
    return [f"**/*{ext}" for ext in project["extensions"]]


def extract(archive, into):
    """Unpack regular files and directories only, refusing absolute paths, `..` and links: the
    archive is somebody else's data."""
    os.makedirs(into, exist_ok=True)
    for member in archive.getmembers():
        parts = member.name.split("/")
        if member.name.startswith("/") or ".." in parts or not (member.isfile() or member.isdir()):
            continue
        name = "/".join(parts[1:]) if archive.github else member.name
        if not name:
            continue
        target = os.path.join(into, name)
        if member.isdir():
            os.makedirs(target, exist_ok=True)
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with archive.extractfile(member) as src, open(target, "wb") as dst:
                dst.write(src.read())


def fetch():
    for name, project in PROJECTS.items():
        into = os.path.join(ZERO, name)
        if os.path.isdir(into):
            print(f"{into}: present")
            continue
        if "git" in project:
            if not os.path.isdir(project["git"]):
                raise SystemExit(f"{project['git']}: no checkout there, so {name} cannot be taken")
            tar = subprocess.run(
                ["git", "-C", project["git"], "archive", "--format=tar", project["rev"], *project["paths"]],
                capture_output=True, check=True,
            ).stdout
            archive = tarfile.open(fileobj=io.BytesIO(tar))
            archive.github = False
        else:
            path = os.path.join(ZERO, os.path.basename(project["url"]))
            corpuslib.fetch(project["url"], path, sha256=project["sha256"])
            archive = tarfile.open(path)
            archive.github = True  # one top-level directory, `ripgrep-15.2.0/`, stripped
        extract(archive, into)
        print(f"{into}: {name} ({project['license']})")


def place(root, path, line, column):
    """Where a finding sits: Markdown, a comment, a string literal, or code (an identifier).

    Read off the source line with no parser, so it is a classification and not a proof: a `//`
    inside a string counts as a comment, and a raw string as code. Good enough to say where the
    noise comes from, which is all it is for."""
    if path.endswith(".md"):
        return "markdown"
    with open(os.path.join(root, path), encoding="utf-8", errors="replace") as f:
        for _ in range(line - 1):
            f.readline()
        text = f.readline()
    before = text[: column - 1]
    if "//" in before:
        return "comment"
    if before.count('"') % 2 == 1:
        return "string"
    return "code"


def is_test(path):
    parts = path.split("/")
    return "tests" in parts or path.endswith("/tests.rs") or parts[-1].startswith("test")


def codebook():
    for name, project in PROJECTS.items():
        root = os.path.join(ZERO, name)
        if not os.path.isdir(root):
            raise SystemExit(f"{root}: not there yet; run `fetch` first")
        total = 0
        for directory, _, files in os.walk(root):
            for file in files:
                if counted(project, file):
                    with open(os.path.join(directory, file), encoding="utf-8", errors="replace") as f:
                        total += sum(1 for _ in f)
        found = corpuslib.codebook(root, project["dictionaries"], [os.path.join(root, p) for p in patterns(project)])
        found = [f for f in found if counted(project, f[0])]
        places = collections.Counter()
        words = collections.defaultdict(collections.Counter)
        possessive = in_tests = 0
        for path, line, column, word, _ in found:
            where = place(root, path, line, column)
            places[where] += 1
            words[where][word.lower()] += 1
            possessive += word.lower().endswith("'s")
            in_tests += is_test(path)
        print(f"== {name}, dictionaries {project['dictionaries']}: {total} lines of {' and '.join(project['extensions'])}, "
              f"{len(found)} findings, {1000 * len(found) / total:.1f} per 1 000 lines")
        print(f"   in test files {in_tests}; ending in 's {possessive}")
        for where, n in places.most_common():
            top = ", ".join(f"{w} {c}" for w, c in words[where].most_common(14))
            print(f"   {where:<9} {n:>6}  {top}")


def orthra():
    """orthra beside codebook on the same files: every file of the project's extensions."""
    for name, project in PROJECTS.items():
        root = os.path.join(ZERO, name)
        if not os.path.isdir(root):
            raise SystemExit(f"{root}: not there yet; run `fetch` first")
        files = sorted(
            os.path.join(directory, file)
            for directory, _, names in os.walk(root)
            for file in names
            if counted(project, file)
        )
        lines = collections.Counter()
        for path in files:
            with open(path, encoding="utf-8", errors="replace") as f:
                lines[os.path.splitext(path)[1]] += sum(1 for _ in f)
        total = sum(lines.values())
        # The same languages codebook was given, so the two are compared on the same terms.
        languages = [{"en_us": "en-US", "en_gb": "en-GB"}[d] for d in project["dictionaries"]]
        # The project root, not the files: orthra finds the manifest and learns every file from it.
        found = [f for f in corpuslib.orthra([root], languages) if counted(project, f[0])]
        theirs = corpuslib.codebook(root, project["dictionaries"], [os.path.join(root, p) for p in patterns(project)])
        theirs = [f for f in theirs if counted(project, f[0])]
        print(f"== {name}: {len(files)} files, {total} lines of {' and '.join(project['extensions'])} (languages {languages})")
        for label, findings in (("orthra", found), ("codebook", theirs)):
            by_ext = collections.Counter(os.path.splitext(f[0])[1] for f in findings)
            per = "  ".join(f"{ext} {1000 * by_ext[ext] / lines[ext]:5.1f}" for ext in project["extensions"] if lines[ext])
            print(f"   {label:<8} {len(findings):>6} findings, {1000 * len(findings) / total:5.1f} per 1 000 lines  ({per})")
        codes = collections.Counter(code for _, _, _, code, _, _ in found)
        words = collections.Counter(word.lower() for *_, word, _ in found)
        print(f"   orthra by code: {dict(codes)}")
        print("   orthra's most frequent: " + ", ".join(f"{w} {n}" for w, n in words.most_common(30)))


BASELINES = os.path.join(corpuslib.ROOT, "corpora", "zero")


def ratchet(update=False):
    """`orthra check --ratchet` on every project, with the languages codebook is given, against a
    baseline committed in this repository: the projects live in `.cache/`, which is not."""
    rose = []
    for name, project in PROJECTS.items():
        root = os.path.join(ZERO, name)
        if not os.path.isdir(root):
            raise SystemExit(f"{root}: not there yet; run `fetch` first")
        languages = [{"en_us": "en-US", "en_gb": "en-GB"}[d] for d in project["dictionaries"]]
        command = [corpuslib.ORTHRA, "check", "--isolated", "--baseline", os.path.join(BASELINES, f"{name}.toml")]
        command += [arg for tag in languages for arg in ("--lang", tag)]
        command += ["--update-baseline"] if update else []
        result = subprocess.run(command + [root], capture_output=True, text=True)
        print(f"== {name}")
        print(result.stdout, end="")
        if result.returncode == 1:
            rose.append(name)
        elif result.returncode != 0:
            raise SystemExit(result.stderr)
    if rose:
        sys.exit(f"findings rose on {', '.join(rose)}")


if __name__ == "__main__":
    commands = {"fetch": fetch, "codebook": codebook, "orthra": orthra}
    if len(sys.argv) >= 2 and sys.argv[1] == "ratchet" and set(sys.argv[2:]) <= {"--update-baseline"}:
        ratchet(update=len(sys.argv) == 3)
    elif len(sys.argv) != 2 or sys.argv[1] not in commands:
        sys.exit(__doc__)
    else:
        commands[sys.argv[1]]()
