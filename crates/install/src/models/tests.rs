use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use orthra_context::{FakeScorer, Runtime, ScoreError, Scorer};
use sha2::{Digest, Sha256};

use super::{MODEL_FILE, Model, install_model, model};
use crate::{InstallError, hex};

/// A directory of the test's own to install into: never the person's.
fn models_dir() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "orthra-models-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

/// A runtime that builds a file naming what it was given, and loads only what it built.
#[derive(Debug)]
struct Pretend;

impl Runtime for Pretend {
    fn build(&self, files: &HashMap<String, Vec<u8>>, prior: f32) -> Result<Vec<u8>, ScoreError> {
        let mut names: Vec<&String> = files.keys().collect();
        names.sort();
        Ok(format!("built from {names:?} with prior {prior}").into_bytes())
    }

    fn load(&self, model: &[u8]) -> Result<Arc<dyn Scorer>, ScoreError> {
        if model.starts_with(b"built from") {
            Ok(Arc::new(FakeScorer))
        } else {
            Err(ScoreError("not a model".to_owned()))
        }
    }
}

/// A runtime whose files never load.
#[derive(Debug)]
struct Unloadable;

impl Runtime for Unloadable {
    fn build(&self, _: &HashMap<String, Vec<u8>>, _: f32) -> Result<Vec<u8>, ScoreError> {
        Ok(b"garbage".to_vec())
    }

    fn load(&self, _: &[u8]) -> Result<Arc<dyn Scorer>, ScoreError> {
        Err(ScoreError("not a model".to_owned()))
    }
}

/// The small model's definition, each source pinned to its file's name as the test's bytes.
fn pinned() -> Model {
    let mut small = model("small").unwrap();
    for source in &mut small.install.sources {
        source.sha256 = hex(&Sha256::digest(source.files()[0].as_bytes()));
    }
    small
}

fn by_name(source: &orthra_packs::installed::Source) -> Result<Vec<u8>, InstallError> {
    Ok(source.files()[0].as_bytes().to_vec())
}

#[test]
fn the_small_model_is_electra_from_pinned_revisions() {
    let small = model("small").unwrap();
    let sources: Vec<String> = small
        .install
        .sources
        .iter()
        .map(|source| format!("{} {}\n  {}", source.files()[0], source.sha256, source.url))
        .collect();
    expect_test::expect![[r#"
        small electra-small-generator.51a5c11.1 prior 0.25
        model.safetensors c18d39e9f75cd9be4528ec96b187e72da47321845ea96458a99831c556cfd9f4
          https://huggingface.co/google/electra-small-generator/resolve/51a5c116d83a1cd5b3503636281e432f0ec0c78a/model.safetensors
        config.json 7c52c07d59bf9a757f75959b58389ebbb833ee46f74cbfc36fccb72563833f33
          https://huggingface.co/google/electra-small-generator/resolve/51a5c116d83a1cd5b3503636281e432f0ec0c78a/config.json
        vocab.txt 07eced375cec144d27c900241f3e339478dec958f92fddbc551f295c992038a3
          https://huggingface.co/google/electra-small-generator/resolve/51a5c116d83a1cd5b3503636281e432f0ec0c78a/vocab.txt"#]].assert_eq(&format!(
        "{} {} prior {}\n{}",
        small.name(),
        small.install.version,
        small.prior,
        sources.join("\n")
    ));
    assert!(
        model("huge")
            .unwrap_err()
            .0
            .contains("not a model orthra knows")
    );
}

#[test]
fn every_download_that_hashes_as_pinned_is_built_into_one_file() {
    let models = models_dir();
    let small = pinned();
    let file = install_model(&small, by_name, &Pretend, &models).unwrap();
    assert_eq!(file, small.file(&models));
    assert!(file.ends_with(MODEL_FILE));
    assert!(small.is_installed(&models));
    expect_test::expect![[
        r#"built from ["config.json", "model.safetensors", "vocab.txt"] with prior 0.25"#
    ]]
    .assert_eq(&fs::read_to_string(&file).unwrap());
    let left: Vec<String> = fs::read_dir(models.join("small"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        left,
        std::slice::from_ref(&small.install.version),
        "no partial directory"
    );
    fs::remove_dir_all(&models).unwrap();
}

#[test]
fn a_download_that_does_not_hash_as_pinned_or_a_model_that_does_not_load_writes_nothing() {
    let models = models_dir();
    let small = pinned();
    let tampered = install_model(
        &small,
        |_| Ok(b"something else".to_vec()),
        &Pretend,
        &models,
    )
    .unwrap_err();
    let unloadable = install_model(&small, by_name, &Unloadable, &models).unwrap_err();
    let short = |err: InstallError| err.0.split(" has SHA-256").next().unwrap().to_owned();
    expect_test::expect![[r#"
        small: https://huggingface.co/google/electra-small-generator/resolve/51a5c116d83a1cd5b3503636281e432f0ec0c78a/model.safetensors
        small: the model it built does not load: not a model"#]].assert_eq(&format!("{}\n{}", short(tampered), unloadable.0));
    assert!(!small.is_installed(&models));
    assert!(
        !models.exists()
            || fs::read_dir(models.join("small")).map_or(true, |mut dir| dir.next().is_none())
    );
}
