#!/usr/bin/env python3
"""Builds the language packs that ship inside orthra, from pinned sources.

    python3 -I tools/packs.py en-US     write packs/en-US/ from the sources below
    python3 -I tools/packs.py en-GB     write packs/en-GB/, sharing en-US's typos and vocabulary

A pack is data (`docs/01-architecture.md` §4), and every file in it comes from a source pinned by
commit or tag and checked by digest, with that source's license beside it. Run it again after
changing a pin. The output is committed; this script is how it can be re-made, and how it can be
checked that nobody edited it by hand.
"""
import hashlib
import io
import os
import re
import sys
import urllib.request
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

DICTIONARY = "https://raw.githubusercontent.com/wooorm/dictionaries/8cfea406b505e4d7df52d5a19bce525df98c54ab/dictionaries/en"
TYPOS = "https://raw.githubusercontent.com/crate-ci/typos/v1.52.0"

EN_US = {
    "en_US.aff": (f"{DICTIONARY}/index.aff", "8ae1f19d4840d957728ad90555d5a8dff6cc5c046279c95ff0c00fc0a0136c7b"),
    "en_US.dic": (f"{DICTIONARY}/index.dic", "f0b1a234bd178bdd01875b2a392a9647f888b8fe879f79c52aae62c2759b3647"),
    "LICENSE.dictionary": (f"{DICTIONARY}/license", "2a7e8d8ae9e8facc84818546ae2a8d83aec5e9c80a675ff789acd1c338b53b3d"),
    "LICENSE.typos": (f"{TYPOS}/LICENSE-MIT", "6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6"),
}
# SCOWL's en_US-large, size 70, from the same 2020.12.07 release as the size-60 dictionary above.
# Only what it adds is taken (`en_US-rare.dic`): words a reader writes that size 60 lacks
# (`integrations`, `vanishingly`, `naïve`), and uncommon words that are also common misspellings
# (`incudes`, `algin`), which is why the engine accepts a rare word only when no pack lists it as a
# known typo. Its README is its license, the same SCOWL and WordNet terms as `LICENSE.dictionary`.
SCOWL_LARGE = (
    "https://downloads.sourceforge.net/project/wordlist/speller/2020.12.07/hunspell-en_US-large-2020.12.07.zip",
    "5a93617312ffb7cccc54c8fd8a3e6712eb0e9cd4a69f1d358b82eb4d352072b7",
)
CSPELL_COMMIT = "72b65202e1b774153003577073303c4ef1d72e4e"
CSPELL = f"https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/{CSPELL_COMMIT}/dictionaries"
VOCABULARY_LICENSE = (f"{CSPELL}/software-terms/LICENSE", "6245ecd81789d113a600fa0e4f5f503e6d1b4f425860f6047dc945560dfcf13c")
# cspell-dicts' word lists for software (all MIT, Street Side Software): the terms, tools, companies,
# languages, frameworks and packages programmers write, which a general dictionary does not have.
# Taken whole they cut the zero corpus's findings by a fifth (2026-11-06, `docs/02-testing.md`
# level 9). Left out: the lists of API identifiers (Java, C++), science, and natural languages.
VOCABULARY = [
    ("software-terms/src/coding-compound-terms.txt", "a3c2d70c9684592ce7fa45ffc838e0abc8be9bbe45d5a039c2c4cc9ef2f8602e"),
    ("software-terms/src/coding-terms.txt", "5f9f1a401e83006543d7ea4470591f1bb003d41ed8e43155115f3dc0c3b71598"),
    ("software-terms/src/computing-acronyms.txt", "5ad1c671d8f9212b6f5c1c169f19f21e480c98b34a763f9f6ed0995ad6d63ff3"),
    ("software-terms/src/cybersecurity-terms.txt", "1c00c6015c5d7970e763756ecc7cfafc1a50eb98f44f544309bdd3c234cb0b45"),
    ("software-terms/src/network-os.txt", "75749b9e90aea3ffaaa51535ad4f3d9b96610f68577cc5ef612a388687ceb9c9"),
    ("software-terms/src/network-protocols.txt", "b2b8f0f0cb21dce4b6c26ed8b044dc99b1ebae5e95e85bfff520265e8874f0eb"),
    ("software-terms/src/posix-errno.txt", "939a13f53c1b407a81f41df6fb4e86ce7e7633f9b308f9eb5cf3589fe18f6c61"),
    ("software-terms/src/posix-signals.txt", "32f76414f3c766d71cec398069db204d6e10038a369d00133d6119ec92c638f7"),
    ("software-terms/src/software-services.txt", "4125c5bd54e9435111dcb3d9c4f00b2998f391ea6421aac057287e5d23c6ff1f"),
    ("software-terms/src/software-terms-alternative.txt", "f14a117078732e10e278d139c7ab918b09c50caa349e87918e34cacf0e5ebb3a"),
    ("software-terms/src/software-terms-legacy.txt", "ecae9afb634c0b1466fb01f1ec8025981c3c37c4e0eca9f1b434fc0a759846a0"),
    ("software-terms/src/software-terms.txt", "b48e669314a91f6ef4ad3f8797bc51aa0292dd41300bc2082a49316c8cdfa251"),
    ("software-terms/src/software-tools.txt", "dc9036b00cf72b0881274207e3a9802c4b840c4560c14d43994b919cfaf47aa3"),
    ("software-terms/src/source-files-networking.txt", "4a005ebe8cd4459a2b2f87ee7408847867d9ac6ada94cb3b15eaa7a197bc9e63"),
    ("software-terms/src/source-files-software.txt", "147640618869b31065504f0cfbfb8c163bd35d312b740d77c0eee6810302d3f9"),
    ("companies/src/companies.txt", "37fc1e822e837e45c3e79e93089e1e40dd7e19073423e95abaad036f879ccd79"),
    ("rust/src/crates.txt", "19f7cdb14a5e2d8017c5378d8643fa2011369ff2a6f787c792152e65a7ebe9be"),
    ("rust/src/rust.txt", "06caf50cb5cbce0253b2ea56197ea5b6190112323fa371f9ae536b3297023220"),
    ("golang/src/additional-terms.txt", "7f2c1bde0c3148ee9751c52f679c22c1c7464864a1962e79bd61f1653bbfb7d2"),
    ("golang/src/allowed-terms.txt", "70fe58d4517af912e60b2a20e5a925c894724305367c9006a9819236c41e5058"),
    ("golang/src/go-lib.txt", "466f0bf2f837146829566ca7186926d41860fdf0f651be2258b8da5c34c5581d"),
    ("golang/src/go.txt", "3ba7186fdbf04783c93653504b24947d7fea6e3f34c44d242176e0760252c64e"),
    ("python/src/additional_words.txt", "e46cb3f3627065f6eb305e19cea6cf55c1ce04945af35cf7d14fc9bbcc9b247f"),
    ("python/src/common_packages.txt", "f896ef5fce8e7c7ae29bd0a2f4df5f3ec54a14bf1e2782ce6f4d6f358ffba6d6"),
    ("python/src/data_science.txt", "1b08eea6791cf86434a3916065ae9af857464fd7c1f99b6e4b4b53b9c62ecc4f"),
    ("python/src/legacy.txt", "07374056321fa22901e94d6d579ba7356331e11c00154fc24f44073c9ef2f614"),
    ("node/src/additional_words.txt", "2da55184d40f1046644835dd8d8b6d686860aa4bd76d06ed9b8ff91faf6b168e"),
    ("node/src/exclude_words.txt", "352c790757c5a9f1a4d7bea3a3818e013b1a9de079327d145327d75c848f604a"),
    ("node/src/node-old.txt", "677bb9ed2a16c6627daf79191a54943917bda37330fe7ca6d856de623832553f"),
    ("node/src/node.txt", "5c1ad4ebcc7bc1283fea1773dbe87483f71cf72978834c2faaf7e4e671ceaecd"),
    ("node/src/split_words.txt", "7427877c40fb0361401248f9c96abe6117396bc6ab16811b5b1706274c02443e"),
    ("typescript/src/domGlobalEventHandlers.txt", "af8fc91f7b6086d7534c891c165b5cd42389dbdf78a21993e0295433bfde7bc1"),
    ("typescript/src/domGlobalEvents.txt", "51f126ac04c558a55ceaeb682b84e6de02084b6947859a30bd2752f6a339b3b3"),
    ("typescript/src/ecosystem-terms.txt", "7509ebd1ccfde848621256bd7e0862ef8d38f5033ff4392a819cae1fec0366de"),
    ("typescript/src/typescript.txt", "11a73493bdad131c6ccdb5ae84f440c1bbbed07ef68049cfb351cd5ca38dfeef"),
    ("php/src/allowed-terms.txt", "14bda00626e7931144d968e909f68fc439f9b16a1e6f7d5ff7d72cda77f09a8d"),
    ("php/src/php-functions.txt", "f0154c48c3f4ac22fd6fa9472242c29e2b41cee3eab450e7840d700fd86ffe55"),
    ("php/src/php.txt", "d6fc5167fa17a080699e0c3c81d8dfd8bf41c3a0cce693fe53135d615f5de056"),
    ("dotnet/src/allowed-terms.txt", "c7975b05e5cecd81bd268b15f0f05f60107466c643bac6fda43197c69fd780f5"),
    ("dotnet/src/dotnet-no-split.txt", "da281d167d0d645cbac673f901b0719678ee6034465abb7a65f7a1ab493aae4c"),
    ("dotnet/src/dotnet.txt", "04a94eb827529c61c0f01a021fccf31e271c6613998ce9f58c905c23f654d06a"),
    ("html/src/html.txt", "a8ae6198cd2123565100af1e52c83e232c13fe2174df7d583c2d3b7083b526c7"),
    ("html/src/svg.txt", "cd1796616da5c2179a3647cea92cc12f2d04b8654a17bde724cc231e79441913"),
    ("css/src/css.txt", "276a90e22f386bf0783bddce7ba6a029acbe6a4de479acc73a2df59f2e029196"),
    ("sql/src/common-terms.txt", "4d47936eda5e58a7708705b1258e9829c0981cb3179bef721b14f918435b5327"),
    ("sql/src/sql.txt", "70c4c10a30af15354e9320c55b7c7d65edeef5411562cb3fd1b7506dde700e40"),
    ("sql/src/tsql.txt", "ec64c61d7a1d7abc9678ab33606fc266589e1a63debfda9f985257e70f20e13c"),
    ("aws/src/aws.txt", "57328affb56e5a9bd556534d915ef93e482eb5439dbdcdec551eaa8de13a0ba3"),
    ("docker/src/docker-words.txt", "99b9b6c56fb58d221b6df6a14e3ffd7cc440ad5dee7c16161751cf9dc7888371"),
    ("k8s/src/k8s.txt", "ff1b1ffeee0131bd608f1cf4ef8310e9e1755a984b76c4eedd4d826f378c5f4f"),
    ("git/src/additional_words.txt", "54b83ca3aadb472b8263fe354c0e422ee998d8b82ec7f534d09296c4fb5d376c"),
    ("git/src/terms.txt", "abaaf9a6fb284345a4d8e2b2c93b55aac3cdabbd84a3657684e4465182b22439"),
    ("terraform/src/terraform.txt", "080b0a7fc148c0bc20f82ca541ee2d00fef9caac69303c7762b6993786032a36"),
    ("shell/src/command-words.txt", "8f6a9464a0aab2368e1f73cfafdf19cdd4091c97de7fb8252ca006a66b052c4f"),
    ("shell/src/glob-words.txt", "b78bea25135b3d54e0f326c822c8228f1aff550e7abba065f79a42f613e8d302"),
    ("shell/src/miscellaneous-words.txt", "884387974cbb06c5a950001a453c5981ee5d562b47345511809e54efcbb056ac"),
    ("shell/src/shell-bash-words.txt", "0edc6ac35ee57e34c65aac97a0810b23ab750aa6c9d47997c412e86a8f891b27"),
    ("shell/src/shell-sh-words.txt", "a492e9474fa0a7c401515fb590b640d6fc6f3a74397c586463c201d19a5c9432"),
    ("shell/src/shell-zsh-words.txt", "d4122761cd8ddc84e30a8fcfb0f4cb61a4cedc9ee393ad5ff6fab7f3410e49a9"),
    ("makefile/src/makefile.txt", "235f939e4c653830c29a5ac594d5b0f276965540b35ba7524fd359a48d5fc8ff"),
    ("fullstack/src/fullstack.txt", "ec10f0ae2513b9e749e42929dfdade6e13bd2f66fb784dbd83c7798e329cba2e"),
    ("fullstack/src/tools-and-services.txt", "126a6cb7e906e9d4a0551079a36802b907aca873c9d1e591da1c102847c1c004"),
    ("filetypes/src/filetypes.txt", "9714cb1357d0ff02d5e3f571bc9a3381cbfbee98ef24494cf27d533dec27e2ea"),
    ("google/src/google.txt", "46a59d53145464ea74c452002cdf4d721f318a545734dfcd25e37c7a2b1eedee"),
    ("google/src/legacy.txt", "afb7a67d0beeec488fa2568f495ecea70fa5d3b60583c6612452cd80b6b3afd8"),
    ("google/src/misc.txt", "dcb68a1a619457040a1a94f1880a6c5e5da5623cdec13927a12b639079718e1a"),
    ("redis/src/redis-commands.txt", "bf3306211eb42a5f29038015d0980ab79f3c8e3457b7b878f29f5e97703e9f81"),
    ("fonts/src/fonts.txt", "d8f727442b8242a4e573186ee3ff7aaa0dd017989406f48443c835ea16665971"),
    ("ruby/src/allowed-terms.txt", "adfb16a50f4779987c293cdfaf4e03f4fc3e3ac1448b8ae2620812624e7b3e55"),
    ("ruby/src/gems.txt", "8f4f20feda5e72b4f4359cd8146453dad48e993c6379c590d9ea6a6ceae10c4f"),
    ("ruby/src/ruby.txt", "42557109ca3d9225c3aaff9e09fcfcf3b245d55f661175fd8a59f828c7aed52a"),
    ("swift/src/swift.txt", "4991db7f60763ba25041b7144dc7b74d76c2bdff8bd9f5a4d3d44dbfecf205b4"),
    ("kotlin/src/kotlin.txt", "29a55cd30363ddff03a04127f3f5b8cdf7427e3776d8797272477badafe93745"),
    ("scala/src/allowed-terms.txt", "ea40fc2480f4f9c65f85fe54a4ec2810e7fa503dd29825b25abcddf681d5dbfa"),
    ("scala/src/scala.txt", "7e1ccff1ebfaa8bc7d190a9c30878af0122859385c6706eb06042e45266de28f"),
    ("dart/src/dart-common.txt", "711715147d2259fbb63cc947fd074d7705c45568c3af5fe86bd9892438411aa8"),
    ("dart/src/dart-pubdev.txt", "4f84f3daf77050db31ce4fba5fb1f9d95f78b2a4262762610c9ea110d9249191"),
    ("dart/src/dart-wxyz.txt", "a2d99e3c823344d961ea63d984b5c15046517c0fa31f0f0179769673ff65a1dd"),
    ("dart/src/dart.txt", "e425f63423c31d977507f4b0b0068f20945ebee1de7504056ba5a9e97791894c"),
    ("lua/src/lua.txt", "ab8b0d7bac40ee03449cf4e3e8a30f24d10bab85e3feaf2826792429be14c038"),
    ("latex/src/latex.txt", "cf44541e816c7b3956aaa3df6fc0211c666176256a18bccde4142db436afdf6c"),
    ("powershell/src/allowed-terms.txt", "6dd262f754439ce98c901214934e587fd8a00be8b2b1cdf0b62cdade3d93379b"),
    ("powershell/src/powershell.txt", "01864992c232a89afabffb588991ac60b4785635704b97b8cebf9d33e4c21939"),
    ("vim/src/vim.txt", "0294dac88a0f74daf438a05fc31153f7a182a67a7ceff0eeaac653ef0ba058fe"),
    ("data-science/src/data-science-models.txt", "7945d134c60a190c896ea018a372eed4a4d36f690126c9a5ab5349c07a18e615"),
    ("data-science/src/data-science-terms.txt", "f7a4ffc2a5d20524efa987123b52cdf3c647b276d332ca4a93dde0a55f3da1c3"),
    ("data-science/src/data-science-tools.txt", "d2d53ffbb2d9ec6a0eea0b7f8f371447802cfd7564337f73aac555ae6203578f"),
    ("mime-types/src/additional-mime-types.txt", "f82c6845cf5fc6d1958182ce3344aa1b5286a9423fc9dcef667200986e8304b0"),
    ("mime-types/src/mime-types.txt", "64189ef0aa26a714d5236723af8f39c4404872ef936401c2a3872b565a4f08d0"),
    ("npm/src/imports.txt", "092323aeae3c6af418a616ca9e3d7389a0ccad1806caafd1f47a7d09f7b0755a"),
    ("npm/src/npm-keywords.txt", "1bf127dce9d9c88e554e5991a0424a23038ef767487788b949c5c7af90cf5a33"),
    ("npm/src/npm.txt", "e37c994aa68876347e228f2a3d9ddbfc833beaefe99faaa39c5870e593a9efbb"),
    ("npm/src/people.txt", "e147cb9611c877c70ee6a1a3a7a22702cfd53975d2fda66dc9d624400d696065"),
    ("npm/src/tools.txt", "3903a81f4bc16635faf25fa0c2ca497932a6f09c49b40150b802313348c3c277"),
    ("elixir/src/elixir.txt", "f27ef2f08f2375909b17f8b48d84dad79ef225d6c89df15db7427d7c2f391f96"),
    ("haskell/src/haskell.txt", "d505bb14e870a7b8ce9379c87a633620b79a3de349eb304ce2159a618f779f20"),
    ("zig/src/zig.txt", "ce6d569bde6efaa8dc7b962ba1b1065951f069d4d500222d78bb158223bf9da6"),
    ("julia/src/julia-keywords.txt", "715f26d930653cc4ef81a5f0bb95e1a00a5d3f0f4d3a2abb248131ac1fd2aa26"),
    ("julia/src/julia.txt", "a7f5e8a15a739e1635f1a33b7ca19b97a329907376700201ce3c5c4fbba30c53"),
    ("r/src/r.txt", "6d13141a68fc30d611400808999783aa1c5fd64dd3e6789af6d2dfd0e30ad794"),
    ("clojure/src/clojure.txt", "086c14e3d254b7534a168ad103e93f34161836c378bff0cf6ef5c5bf03bcacb9"),
    ("fsharp/src/from-fsharp-src.txt", "991b076f2d2dc928af9dcbd79431be53ae7247e1f00e700f68053b4c69c3d85d"),
    ("fsharp/src/fsharp.txt", "16c3d24b4b808a1ed6081daaeccf5977fd7c6400a321ea127184a9680e2864c6"),
    ("perl/src/perl.txt", "0f46dc92653a71e58b0689caf287fff107ccdf50d33eb81e23eb81668aadf820"),
    ("flutter/src/flutter.txt", "e076525d84870e1c44b4c59853bc4ec43bc5767670b89682ea7f8b6a1f2d43a7"),
    ("svelte/src/svelte.txt", "e12406023e82c1267abedf51725f768ca03365611c354203f33357b8cb255b3d"),
    ("gis/src/gis.txt", "fa72335e3cc8331b21b88956673a8e5c43d0a4c6c9d01bda769fb97b990e0bd3"),
]
# The lists whose mixed-case entries are names: `JavaScript`, `PostgreSQL`, `AstraZeneca`. The
# others' are API names (`assertTrue`, `ArgumentError`, `CNodeBase`): 12 000 of them, which in prose
# are identifiers, checked by their parts, and as corrections were junk ahead of the word meant
# (`clodebase` offered `CNodeBase` before `codebase`). Their lower-case and capitalized entries stay.
NAMES = {
    "software-terms/src/computing-acronyms.txt",
    "software-terms/src/cybersecurity-terms.txt",
    "software-terms/src/software-services.txt",
    "software-terms/src/software-terms.txt",
    "software-terms/src/software-tools.txt",
    "companies/src/companies.txt",
    "rust/src/crates.txt",
    "python/src/additional_words.txt",
    "node/src/additional_words.txt",
    "typescript/src/ecosystem-terms.txt",
    "aws/src/aws.txt",
    "docker/src/docker-words.txt",
    "fullstack/src/tools-and-services.txt",
    "google/src/google.txt",
    "google/src/legacy.txt",
    "google/src/misc.txt",
    "data-science/src/data-science-models.txt",
    "data-science/src/data-science-terms.txt",
    "data-science/src/data-science-tools.txt",
}
# The cspell-dicts directories of software terms, tools, companies and services, whose words come
# first among corrections as near: `Githb` is `GitHub` before it is the Python package `gitdb`. The
# others are programming languages' and formats' words, accepted alike and offered after.
TERMS = {"software-terms", "companies", "aws", "google", "docker", "git", "terraform", "k8s", "fullstack", "data-science"}
GB = "https://raw.githubusercontent.com/wooorm/dictionaries/8cfea406b505e4d7df52d5a19bce525df98c54ab/dictionaries/en-GB"
EN_GB = {
    "en_GB.aff": (f"{GB}/index.aff", "8ae1f19d4840d957728ad90555d5a8dff6cc5c046279c95ff0c00fc0a0136c7b"),
    "en_GB.dic": (f"{GB}/index.dic", "869fe17ba4ee4b5401c60a666ee2d6a3dcc237f460b7294435df1cc6a799aa57"),
    "LICENSE.dictionary": (f"{GB}/license", "81b644347b40804c25811267efbea024a30b2bb1642fefa9c9c50f7b0bbcb67f"),
}
EN_GB_TOML = """\
# en-GB: built by tools/packs.py from the sources in SOURCES.md. Do not edit by hand.
tag = "en-GB"
script = "Latn"
dictionary = { hunspell = "en_GB" }
# English's known typos and technical vocabulary are en-US's: shared, not copied.
typos = "../en-US/typos.tsv"
vocabulary = ["../en-US/vocabulary.txt", "../en-US/vocabulary.orthra.txt", "../en-US/vocabulary.code.txt"]
word-internal = ["'", "\u2019"]
normalize = { "\u2019" = "'" }
# A comment line that starts with one of these and a colon names people, not prose:
# `# Author: Jens Getreu <...>`.
name-labels = ["author", "authors", "contributor", "contributors", "credits", "maintainer", "maintainers", "original author"]
"""
EN_GB_SOURCES = """\
# en-GB — sources

Built by `python3 -I tools/packs.py en-GB`. Every file is fetched from a pinned source and checked
by SHA-256 before it is written.

| file                     | source                                                                                                                                                      | license                              |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| `en_GB.aff`, `en_GB.dic` | [wooorm/dictionaries](https://github.com/wooorm/dictionaries) `dictionary-en-gb` at `8cfea406`: the SCOWL en_GB-ise Hunspell dictionary, version 2020.12.07 | MIT AND BSD, in `LICENSE.dictionary` |
| known typos, vocabulary  | en-US's, by relative path (`../en-US/SOURCES.md`)                                                                                                           | as there                             |
"""
WORDS_CSV = (f"{TYPOS}/crates/typos-dict/assets/words.csv", "bcd34f5e29d82ae21f6b1d7c6268c69166c51949537a11354d70eb542f4bf39e")

