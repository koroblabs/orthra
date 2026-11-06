#!/bin/sh
# Packs one platform's release archive: the binary, the project's own licenses, and the licenses of
# everything in the binary that is not orthra's.
#
#   sh tools/dist/pack.sh <target> <path to orthra> <THIRD-PARTY-LICENSES.md> <out dir>
#
# writes <out dir>/orthra-<target>.tar.gz and its .sha256. The archive holds one directory,
# orthra-<target>/, which is the name the Zed extension looks for. In it:
# - orthra, README.md, CHANGELOG.md, LICENSE-MIT and LICENSE-APACHE;
# - THIRD-PARTY-LICENSES.md, every crate linked in, by license (cargo-about, `about.toml`);
# - licenses/<pack>/, each pack's LICENSE.* and SOURCES.md: the dictionaries and word lists
#   embedded in the binary keep their own licenses, and a pack installed later says where it
#   comes from.
set -eu
target=$1
binary=$2
third_party=$3
out=$4

name="orthra-$target"
stage="$out/$name"
rm -rf "$stage"
mkdir -p "$stage/licenses"
cp "$binary" "$stage/orthra"
cp README.md CHANGELOG.md LICENSE-MIT LICENSE-APACHE "$stage/"
cp "$third_party" "$stage/THIRD-PARTY-LICENSES.md"
for pack in packs/*/; do
    tag=$(basename "$pack")
    mkdir -p "$stage/licenses/$tag"
    cp "$pack/SOURCES.md" "$stage/licenses/$tag/"
    for file in "$pack"/LICENSE*; do
        if [ -e "$file" ]; then
            cp "$file" "$stage/licenses/$tag/"
        fi
    done
done
# macOS tar would add extended attributes as `._` files.
COPYFILE_DISABLE=1 tar -C "$out" -czf "$out/$name.tar.gz" "$name"
(cd "$out" && shasum -a 256 "$name.tar.gz" > "$name.tar.gz.sha256")
rm -rf "$stage"
echo "$out/$name.tar.gz"
