//! The `Dictionary` trait, and the only crate that knows spellbook and fst exist.
//!
//! Two implementations answer the same two questions, and a pack's `pack.toml` chooses one
//! (`docs/01-architecture.md` §7): [`hunspell::HunspellDict`] reads a Hunspell dictionary as it is,
//! and [`compiled::CompiledDict`] reads the forms [`compiled::build`] expanded it to. Nothing above
//! this crate can tell them apart.

use std::fmt;

pub mod compiled;
pub mod edits;
pub mod hunspell;
pub mod ranks;

pub trait Dictionary: Send + Sync + fmt::Debug {
    /// Whether the dictionary accepts `word` as it is written, case included: a Hunspell dictionary
    /// knows `Ukraine` and not `ukraine`.
    fn check(&self, word: &str) -> bool;

    /// Every correction it has for `word`, best first by its own measure, each with its distance
    /// from `word`. A pack orders them by how often each is written, where it knows, and keeps as
    /// many as it was asked for.
    ///
    /// Expensive: codebook's spellbook took about 23 ms a word on the typo sample, against
    /// microseconds to check (`docs/02-testing.md` level 9). Called for one word, on request, and
    /// never while producing diagnostics.
    fn suggest(&self, word: &str, out: &mut Vec<Candidate>);
}

/// A correction a dictionary offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub word: String,
    /// The edits between the word written and this one, case ignored, a swap of neighbors
    /// counted as one.
    pub edits: u32,
}

/// A dictionary that could not be read. Its message says which file and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictError(pub String);

impl fmt::Display for DictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DictError {}
