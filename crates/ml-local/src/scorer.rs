//! ELECTRA's generator as the context tier's [`Scorer`]: each candidate written in its slot with
//! its tokens masked, scored by the log-probabilities the model gives them there, and the lexical
//! order weighed in.
//!
//! One pass for all of a word's candidates: masking a candidate's tokens together scored as well
//! as masking them one at a time (pseudo-log-likelihood) for a fraction of the passes
//! (`docs/04-context.md` §4).

use orthra_context::{ScoreError, Scorer};
use orthra_text::TextRange;

use crate::{ModelError, electra::Electra, model_file, wordpiece::WordPiece};

/// The context tokens kept on each side of a slot: a 32-token window scored as well as 128.
const SIDE: usize = 48;

/// The most tokens a candidate is given: a word of more pieces is cut, as no correction is.
const LONGEST_CANDIDATE: usize = 16;

/// A candidate with no tokens at all is scored as unlikely as can be, and stays finite.
const NOTHING: f32 = -1.0e9;

/// The threads a scorer computes on, so a model does not take every core. Its matrices are small
/// enough that more barely help: in the server, an asked code action took 12.2 / 16.6 ms (p50 /
/// p95) on one thread, 12.3 / 13.3 on two and 13.0 / 13.7 on four, with the same footprint
/// (2026-11-06).
const THREADS: usize = 2;

pub struct ElectraScorer {
    model: Electra,
    words: WordPiece,
    /// How much the lexical order counts: each candidate gets `prior · −ln(position + 1)`. 0.25
    /// for ELECTRA-small in Q8_0, tuned on half the typo sample and measured on the other half;
    /// written into the model file when it is installed.
    prior: f32,
    /// candle computes on rayon's pool of the moment: this one, held as long as the model is.
    threads: rayon::ThreadPool,
    /// What the weights and the vocabulary hold, counted when they are read.
    bytes: usize,
}

impl std::fmt::Debug for ElectraScorer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElectraScorer")
            .field("model", &self.model)
            .field("prior", &self.prior)
            .finish_non_exhaustive()
    }
}

impl ElectraScorer {
    /// The scorer for a model file [`crate::quantize`] wrote.
    pub fn read(model: &[u8]) -> Result<ElectraScorer, ModelError> {
        let mut file = model_file::read(model)?;
        // The weights as stored -- Q8_0, and the small f32 ones -- and each token with what a map
        // entry costs beside it.
        let bytes = file
            .tensors
            .values()
            .map(candle_core::quantized::QTensor::storage_size_in_bytes)
            .sum::<usize>()
            + file
                .vocabulary
                .iter()
                .map(|token| token.len() + 48)
                .sum::<usize>();
        let words = WordPiece::new(std::mem::take(&mut file.vocabulary)).map_err(ModelError)?;
        let threads = rayon::ThreadPoolBuilder::new()
            .num_threads(THREADS)
            .thread_name(|at| format!("orthra-model-{at}"))
            .build()
            .map_err(|err| ModelError(format!("the model's threads: {err}")))?;
        Ok(ElectraScorer {
            prior: file.prior,
            model: Electra::new(file)?,
            words,
            threads,
            bytes,
        })
    }

    /// What the model alone says of each candidate in `slot` of `sentence`.
    pub(crate) fn model_scores(
        &self,
        sentence: &str,
        slot: TextRange,
        candidates: &[&str],
    ) -> Result<Vec<f32>, ModelError> {
        let (start, end) = (usize::from(slot.start()), usize::from(slot.end()));
        let (mut before, mut after) = (Vec::new(), Vec::new());
        self.words.ids(&sentence[..start], &mut before);
        self.words.ids(&sentence[end..], &mut after);
        let before = &before[before.len().saturating_sub(SIDE)..];
        let after = &after[..after.len().min(SIDE)];

        let mut pieces: Vec<Vec<u32>> = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let mut ids = Vec::new();
            self.words.ids(candidate, &mut ids);
            ids.truncate(LONGEST_CANDIDATE);
            pieces.push(ids);
        }
        let width = 2 + before.len() + after.len() + pieces.iter().map(Vec::len).max().unwrap_or(0);
        if width > self.model.longest {
            return Err(ModelError(format!(
                "{width} tokens, and the model reads {}",
                self.model.longest
            )));
        }

        let rows = candidates.len();
        let mut ids = Vec::with_capacity(rows * width);
        let mut real = Vec::with_capacity(rows * width);
        let mut masked = Vec::new();
        let mut wanted = Vec::new();
        for (row, candidate) in pieces.iter().enumerate() {
            let at = ids.len();
            ids.push(self.words.start);
            ids.extend_from_slice(before);
            for (offset, id) in candidate.iter().enumerate() {
                masked.push(u32::try_from(at + 1 + before.len() + offset).unwrap_or(u32::MAX));
                wanted.push(*id);
                ids.push(self.words.mask);
            }
            ids.extend_from_slice(after);
            ids.push(self.words.end);
            let used = ids.len() - at;
            real.extend(std::iter::repeat_n(true, used));
            ids.extend(std::iter::repeat_n(self.words.pad, width - used));
            real.extend(std::iter::repeat_n(false, width - used));
            debug_assert_eq!(ids.len(), (row + 1) * width);
        }

        let hidden = self.model.encode(ids, &real, rows, width)?;
        let states = Electra::rows(&hidden, &masked)?;
        let log_probs = self.model.log_probs(&states, &wanted)?;
        let mut scores = Vec::with_capacity(rows);
        let mut next = 0;
        for candidate in &pieces {
            scores.push(if candidate.is_empty() {
                NOTHING
            } else {
                log_probs[next..next + candidate.len()].iter().sum()
            });
            next += candidate.len();
        }
        Ok(scores)
    }
}

impl Scorer for ElectraScorer {
    fn score(
        &self,
        sentence: &str,
        slot: TextRange,
        candidates: &[&str],
    ) -> Result<Vec<f32>, ScoreError> {
        let scores = self
            .threads
            .install(|| self.model_scores(sentence, slot, candidates))
            .map_err(|err| ScoreError(err.0))?;
        Ok(scores
            .into_iter()
            .enumerate()
            .map(|(position, score)| score - self.prior * ((position + 1) as f32).ln())
            .collect())
    }

    fn bytes(&self) -> usize {
        self.bytes
    }
}
