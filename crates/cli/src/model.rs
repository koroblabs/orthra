//! `orthra model install <name> [--from <file>]...` and `orthra model list`: the context tier's
//! models, installed where the server loads them from when the editor's settings turn the tier on.
//!
//! A model is built here from the sources its definition pins, by the in-process runtime the
//! binary was built with (feature `ml-local`); a binary without one says so.

use std::{collections::HashMap, fs, path::PathBuf, process::ExitCode, sync::Arc};

use orthra_engine::context::Runtime;
use orthra_install::models::{install_model, model, models};
use orthra_project::Config;

pub(crate) fn run(args: &[String], runtime: Option<Arc<dyn Runtime>>) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("list") => list(),
        Some("install") => install(&args[1..], runtime),
        _ => usage(),
    }
}

fn list() -> ExitCode {
    let dir = Config::models_dir();
    for name in models() {
        let Ok(model) = model(name) else {
            continue;
        };
        let state = match &dir {
            Some(dir) if model.is_installed(dir) => {
                format!("installed in {}", model.file(dir).display())
            }
            _ => "not installed".to_owned(),
        };
        println!("{name:<8} {:<36} {state}", model.install.version);
        for license in model.install.licenses() {
            println!("         {license}");
        }
    }
    ExitCode::SUCCESS
}

fn install(args: &[String], runtime: Option<Arc<dyn Runtime>>) -> ExitCode {
    let Some(runtime) = runtime else {
        eprintln!(
            "orthra model install: this orthra was built without a model runtime (feature `ml-local`)"
        );
        return ExitCode::from(2);
    };
    let mut name = None;
    let mut from: Vec<PathBuf> = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--from" {
            from.extend(args.next().map(PathBuf::from));
        } else {
            name = Some(arg.as_str());
        }
    }
    let Some(name) = name else {
        return usage();
    };
    let Some(dir) = Config::models_dir() else {
        eprintln!("orthra model install: no data directory: set XDG_DATA_HOME or HOME");
        return ExitCode::from(2);
    };
    // Files the person has, for a machine that cannot reach a source, each matched to its source by
    // its SHA-256; every other source is fetched. Either way it is checked before it is used.
    let mut given = HashMap::new();
    for file in &from {
        match fs::read(file) {
            Ok(bytes) => {
                given.insert(orthra_install::sha256(&bytes), bytes);
            }
            Err(err) => {
                eprintln!("orthra model install: {}: {err}", file.display());
                return ExitCode::from(2);
            }
        }
    }
    let installed = model(name).and_then(|model| {
        let file = install_model(
            &model,
            |source| match given.remove(&source.sha256) {
                Some(bytes) => Ok(bytes),
                None => {
                    eprintln!("orthra: fetching {}", source.url);
                    orthra_install::fetch(source)
                }
            },
            &*runtime,
            &dir,
        )?;
        Ok((model, file))
    });
    match installed {
        Ok((model, file)) => {
            println!(
                "model {name} {} installed in {}",
                model.install.version,
                file.display()
            );
            for license in model.install.licenses() {
                println!("  {license}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("orthra model install: {err}");
            ExitCode::from(1)
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: orthra model list\n       orthra model install <name> [--from <file>]...");
    ExitCode::from(2)
}
