//! ELECTRA's generator, a small masked language model, on candle's CPU: an encoder of BERT's
//! shape whose matrices are Q8_0, and a head that turns a position's hidden state into a
//! log-probability for every word of the vocabulary, the word embeddings serving as its output
//! weights.
//!
//! The probe it is taken from scored bit-identically to candle-transformers in f32
//! (`docs/04-context.md` §4). Only the rows a caller asks for go through the head: the probe ran
//! every position through it, and its `[rows, 30 522]` logits were most of its 94 MB peak.

use std::sync::Arc;

use candle_core::{D, DType, Device, Module, Tensor, quantized::QMatMul};
use candle_nn::LayerNorm;

use crate::{
    ModelError,
    model_file::{Dimensions, ModelFile, dense},
};

fn failed(err: candle_core::Error) -> ModelError {
    ModelError(format!("the model: {err}"))
}

/// A linear layer: Q8_0, or f32 where the matrix did not fit Q8_0's blocks.
struct Linear {
    weight: QMatMul,
    bias: Tensor,
}

impl Linear {
    fn read(file: &mut ModelFile, prefix: &str) -> Result<Linear, ModelError> {
        let name = format!("{prefix}.weight");
        let weight = file
            .tensors
            .remove(&name)
            .ok_or_else(|| ModelError(format!("the model file has no {name}")))?;
        Ok(Linear {
            weight: QMatMul::from_arc(Arc::new(weight)).map_err(failed)?,
            bias: dense(file, &format!("{prefix}.bias"))?,
        })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor, ModelError> {
        self.weight
            .forward(x)
            .and_then(|y| y.broadcast_add(&self.bias))
            .map_err(failed)
    }
}

fn norm(file: &ModelFile, prefix: &str, epsilon: f64) -> Result<LayerNorm, ModelError> {
    Ok(LayerNorm::new(
        dense(file, &format!("{prefix}.weight"))?,
        dense(file, &format!("{prefix}.bias"))?,
        epsilon,
    ))
}

struct Layer {
    query: Linear,
    key: Linear,
    value: Linear,
    attended: Linear,
    attended_norm: LayerNorm,
    widened: Linear,
    narrowed: Linear,
    output_norm: LayerNorm,
}

pub(crate) struct Electra {
    /// The word embeddings, looked up for the input and multiplied for the output.
    words: QMatMul,
    positions: Tensor,
    /// The token type embedding of the first segment: every input is one segment.
    segment: Tensor,
    embedded_norm: LayerNorm,
    /// From the embedding width to the hidden width, when they differ (128 to 256 in ELECTRA-small).
    project: Option<Linear>,
    layers: Vec<Layer>,
    head: Linear,
    head_norm: LayerNorm,
    output_bias: Tensor,
    heads: usize,
    pub(crate) longest: usize,
}

impl std::fmt::Debug for Electra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Electra")
            .field("layers", &self.layers.len())
            .field("heads", &self.heads)
            .finish_non_exhaustive()
    }
}

impl Electra {
    pub(crate) fn new(mut file: ModelFile) -> Result<Electra, ModelError> {
        let Dimensions {
            heads,
            layers,
            positions: longest,
            epsilon,
            ..
        } = file.dimensions;
        let words = file
            .tensors
            .remove("electra.embeddings.word_embeddings.weight")
            .ok_or_else(|| ModelError("the model file has no word embeddings".to_owned()))?;
        let project = if file
            .tensors
            .contains_key("electra.embeddings_project.weight")
        {
            Some(Linear::read(&mut file, "electra.embeddings_project")?)
        } else {
            None
        };
        let mut encoder = Vec::with_capacity(layers);
        for at in 0..layers {
            let layer = format!("electra.encoder.layer.{at}");
            encoder.push(Layer {
                query: Linear::read(&mut file, &format!("{layer}.attention.self.query"))?,
                key: Linear::read(&mut file, &format!("{layer}.attention.self.key"))?,
                value: Linear::read(&mut file, &format!("{layer}.attention.self.value"))?,
                attended: Linear::read(&mut file, &format!("{layer}.attention.output.dense"))?,
                attended_norm: norm(
                    &file,
                    &format!("{layer}.attention.output.LayerNorm"),
                    epsilon,
                )?,
                widened: Linear::read(&mut file, &format!("{layer}.intermediate.dense"))?,
                narrowed: Linear::read(&mut file, &format!("{layer}.output.dense"))?,
                output_norm: norm(&file, &format!("{layer}.output.LayerNorm"), epsilon)?,
            });
        }
        Ok(Electra {
            words: QMatMul::from_arc(Arc::new(words)).map_err(failed)?,
            positions: dense(&file, "electra.embeddings.position_embeddings.weight")?,
            segment: dense(&file, "electra.embeddings.token_type_embeddings.weight")?
                .get(0)
                .map_err(failed)?,
            embedded_norm: norm(&file, "electra.embeddings.LayerNorm", epsilon)?,
            project,
            head: Linear::read(&mut file, "generator_predictions.dense")?,
            head_norm: norm(&file, "generator_predictions.LayerNorm", epsilon)?,
            output_bias: dense(&file, "generator_lm_head.bias")?,
            layers: encoder,
            heads,
            longest,
        })
    }