PACK_TOML = """\
# en-US: built by tools/packs.py from the sources in SOURCES.md. Do not edit by hand.
tag = "en-US"
script = "Latn"
# `rare` holds what SCOWL size 70 adds to the size-60 dictionary: accepted only when the known typos
# do not name the word, since uncommon words are also common misspellings.
dictionary = { hunspell = "en_US", rare = "en_US-rare" }
typos = "typos.tsv"
# Words accepted whatever their case, beside the dictionary: technical vocabulary. Each is accepted
# alike; among corrections as near, an earlier file's come first: names and software terms, then
# the words of programming languages.
vocabulary = ["vocabulary.txt", "vocabulary.orthra.txt", "vocabulary.code.txt"]
# Characters that do not end a word, and the one form each is checked in.
word-internal = ["'", "’"]
normalize = { "’" = "'" }
# A comment line that starts with one of these and a colon names people, not prose:
# `# Author: Jens Getreu <...>`.
name-labels = ["author", "authors", "contributor", "contributors", "credits", "maintainer", "maintainers", "original author"]
"""

SOURCES = """\
# en-US — sources

Built by `python3 -I tools/packs.py en-US`. Every file is fetched from a pinned source and checked
by SHA-256 before it is written.

| file                                    | source                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | license                                                                          |
| --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `en_US.aff`, `en_US.dic`                | [wooorm/dictionaries](https://github.com/wooorm/dictionaries) `dictionary-en` at `8cfea406`: the SCOWL en_US Hunspell dictionary, version 2020.12.07, SCOWL size 60                                                                                                                                                                                                                                                                                                                                                                                             | MIT AND BSD, in `LICENSE.dictionary`                                             |
| `en_US-rare.dic`                        | [SCOWL](http://wordlist.aspell.net/dicts/) `hunspell-en_US-large-2020.12.07.zip` from SourceForge: the lines of `en_US-large.dic` (SCOWL size 70, accents kept) that `en_US.dic` does not have, read with `en_US.aff`                                                                                                                                                                                                                                                                                                                                           | SCOWL's, the same terms as `LICENSE.dictionary`; its README is in `LICENSE.rare` |
| `vocabulary.txt`, `vocabulary.code.txt` | [streetsidesoftware/cspell-dicts](https://github.com/streetsidesoftware/cspell-dicts) at `72b65202`: the 107 word lists for software that `tools/packs.py` pins by SHA-256, with cspell's markers (`~`, `*`, `+`) and comments removed, and without its forbidden words (`!`), known typos, British spellings, and API names (mixed-case entries outside the lists of names). `vocabulary.txt` is from the lists of software terms, tools, companies and services; `vocabulary.code.txt` is what the lists of 40 programming languages and their ecosystems add | MIT, in `LICENSE.vocabulary`                                                     |
| `vocabulary.orthra.txt`                 | orthra's own additions, kept by hand: general software English the zero corpus showed missing                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | orthra's                                                                         |
| `typos.tsv`                             | [crate-ci/typos](https://github.com/crate-ci/typos) `v1.52.0`, `crates/typos-dict/assets/words.csv`, re-written as tab-separated `typo<TAB>fix…`                                                                                                                                                                                                                                                                                                                                                                                                                | MIT OR Apache-2.0; the MIT text is in `LICENSE.typos`                            |
"""


