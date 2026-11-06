//! Embeds every pack under `packs/` in the binary.
//!
//! This is what makes a built-in language a directory and not a diff (`docs/01-architecture.md`
//! §4): every file of every pack is included under `<tag>/<name>`, and a pack reads its own files,
//! or a neighbor's by `../`, through that one table. Licenses and source notes stay out of the
//! binary; they are for people, beside the data.

use std::{env, fmt::Write as _, fs, path::Path};

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

    let mut tags = Vec::new();
    let mut files = String::new();
    for dir in &dirs {
        let tag = dir
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a UTF-8 pack name")
            .to_owned();
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
         /// Every file of every built-in pack, as `<tag>/<name>`.\npub(crate) const FILES: &[(&str, &str)] = &[\n{files}];\n"
    );
    let target = Path::new(&env::var("OUT_DIR").expect("cargo sets it")).join("builtin.rs");
    fs::write(target, out).expect("OUT_DIR is writable");
}
