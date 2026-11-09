//! Embeds every pack under `packs/` in the binary.
//!
//! This is what makes a built-in language a directory and not a diff (`docs/01-architecture.md`
//! §4): every file of every pack is included under `<tag>/<name>`, and a pack reads its own files,
//! or a neighbor's by `../`, through that one table. Licenses and source notes stay out of the
//! binary; they are for people, beside the data.
//!
//! A built-in pack whose `pack.toml` says `dictionary = { compiled = "en_US", rare = ... }` has its
//! Hunspell files compiled here, to the forms they make, as an install compiles an installed
//! pack's (`orthra-dict`'s `compiled`): the FSTs are embedded as `<name>.fst` and
//! `<name>.marks.fst`, and the `.dic` files are not, since nothing reads them then. English's
//! three dictionaries compile in about 0.3 s, to 0.9 MB.
//!
//! A built-in pack's `frequency` file is compiled too, to a table of each word's rank
//! (`orthra-dict`'s `ranks`), embedded under the file's own name in place of its text: once, when
//! two packs name it, as en-GB names en-US's.

use std::{
    collections::BTreeMap,
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

/// The `pack.toml` of the pack in `dir`, when it is built in; `None` for an installed pack, whose
/// files an install compiles.
fn built_in(dir: &Path) -> Option<toml::Table> {
    if dir.join("install.toml").is_file() {
        return None;
    }
    Some(
        fs::read_to_string(dir.join("pack.toml"))
            .expect("pack.toml is readable")
            .parse()
            .expect("pack.toml parses"),
    )
}

/// The dictionaries of the pack in `dir` that `pack.toml` says are compiled, by base name, the
/// main one first, with the `.aff` they are read with; nothing for an installed pack, which an
/// install compiles.
fn compiled(dir: &Path) -> (Vec<String>, String) {
    let Some(toml) = built_in(dir) else {
        return (Vec::new(), String::new());
    };
    let dictionary = |key: &str| {
        toml.get("dictionary")
            .and_then(|d| d.get(key))
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    match dictionary("compiled") {
        Some(base) => (
            std::iter::once(base.clone())
                .chain(dictionary("rare"))
                .collect(),
            base,
        ),
        None => (Vec::new(), String::new()),
    }
}

fn main() {
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").expect("cargo sets it"))
        .join("../../packs")
        .canonicalize()
        .expect("packs/ beside crates/");
    println!("cargo:rerun-if-changed={}", root.display());

    let mut dirs: Vec<_> = fs::read_dir(&root)
        .expect("packs/ is readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("pack.toml").is_file())
        .collect();
    dirs.sort();

    let out_dir = Path::new(&env::var("OUT_DIR").expect("cargo sets it")).to_owned();
    let mut tags = Vec::new();
    let mut files = String::new();
    let mut fsts = String::new();

    // Every built-in pack's frequency file, by where it is, under the name a pack reads it by:
    // `en-US/frequency.txt`, which en-GB names as `../en-US/frequency.txt`.
    let mut frequencies: BTreeMap<PathBuf, String> = BTreeMap::new();
    for dir in &dirs {
        let Some(name) =
            built_in(dir).and_then(|toml| toml.get("frequency")?.as_str().map(str::to_owned))
        else {
            continue;
        };
        let path = dir
            .join(&name)
            .canonicalize()
            .unwrap_or_else(|err| panic!("{}: {name}: {err}", dir.display()));
        let key = path
            .strip_prefix(&root)
            .ok()
            .and_then(Path::to_str)
            .unwrap_or_else(|| panic!("{}: {name} is not a pack's file", dir.display()))
            .replace('\\', "/");
        frequencies.insert(path, key);
    }
    for (path, key) in &frequencies {
        let text = fs::read_to_string(path).unwrap_or_else(|err| panic!("packs/{key}: {err}"));
        let ranks = orthra_dict::ranks::Ranks::compile(&text)
            .unwrap_or_else(|err| panic!("packs/{key} does not compile: {}", err.0));
        let compiled = out_dir.join(format!("{key}.ranks.fst"));
        fs::create_dir_all(compiled.parent().expect("a pack's directory"))
            .expect("OUT_DIR is writable");
        fs::write(&compiled, ranks).expect("OUT_DIR is writable");
        writeln!(
            fsts,
            "    ({key:?}, include_bytes!({:?})),",
            compiled.display().to_string()
        )
        .expect("writing to a String");
    }

    for dir in &dirs {
        let tag = dir
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a UTF-8 pack name")
            .to_owned();
        let (compiled, aff) = compiled(dir);
        for name in &compiled {
            let read = |file: String| {
                fs::read_to_string(dir.join(&file))
                    .unwrap_or_else(|err| panic!("packs/{tag}/{file}: {err}"))
            };
            let built = orthra_dict::compiled::build(
                &read(format!("{aff}.aff")),
                &read(format!("{name}.dic")),
            )
            .unwrap_or_else(|err| panic!("packs/{tag}/{name}.dic does not compile: {}", err.0));
            fs::create_dir_all(out_dir.join(&tag)).expect("OUT_DIR is writable");
            for (file, bytes) in [
                (format!("{name}.fst"), built.forms),
                (format!("{name}.marks.fst"), built.marks),
            ] {
                if bytes.is_empty() {
                    continue;
                }
                let path = out_dir.join(&tag).join(&file);
                fs::write(&path, bytes).expect("OUT_DIR is writable");
                writeln!(
                    fsts,
                    "    ({:?}, include_bytes!({:?})),",
                    format!("{tag}/{file}"),
                    path.display().to_string()
                )
                .expect("writing to a String");
            }
        }
        let mut paths: Vec<_> = fs::read_dir(dir)
            .expect("a readable pack")
            .flatten()
            .map(|e| e.path())
            .collect();
        paths.sort();
        for path in paths.iter().filter(|path| path.is_file()) {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("a UTF-8 file name");
            if name.starts_with("LICENSE") || name == "SOURCES.md" {
                continue;
            }
            if compiled
                .iter()
                .any(|base| name.strip_suffix(".dic") == Some(base.as_str()))
                || path
                    .canonicalize()
                    .is_ok_and(|path| frequencies.contains_key(&path))
            {
                continue;
            }
            writeln!(
                files,
                "    ({:?}, include_str!({:?})),",
                format!("{tag}/{name}"),
                path.display().to_string()
            )
            .expect("writing to a String");
        }
        tags.push(tag);
    }
    let out = format!(
        "/// Every built-in pack, by tag.\npub(crate) const PACKS: &[&str] = &{tags:?};\n\n\
         /// Every file of every built-in pack, as `<tag>/<name>`.\npub(crate) const FILES: &[(&str, &str)] = &[\n{files}];\n\n\
         /// Every compiled file of a built-in pack: a dictionary as `<tag>/<name>.fst`, a frequency table under its text's name.\npub(crate) const COMPILED: &[(&str, &[u8])] = &[\n{fsts}];\n"
    );
    let target = out_dir.join("builtin.rs");
    fs::write(target, out).expect("OUT_DIR is writable");
}
