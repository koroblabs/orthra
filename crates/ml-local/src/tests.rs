//! A tiny ELECTRA generator of made-up weights, in every shape the real one has, so the pipeline
//! from f32 safetensors to scores is tested with no download. What it scores means nothing; what
//! is tested is that scoring is deterministic, that a candidate scores the same alone as in a
//! batch padded beside longer ones, and that the prior is the lexical order weighed in.

use std::collections::HashMap;

use candle_core::{Device, Tensor};
use orthra_context::Scorer;
use orthra_text::{TextRange, TextSize};

use crate::{ElectraScorer, quantize};

const EMBEDDING: usize = 32;
const HIDDEN: usize = 64;
const LAYERS: usize = 2;
const WIDE: usize = 64;
const POSITIONS: usize = 160;

pub(crate) const VOCABULARY: &[&str] = &[
    "[PAD]", "[UNK]", "[CLS]", "[SEP]", "[MASK]", "the", "a", "cat", "sat", "on", "mat", "we",
    "re", "##ceive", "##lieve", "##s", "it", "fix", "fox", "box", "send", ".", ",", "'", "cafe",
    "##ing", "will", "please", "and", "to", "of", "in",
];

/// Made-up weights: the same every run, varied enough that no two rows are alike.
fn weights(shape: &[usize], seed: u64) -> Tensor {
    let count: usize = shape.iter().product();
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    let values: Vec<f32> = (0..count)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state % 2001) as f32 / 1000.0 - 1.0) * 0.2
        })
        .collect();
    Tensor::from_vec(values, shape, &Device::Cpu).unwrap()
}

fn ones(width: usize) -> Tensor {
    Tensor::ones(width, candle_core::DType::F32, &Device::Cpu).unwrap()
}

/// The tiny model's weights as safetensors, made once for every test of the process.
fn tiny_safetensors() -> Vec<u8> {
    static MADE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    MADE.get_or_init(make_safetensors).clone()
}

fn make_safetensors() -> Vec<u8> {
    let vocabulary = VOCABULARY.len();
    let mut tensors: HashMap<String, Tensor> = HashMap::new();
    let mut seed = 0;
    let mut put = |name: &str, tensor: Tensor| {
        tensors.insert(name.to_owned(), tensor);
    };
    let mut next = || {
        seed += 1;
        seed
    };
    put(
        "electra.embeddings.word_embeddings.weight",
        weights(&[vocabulary, EMBEDDING], next()),
    );
    put(
        "electra.embeddings.position_embeddings.weight",
        weights(&[POSITIONS, EMBEDDING], next()),
    );
    put(
        "electra.embeddings.token_type_embeddings.weight",
        weights(&[2, EMBEDDING], next()),
    );
    put("electra.embeddings.LayerNorm.weight", ones(EMBEDDING));
    put(
        "electra.embeddings.LayerNorm.bias",
        weights(&[EMBEDDING], next()),
    );
    put(
        "electra.embeddings_project.weight",
        weights(&[HIDDEN, EMBEDDING], next()),
    );
    put(
        "electra.embeddings_project.bias",
        weights(&[HIDDEN], next()),
    );
    for layer in 0..LAYERS {
        let at = format!("electra.encoder.layer.{layer}");
        for linear in [
            "attention.self.query",
            "attention.self.key",
            "attention.self.value",
            "attention.output.dense",
        ] {
            put(
                &format!("{at}.{linear}.weight"),
                weights(&[HIDDEN, HIDDEN], next()),
            );
            put(&format!("{at}.{linear}.bias"), weights(&[HIDDEN], next()));
        }
        put(
            &format!("{at}.intermediate.dense.weight"),
            weights(&[WIDE, HIDDEN], next()),
        );
        put(
            &format!("{at}.intermediate.dense.bias"),
            weights(&[WIDE], next()),
        );
        put(
            &format!("{at}.output.dense.weight"),
            weights(&[HIDDEN, WIDE], next()),
        );
        put(
            &format!("{at}.output.dense.bias"),
            weights(&[HIDDEN], next()),
        );
        for norm in ["attention.output.LayerNorm", "output.LayerNorm"] {
            put(&format!("{at}.{norm}.weight"), ones(HIDDEN));
            put(&format!("{at}.{norm}.bias"), weights(&[HIDDEN], next()));
        }
    }
    put(
        "generator_predictions.dense.weight",
        weights(&[EMBEDDING, HIDDEN], next()),
    );
    put(
        "generator_predictions.dense.bias",
        weights(&[EMBEDDING], next()),
    );
    put("generator_predictions.LayerNorm.weight", ones(EMBEDDING));
    put(
        "generator_predictions.LayerNorm.bias",
        weights(&[EMBEDDING], next()),
    );
    put("generator_lm_head.bias", weights(&[vocabulary], next()));
    let path = std::env::temp_dir().join(format!(
        "orthra-ml-local-{}.safetensors",
        std::process::id()
    ));
    candle_core::safetensors::save(&tensors, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    bytes
}

fn config(model_type: &str) -> String {
    format!(
        r#"{{"model_type": "{model_type}", "hidden_size": {HIDDEN}, "num_attention_heads": 2,
            "num_hidden_layers": {LAYERS}, "max_position_embeddings": {POSITIONS},
            "embedding_size": {EMBEDDING}, "layer_norm_eps": 1e-12}}"#
    )
}

