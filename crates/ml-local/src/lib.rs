//! The context tier's model, run in process: ELECTRA-small's generator, Q8_0, on candle's CPU.
//!
//! The only crate that knows candle exists (`CLAUDE.md` §1.1), a leaf behind the binary's
//! `ml-local` feature. Measured on the GitHub Typo Corpus, it puts the right correction first 71.1 %
//! of the time against 68.6 % for the lexical order alone, in 6 ms at the median and 17 ms at the
//! 95th percentile on 4 threads, from a 14.8 MB file that is 16 MB of memory once read
//! (`docs/04-context.md` §4).
//!
//! It reorders the lexical tier's corrections and does nothing else: [`ElectraScorer`] is an
//! `orthra_context::Scorer`.

use std::{collections::HashMap, fmt, sync::Arc};

use orthra_context::{Runtime, ScoreError, Scorer};

mod electra;
mod model_file;
mod scorer;
mod wordpiece;

#[cfg(test)]
mod tests;

pub use model_file::{FORMAT, quantize};
pub use scorer::ElectraScorer;

/// The in-process runtime: ELECTRA generators, quantized to Q8_0, on candle's CPU.
#[derive(Debug, Default, Clone, Copy)]
pub struct Local;

impl Runtime for Local {
    fn build(&self, files: &HashMap<String, Vec<u8>>, prior: f32) -> Result<Vec<u8>, ScoreError> {
        let file = |name: &str| {
            files
                .get(name)
                .ok_or_else(|| ScoreError(format!("the model's sources give no {name}")))
        };
        let text = |name: &str| {
            std::str::from_utf8(file(name)?).map_err(|err| ScoreError(format!("{name}: {err}")))
        };
        quantize(
            file("model.safetensors")?,
            text("config.json")?,
            text("vocab.txt")?,
            prior,
        )
        .map_err(|err| ScoreError(err.0))
    }

    fn load(&self, model: &[u8]) -> Result<Arc<dyn Scorer>, ScoreError> {
        ElectraScorer::read(model)
            .map(|scorer| Arc::new(scorer) as Arc<dyn Scorer>)
            .map_err(|err| ScoreError(err.0))
    }
}

/// A model that could not be made or read, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError(pub String);

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ModelError {}
