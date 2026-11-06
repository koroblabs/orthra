#!/bin/sh
# Whether a built orthra runs on this machine: it says its version, and finds a known typo.
#
#   sh tools/dist/smoke.sh <path to orthra>
set -eu
orthra=$1
"$orthra" --version
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
printf 'Teh value is here.\n' > "$work/smoke.md"
# `check` exits 1 when it reports something, which is what this asks of it.
set +e
"$orthra" check --isolated "$work/smoke.md" > "$work/out.txt"
code=$?
set -e
cat "$work/out.txt"
if [ "$code" != 1 ] || ! grep -q 'known-typo: Teh' "$work/out.txt"; then
    echo "smoke: expected exit 1 and a known typo, got exit $code" >&2
    exit 1
fi