fn scorer(prior: f32) -> ElectraScorer {
    let model = quantize(
        &tiny_safetensors(),
        &config("electra"),
        &VOCABULARY.join("\n"),
        prior,
    )
    .unwrap();
    ElectraScorer::read(&model).unwrap()
}

fn slot(sentence: &str, word: &str) -> TextRange {
    let start = sentence.find(word).unwrap();
    TextRange::at(
        TextSize::try_from(start).unwrap(),
        TextSize::try_from(word.len()).unwrap(),
    )
}

#[test]
fn a_candidate_scores_the_same_alone_as_padded_in_a_batch() {
    let scorer = scorer(0.0);
    let sentence = "please recieve the cat and fix it.";
    let at = slot(sentence, "recieve");
    // Of one, two and four pieces: the shorter ones are padded beside the longest.
    let candidates = ["receive", "fix", "relieves", "cafe"];
    let together = scorer.model_scores(sentence, at, &candidates).unwrap();
    for (candidate, score) in candidates.iter().zip(&together) {
        let alone = scorer.model_scores(sentence, at, &[candidate]).unwrap()[0];
        assert!(
            (alone - score).abs() < 1e-4,
            "{candidate}: alone {alone}, in a batch {score}"
        );
    }
    let again = scorer.model_scores(sentence, at, &candidates).unwrap();
    assert_eq!(together, again, "the same scores every time");
    assert!(
        together
            .iter()
            .all(|score| score.is_finite() && *score < 0.0),
        "{together:?}"
    );
}

#[test]
fn the_prior_weighs_the_lexical_order_in() {
    let plain = scorer(0.0);
    let weighed = scorer(0.25);
    let sentence = "we sat on the mat.";
    let at = slot(sentence, "mat");
    let candidates = ["mat", "cat", "box"];
    let model = plain.model_scores(sentence, at, &candidates).unwrap();
    let scored = weighed.score(sentence, at, &candidates).unwrap();
    for (position, (model, scored)) in model.iter().zip(&scored).enumerate() {
        let expected = model - 0.25 * ((position + 1) as f32).ln();
        assert!(
            (scored - expected).abs() < 1e-5,
            "{position}: {scored} against {expected}"
        );
    }
}

#[test]
fn only_an_electra_generator_is_quantized_and_a_model_file_must_be_one() {
    let errors = [
        quantize(
            &tiny_safetensors(),
            &config("bert"),
            &VOCABULARY.join("\n"),
            0.25,
        ),
        quantize(
            b"not weights",
            &config("electra"),
            &VOCABULARY.join("\n"),
            0.25,
        ),
    ]
    .map(|made| made.map(|_| ()).unwrap_err().0);
    expect_test::expect![[r#"
        config.json says "bert", and only an ELECTRA generator is read
        the weights: header too large"#]]
    .assert_eq(&errors.join("\n"));
    let refused = ElectraScorer::read(b"not a model").unwrap_err();
    assert!(refused.0.starts_with("the model file: "), "{refused}");
    let no_vocabulary =
        quantize(&tiny_safetensors(), &config("electra"), "the\ncat", 0.25).unwrap();
    expect_test::expect!["the vocabulary has no [UNK]"].assert_eq(
        &ElectraScorer::read(&no_vocabulary)
            .map(|_| ())
            .unwrap_err()
            .0,
    );
}
