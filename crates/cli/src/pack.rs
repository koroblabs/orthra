//! `orthra pack install <tag> [--from <file>]...` and `orthra pack list`: the packs orthra does not
//! carry, installed where every check and every server reads them from.

use std::{collections::HashMap, fs, path::PathBuf, process::ExitCode};

use orthra_install::Install;
use orthra_project::Config;

pub(crate) fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("list") => list(),
        Some("install") => install(&args[1..]),
        _ => usage(),
    }
}

fn list() -> ExitCode {
    let packs = Config::packs_dir();
    for tag in Install::tags() {
        let Some(Ok(install)) = Install::of(tag) else {
            continue;
        };
        let state = match &packs {
            Some(packs) if install.is_installed(packs) => {
                format!("installed in {}", install.dir(packs).display())
            }
            _ => "not installed".to_owned(),
        };
        println!("{tag:<8} {:<10} {state}", install.version);
        for license in install.licenses() {
            println!("         {license}");
        }
    }
    ExitCode::SUCCESS
}

fn install(args: &[String]) -> ExitCode {
    let mut tag = None;
    let mut from: Vec<PathBuf> = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--from" {
            from.extend(args.next().map(PathBuf::from));
        } else {
            tag = Some(arg.as_str());
        }
    }
    let Some(tag) = tag else {
        return usage();
    };
    let Some(packs) = Config::packs_dir() else {
        eprintln!("orthra pack install: no data directory: set XDG_DATA_HOME or HOME");
        return ExitCode::from(2);
    };
    // Files the person has, for a machine that cannot reach a source, each matched to its source by
    // its SHA-256; every other source is fetched. Either way it is checked before it is written.
    let mut given = HashMap::new();
    for file in &from {
        match fs::read(file) {
            Ok(bytes) => {
                given.insert(orthra_install::sha256(&bytes), bytes);
            }
            Err(err) => {
                eprintln!("orthra pack install: {}: {err}", file.display());
                return ExitCode::from(2);
            }
        }
    }
    match install_from(tag, &packs, given) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("orthra pack install: {err}");
            ExitCode::from(1)
        }
    }
}

/// Installs pack `tag` under `packs`, each source from `given` (by its SHA-256) or fetched, saying
/// what it does on stderr and what it installed on stdout.
fn install_from(
    tag: &str,
    packs: &std::path::Path,
    mut given: HashMap<String, Vec<u8>>,
) -> Result<(), String> {
    let definition = orthra_install::definition(tag).map_err(|err| err.0)?;
    let dir = orthra_install::install(
        tag,
        |source| match given.remove(&source.sha256) {
            Some(bytes) => Ok(bytes),
            None => {
                eprintln!("orthra: fetching {}", source.url);
                orthra_install::fetch(source)
            }
        },
        packs,
    )
    .map_err(|err| err.0)?;
    println!(
        "{tag} {} installed in {}",
        definition.version,
        dir.display()
    );
    for license in definition.licenses() {
        println!("  {license}");
    }
    Ok(())
}

/// Fetches pack `tag` from its sources and installs it under `packs`: what a check does for a
/// language the settings enable and nobody installed.
pub(crate) fn fetch_and_install(tag: &str, packs: &std::path::Path) -> Result<(), String> {
    eprintln!("orthra: installing {tag}");
    install_from(tag, packs, HashMap::new())
}

fn usage() -> ExitCode {
    eprintln!("usage: orthra pack list\n       orthra pack install <tag> [--from <file>]...");
    ExitCode::from(2)
}
