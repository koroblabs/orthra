//! The installed model: one GGUF file holding the weights, quantized to Q8_0, the dimensions from
//! Hugging Face's `config.json`, and the WordPiece vocabulary.
//!
//! Made once, when the model is installed, from the f32 safetensors its source publishes
//! ([`quantize`]), as a pack's dictionary is compiled when it is installed. Q8_0 costs no accuracy
//! and halves the CPU time against f32, and 14.8 MB on disk is a quarter of the download
//! (`docs/04-context.md` §4). Read whole into memory: no `unsafe` mmap.

use std::{collections::HashMap, io::Cursor};

use candle_core::{
    DType, Device, Tensor,
    quantized::{GgmlDType, QTensor, gguf_file},
};

use crate::ModelError;

/// What [`quantize`] writes, as `orthra.format`. A file of another format is refused, not
/// misread: changing it comes with a new version of every model definition.
pub const FORMAT: u32 = 1;

/// The dimensions of an ELECTRA generator.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Dimensions {
    pub(crate) hidden: usize,
    pub(crate) heads: usize,
    pub(crate) layers: usize,
    pub(crate) positions: usize,
    pub(crate) epsilon: f64,
}

/// A model file read back.
pub(crate) struct ModelFile {
    pub(crate) dimensions: Dimensions,
    /// How much the lexical order counts beside the model, as its definition tuned it.
    pub(crate) prior: f32,
    pub(crate) vocabulary: Vec<String>,
    pub(crate) tensors: HashMap<String, QTensor>,
}

impl std::fmt::Debug for ModelFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelFile")
            .field("dimensions", &self.dimensions)
            .field("vocabulary", &self.vocabulary.len())
            .field("tensors", &self.tensors.len())
            .finish()
    }
}

fn failed(what: &str) -> impl Fn(candle_core::Error) -> ModelError + '_ {
    move |err| ModelError(format!("{what}: {err}"))
}

/// The model file for an ELECTRA generator published as `safetensors` (f32), its `config.json`
/// and its `vocab.txt`, with `prior` the weight of the lexical order: every matrix whose rows fit
/// Q8_0's blocks in Q8_0, the word embeddings included (they are the tied output too), and the
/// rest -- norms, biases, the position table -- as it was.
pub fn quantize(
    safetensors: &[u8],
    config: &str,
    vocab: &str,
    prior: f32,
) -> Result<Vec<u8>, ModelError> {
    let config: serde_json::Value =
        serde_json::from_str(config).map_err(|err| ModelError(format!("config.json: {err}")))?;
    let number = |key: &str| {
        config[key]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| ModelError(format!("config.json has no `{key}`")))
    };
    if config["model_type"] != "electra" {
        return Err(ModelError(format!(
            "config.json says {}, and only an ELECTRA generator is read",
            config["model_type"]
        )));
    }
    // A tensor at a time, each quantized before the next is read: holding them all in f32 beside
    // the download put a quantization at 163 MB.
    let weights = candle_core::safetensors::SliceSafetensors::new(safetensors)
        .map_err(failed("the weights"))?;
    let mut names: Vec<String> = weights
        .tensors()
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    names.sort();
    let mut quantized: Vec<(String, QTensor)> = Vec::new();
    for name in names {
        let tensor = weights
            .load(&name, &Device::Cpu)
            .and_then(|tensor| tensor.to_dtype(DType::F32))
            .map_err(failed(&name))?;
        let matrix = tensor.rank() == 2
            && name.ends_with(".weight")
            && !name.contains("LayerNorm")
            && !name.contains("position_embeddings")
            && !name.contains("token_type_embeddings")
            && tensor
                .dim(1)
                .is_ok_and(|width| width % GgmlDType::Q8_0.block_size() == 0);
        let kind = if matrix {
            GgmlDType::Q8_0
        } else {
            GgmlDType::F32
        };
        quantized.push((
            name.clone(),
            QTensor::quantize(&tensor, kind).map_err(failed(&name))?,
        ));
    }

    let epsilon = config["layer_norm_eps"].as_f64().unwrap_or(1e-12) as f32;
    let words: Vec<gguf_file::Value> = vocab
        .lines()
        .map(|token| gguf_file::Value::String(token.to_owned()))
        .collect();
    let metadata = [
        ("orthra.format", gguf_file::Value::U32(FORMAT)),
        (
            "orthra.hidden",
            gguf_file::Value::U32(number("hidden_size")?),
        ),
        (
            "orthra.heads",
            gguf_file::Value::U32(number("num_attention_heads")?),
        ),
        (
            "orthra.layers",
            gguf_file::Value::U32(number("num_hidden_layers")?),
        ),
        (
            "orthra.positions",
            gguf_file::Value::U32(number("max_position_embeddings")?),
        ),
        ("orthra.epsilon", gguf_file::Value::F32(epsilon)),
        ("orthra.prior", gguf_file::Value::F32(prior)),
        ("orthra.vocabulary", gguf_file::Value::Array(words)),
    ];
    let metadata: Vec<(&str, &gguf_file::Value)> =
        metadata.iter().map(|(key, value)| (*key, value)).collect();
    let tensors: Vec<(&str, &QTensor)> = quantized
        .iter()
        .map(|(name, tensor)| (name.as_str(), tensor))
        .collect();
    let mut out = Cursor::new(Vec::new());
    gguf_file::write(&mut out, &metadata, &tensors).map_err(failed("the model file"))?;
    Ok(out.into_inner())
}