def get(url, sha256):
    with urllib.request.urlopen(url) as response:
        data = response.read()
    got = hashlib.sha256(data).hexdigest()
    if got != sha256:
        raise SystemExit(f"{url}: digest {got}, expected {sha256}")
    return data


def en_us():
    out = os.path.join(ROOT, "packs", "en-US")
    os.makedirs(out, exist_ok=True)
    for name, (url, digest) in EN_US.items():
        with open(os.path.join(out, name), "wb") as f:
            f.write(get(url, digest))
    with open(os.path.join(out, "en_US.dic"), encoding="utf-8") as f:
        common = set(f.read().splitlines()[1:])
    large = zipfile.ZipFile(io.BytesIO(get(*SCOWL_LARGE)))
    rare = sorted(set(large.read("en_US-large.dic").decode("utf-8").splitlines()[1:]) - common)
    with open(os.path.join(out, "en_US-rare.dic"), "w", encoding="utf-8") as f:
        f.write(f"{len(rare)}\n")
        f.writelines(f"{line}\n" for line in rare)
    with open(os.path.join(out, "LICENSE.rare"), "wb") as f:
        f.write(large.read("README_en_US-large.txt"))
    rows = 0
    with open(os.path.join(out, "typos.tsv"), "w", encoding="utf-8") as f:
        f.write("# typo<TAB>fix[<TAB>fix…], from crate-ci/typos v1.52.0 (see SOURCES.md)\n")
        for line in get(*WORDS_CSV).decode("utf-8").splitlines():
            fields = [field.strip() for field in line.split(",")]
            if len(fields) < 2 or not all(fields):
                continue
            assert not any("\t" in field for field in fields), line
            f.write("\t".join(fields) + "\n")
            rows += 1
    with open(os.path.join(out, "LICENSE.vocabulary"), "wb") as f:
        f.write(get(*VOCABULARY_LICENSE))
    terms, code = vocabulary(out)
    for name, what, words in (
        ("vocabulary.txt", "software terms, tools and companies", terms),
        ("vocabulary.code.txt", "programming languages' words", code),
    ):
        with open(os.path.join(out, name), "w", encoding="utf-8") as f:
            f.write(f"# Technical vocabulary from cspell-dicts (see SOURCES.md): {what}.\n")
            f.write("# Sorted by the word in lower case, one spelling of each: orthra looks a word up by\n")
            f.write("# halving, and reads it where it is embedded.\n")
            f.writelines(f"{word}\n" for word in words)
    with open(os.path.join(out, "pack.toml"), "w", encoding="utf-8") as f:
        f.write(PACK_TOML)
    with open(os.path.join(out, "SOURCES.md"), "w", encoding="utf-8") as f:
        f.write(SOURCES)
    print(f"{out}: dictionary, {len(rare)} rare words, {rows} known typos, {len(terms)} + {len(code)} vocabulary words, pack.toml, licenses, SOURCES.md")


