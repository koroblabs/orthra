//! Installing the context tier's model: its sources fetched by the revision `models/<name>/install.toml`
//! pins, each checked against its SHA-256, and the file a [`Runtime`] makes of them written where
//! models are read from: `<models>/<name>/<version>/model.gguf`.
//!
//! The downloads are held in memory, not kept: ELECTRA-small's f32 weights are 54 MB, and what is
//! installed is the 15 MB they quantize to.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use orthra_context::Runtime;
use orthra_packs::installed::{Install, Source};
use sha2::{Digest, Sha256};

use crate::{InstallError, hex};

#[cfg(test)]
mod tests;

/// The file an installed model is.
pub const MODEL_FILE: &str = "model.gguf";

/// Every model orthra can install, by name: one line here and a directory under `models/`.
const MODELS: &[(&str, &str)] = &[("small", include_str!("../../../models/small/install.toml"))];

/// A model's definition: where its sources are, and the weight the lexical order keeps beside it.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub install: Install,
    pub prior: f32,
}

impl Model {
    pub fn name(&self) -> &str {
        &self.install.tag
    }

    /// Where the model's file is, installed under `models`.
    pub fn file(&self, models: &Path) -> PathBuf {
        self.install.dir(models).join(MODEL_FILE)
    }

    pub fn is_installed(&self, models: &Path) -> bool {
        self.file(models).is_file()
    }
}

/// The names of the models orthra can install.
pub fn models() -> Vec<&'static str> {
    MODELS.iter().map(|(name, _)| *name).collect()
}

/// Model `name`'s definition.
pub fn model(name: &str) -> Result<Model, InstallError> {
    let Some((_, text)) = MODELS.iter().find(|(known, _)| *known == name) else {
        return Err(InstallError(format!(
            "{name} is not a model orthra knows: the models are {:?}",
            models()
        )));
    };
    let origin = format!("models/{name}/install.toml");
    let install = Install::read(name, &origin, text).map_err(|err| InstallError(err.0))?;
    let prior = text
        .parse::<toml::Table>()
        .ok()
        .and_then(|table| table.get("prior").and_then(toml::Value::as_float))
        .ok_or_else(|| InstallError(format!("{origin}: no `prior`")))?;
    Ok(Model {
        install,
        prior: prior as f32,
    })
}

/// Installs `model` under `models`, each source's bytes from `get` ([`crate::fetch`], or files the
/// person has) and the model file from `runtime`. Nothing is written unless every download hashes
/// to the SHA-256 its source pins and the runtime makes a file of them, which it reads back once:
/// a model that does not load is not left installed. Returns the model file.
pub fn install_model(
    model: &Model,
    mut get: impl FnMut(&Source) -> Result<Vec<u8>, InstallError>,
    runtime: &dyn Runtime,
    models: &Path,
) -> Result<PathBuf, InstallError> {
    let name = model.name();
    let failed = |why: String| InstallError(format!("{name}: {why}"));
    let mut files = HashMap::new();
    for source in &model.install.sources {
        let bytes = get(source).map_err(|err| failed(err.0))?;
        let digest = hex(&Sha256::digest(&bytes));
        if digest != source.sha256 {
            return Err(failed(format!(
                "{} has SHA-256 {digest}, and {name} {} pins {}",
                source.url, model.install.version, source.sha256
            )));
        }
        // A model's source is one file: moved in, not copied, since the weights are 54 MB.
        match source.files()[..] {
            [file] => {
                files.insert(file.to_owned(), bytes);
            }
            _ => return Err(failed(format!("{} is not one file", source.url))),
        }
    }
    let built = runtime
        .build(&files, model.prior)
        .map_err(|err| failed(format!("the model does not build: {err}")))?;
    drop(files);
    runtime
        .load(&built)
        .map_err(|err| failed(format!("the model it built does not load: {err}")))?;

    // Written beside where it goes, then moved there, so a reader never sees half a model.
    let target = model.install.dir(models);
    let partial = target.with_extension(format!("partial-{}", std::process::id()));
    let write = || -> Result<(), String> {
        fs::create_dir_all(&partial).map_err(|err| format!("{}: {err}", partial.display()))?;
        fs::write(partial.join(MODEL_FILE), &built)
            .map_err(|err| format!("{}: {err}", partial.display()))?;
        if target.exists() {
            fs::remove_dir_all(&target).map_err(|err| format!("{}: {err}", target.display()))?;
        }
        fs::rename(&partial, &target).map_err(|err| format!("{}: {err}", target.display()))
    };
    write().map_err(|why| {
        let _ = fs::remove_dir_all(&partial);
        failed(why)
    })?;
    Ok(model.file(models))
}
