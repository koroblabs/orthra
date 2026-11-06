# uk-UA — sources

Not built in. `orthra pack install uk-UA` fetches each file from its source and checks it by SHA-256
before it is written to `$XDG_DATA_HOME/orthra/packs/uk-UA/<version>/`. orthra carries only the
pack's definition: `pack.toml` and `install.toml`.

| file                     | source                                                                                                                                                                                | license                                                                   |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `uk_UA.aff`, `uk_UA.dic` | [brown-uk/dict_uk](https://github.com/brown-uk/dict_uk) release `v6.8.6`, `hunspell-uk_UA_6.8.6.zip`                                                                                  | GPL-3.0, the repository's; the archive carries no license file of its own |
| `uk_50k.txt`             | [hermitdave/FrequencyWords](https://github.com/hermitdave/FrequencyWords) at `525f9b56`, `content/2018/uk/uk_50k.txt`: 50 000 word forms from OpenSubtitles 2018, most frequent first | CC-BY-SA-4.0, the README's terms for the lists (the code is MIT)          |