def vocabulary(out):
    """The technical vocabulary, sorted by lower case, one spelling of each: the words of the
    lists of TERMS, then what the others add.

    Kept to what can be a word, and cleared of three things that would hide a typo:
    - a known typo;
    - a British spelling: an en-US pack would otherwise accept `behaviour` and `licence`, which the
      lists picked up from source code;
    - a lower-case form of an all-capitals entry: `INOUT` is SQL, and `inout` is `input` mistyped.
      The lower-case or mixed-case spelling is kept when there is one, and an all-capitals entry
      matches only a word written in capitals (`orthra-packs`)."""
    # A word as segmentation makes one: letters, then letters, digits, apostrophes or hyphens. An
    # underscore or a dot splits a word, so `my_function` and `ASP.NET` could never be looked up.
    shape = re.compile(r"^[^\W\d_](?:[^\W_]|['’-])*[^\W_]$")
    with open(os.path.join(out, "typos.tsv"), encoding="utf-8") as f:
        typos = {line.split("\t", 1)[0].lower() for line in f if not line.startswith("#")}
    british = british_only()
    spellings = ({}, {})
    for path, digest in VOCABULARY:
        tier = spellings[0 if path.split("/", 1)[0] in TERMS else 1]
        for line in get(f"{CSPELL}/{path}", digest).decode("utf-8").splitlines():
            line = line.split("#", 1)[0].strip()
            for word in line.split():
                # `!word` is one cspell forbids: a misspelling, never a word.
                if word.startswith("!"):
                    continue
                word = word.lstrip("~").strip("*+:")
                lower = word.lower()
                if mixed_case(word) and path not in NAMES:
                    continue
                if shape.match(word) and lower not in typos and not is_british(lower, british):
                    tier.setdefault(lower, set()).add(word)
    def best(forms):
        # Lower or mixed case over all capitals, then the first in order.
        return sorted(forms, key=lambda w: (w.isupper() and len(w) > 1, w))[0]
    terms, code = spellings
    # A word's spelling is chosen from both: `foo` from a language's list keeps `FOO` from matching
    # only capitals.
    return (
        [best(terms[lower] | code.get(lower, set())) for lower in sorted(terms)],
        [best(code[lower]) for lower in sorted(code) if lower not in terms],
    )