/// Reads a model file [`quantize`] wrote.
pub(crate) fn read(bytes: &[u8]) -> Result<ModelFile, ModelError> {
    let mut reader = Cursor::new(bytes);
    let content = gguf_file::Content::read(&mut reader).map_err(failed("the model file"))?;
    let value = |key: &str| {
        content
            .metadata
            .get(key)
            .ok_or_else(|| ModelError(format!("the model file has no `{key}`")))
    };
    let number = |key: &str| -> Result<usize, ModelError> {
        Ok(value(key)?.to_u32().map_err(failed(key))? as usize)
    };
    let format = value("orthra.format")?
        .to_u32()
        .map_err(failed("orthra.format"))?;
    if format != FORMAT {
        return Err(ModelError(format!(
            "a model file of format {format}, and this orthra reads {FORMAT}: install the model again"
        )));
    }
    let dimensions = Dimensions {
        hidden: number("orthra.hidden")?,
        heads: number("orthra.heads")?,
        layers: number("orthra.layers")?,
        positions: number("orthra.positions")?,
        epsilon: f64::from(
            value("orthra.epsilon")?
                .to_f32()
                .map_err(failed("orthra.epsilon"))?,
        ),
    };
    let vocabulary = value("orthra.vocabulary")?
        .to_vec()
        .map_err(failed("orthra.vocabulary"))?
        .iter()
        .map(|token| token.to_string().cloned().map_err(failed("a token")))
        .collect::<Result<Vec<String>, ModelError>>()?;
    let names: Vec<String> = content.tensor_infos.keys().cloned().collect();
    let mut tensors = HashMap::new();
    for name in names {
        let tensor = content
            .tensor(&mut reader, &name, &Device::Cpu)
            .map_err(failed(&name))?;
        tensors.insert(name, tensor);
    }
    let prior = value("orthra.prior")?
        .to_f32()
        .map_err(failed("orthra.prior"))?;
    Ok(ModelFile {
        dimensions,
        prior,
        vocabulary,
        tensors,
    })
}

/// A tensor of `file` as f32, dequantized if it was not stored so.
pub(crate) fn dense(file: &ModelFile, name: &str) -> Result<Tensor, ModelError> {
    file.tensors
        .get(name)
        .ok_or_else(|| ModelError(format!("the model file has no {name}")))?
        .dequantize(&Device::Cpu)
        .map_err(failed(name))
}