    /// The hidden states of a batch of `rows` sequences of `width` ids each, padded where `real`
    /// is false: `[rows, width, hidden]`.
    pub(crate) fn encode(
        &self,
        ids: Vec<u32>,
        real: &[bool],
        rows: usize,
        width: usize,
    ) -> Result<Tensor, ModelError> {
        let device = Device::Cpu;
        let ids = Tensor::from_vec(ids, (rows, width), &device).map_err(failed)?;
        let mut x = self
            .words
            .embedding(&ids)
            .and_then(|x| x.broadcast_add(&self.positions.narrow(0, 0, width)?))
            .and_then(|x| x.broadcast_add(&self.segment))
            .and_then(|x| self.embedded_norm.forward(&x))
            .map_err(failed)?;
        if let Some(project) = &self.project {
            x = project.forward(&x)?;
        }
        // Padding is attended to by nothing: a large negative score before the softmax.
        let ignore: Vec<f32> = real
            .iter()
            .map(|real| if *real { 0.0 } else { -1e9 })
            .collect();
        let ignore = Tensor::from_vec(ignore, (rows, 1, 1, width), &device).map_err(failed)?;
        let hidden = x.dim(D::Minus1).map_err(failed)?;
        let per_head = hidden / self.heads;
        let scale = 1.0 / (per_head as f64).sqrt();
        for layer in &self.layers {
            let split = |t: Tensor| -> Result<Tensor, ModelError> {
                t.reshape((rows, width, self.heads, per_head))
                    .and_then(|t| t.transpose(1, 2))
                    .and_then(|t| t.contiguous())
                    .map_err(failed)
            };
            let query = split(layer.query.forward(&x)?)?;
            let key = split(layer.key.forward(&x)?)?;
            let value = split(layer.value.forward(&x)?)?;
            let context = key
                .transpose(2, 3)
                .and_then(|k| k.contiguous())
                .and_then(|k| query.matmul(&k))
                .and_then(|s| s * scale)
                .and_then(|s| s.broadcast_add(&ignore))
                .and_then(|s| candle_nn::ops::softmax_last_dim(&s))
                .and_then(|a| a.matmul(&value))
                .and_then(|c| c.transpose(1, 2))
                .and_then(|c| c.contiguous())
                .and_then(|c| c.reshape((rows, width, hidden)))
                .map_err(failed)?;
            let attended = layer.attended.forward(&context)?;
            x = layer
                .attended_norm
                .forward(&(attended + &x).map_err(failed)?)
                .map_err(failed)?;
            let widened = layer.widened.forward(&x)?.gelu_erf().map_err(failed)?;
            let narrowed = layer.narrowed.forward(&widened)?;
            x = layer
                .output_norm
                .forward(&(narrowed + &x).map_err(failed)?)
                .map_err(failed)?;
        }
        Ok(x)
    }

    /// The log-probability of `wanted[i]` at hidden state `states[i]`, for each row of `states`
    /// (`[n, hidden]`): only these rows go through the head.
    pub(crate) fn log_probs(
        &self,
        states: &Tensor,
        wanted: &[u32],
    ) -> Result<Vec<f32>, ModelError> {
        let wanted =
            Tensor::from_vec(wanted.to_vec(), (wanted.len(), 1), &Device::Cpu).map_err(failed)?;
        let x = self.head.forward(states)?.gelu_erf().map_err(failed)?;
        self.head_norm
            .forward(&x)
            .and_then(|x| self.words.forward(&x))
            .and_then(|logits| logits.broadcast_add(&self.output_bias))
            .and_then(|logits| candle_nn::ops::log_softmax(&logits, D::Minus1))
            .and_then(|all| all.gather(&wanted, 1))
            .and_then(|picked| picked.flatten_all())
            .and_then(|picked| picked.to_dtype(DType::F32))
            .and_then(|picked| picked.to_vec1::<f32>())
            .map_err(failed)
    }

    /// The rows `at` (flat indices into `[rows * width]`) of `hidden`, `[rows, width, hidden]`.
    pub(crate) fn rows(hidden: &Tensor, at: &[u32]) -> Result<Tensor, ModelError> {
        let (rows, width, size) = hidden.dims3().map_err(failed)?;
        let at = Tensor::from_vec(at.to_vec(), at.len(), &Device::Cpu).map_err(failed)?;
        hidden
            .reshape((rows * width, size))
            .and_then(|flat| flat.index_select(&at, 0))
            .map_err(failed)
    }
}