def mixed_case(word):
    """Whether `word` has a capital after its first letter and a lower-case letter: `JavaScript`,
    `UUIDs`, `assertTrue`; not `kubectl`, `Kubernetes` or `CPU`."""
    return any(c.isupper() for c in word[1:]) and any(c.islower() for c in word)


def british_only():
    """The stems en-GB's dictionary has and en-US's does not: `behaviour`, `licence`, `analyser`."""
    def stems(path):
        with open(path, encoding="utf-8") as f:
            return {line.split("/", 1)[0].strip().lower() for line in f.read().splitlines()[1:]}
    return stems(os.path.join(ROOT, "packs", "en-GB", "en_GB.dic")) - stems(os.path.join(ROOT, "packs", "en-US", "en_US.dic"))


def is_british(lower, british):
    """Whether `lower` is a British stem, or one with an ending: `analysers`, `licences`, `colouring`."""
    candidates = {lower, lower[:-1], lower[:-2], lower[:-3], lower[:-3] + "e", lower[:-2] + "e"}
    return any(len(c) > 2 and c in british for c in candidates)


def en_gb():
    out = os.path.join(ROOT, "packs", "en-GB")
    os.makedirs(out, exist_ok=True)
    for name, (url, digest) in EN_GB.items():
        with open(os.path.join(out, name), "wb") as f:
            f.write(get(url, digest))
    with open(os.path.join(out, "pack.toml"), "w", encoding="utf-8") as f:
        f.write(EN_GB_TOML)
    with open(os.path.join(out, "SOURCES.md"), "w", encoding="utf-8") as f:
        f.write(EN_GB_SOURCES)
    print(f"{out}: dictionary, pack.toml, license, SOURCES.md; typos and vocabulary shared with en-US")


if __name__ == "__main__":
    commands = {"en-US": en_us, "en-GB": en_gb}
    if len(sys.argv) != 2 or sys.argv[1] not in commands:
        sys.exit(__doc__)
    commands[sys.argv[1]]()
