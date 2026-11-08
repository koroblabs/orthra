//! The context tier through an OpenAI-compatible endpoint the user runs -- Ollama, llama.cpp's
//! server, LM Studio -- for the one thing a generative model does in orthra: rewrite the sentence
//! somebody asked to have rewritten (`docs/04-context.md` §2 C). orthra does not care which model
//! it is; the user runs it and manages it.
//!
//! The only crate that sends a document's text anywhere, and a leaf behind the binary's `ml-http`
//! feature (`CLAUDE.md` §1.1). Text goes to this machine only: before every request, the
//! endpoint's host must be a loopback address unless the user wrote `allowRemote` ([`guard`]). No
//! proxy is used and no redirect followed, either of which would hand the text to another machine
//! whatever the endpoint says.

use std::{sync::Arc, time::Duration};

use orthra_context::{ApiKey, Connector, Endpoint, Prompts, Rewrite, Rewriter, ScoreError};
use serde_json::{Value, json};

mod answer;
mod guard;

#[cfg(test)]
mod tests;

/// The longest a rewrite may take. A 4B model on a laptop's CPU rewrites a sentence in seconds,
/// and one the server has not loaded yet loads first; past this, the person is not waiting any
/// more.
const TIMEOUT: Duration = Duration::from_secs(20);

/// The most of an answer read, in bytes. A sentence is under a kilobyte: this is room for a model
/// that thinks out loud before it answers, not a target.
const RESPONSE_LIMIT: u64 = 256 * 1024;

/// What the model is told before text to correct, unless the settings give their own.
/// `{language}` is where the language is named.
const CORRECT: &str = "You correct the spelling and grammar of text. The user's message is one \
    sentence of a document, or part of one{language}. Keep its meaning, keep its wording wherever \
    it is correct, and keep its markup and line breaks exactly as they are: Markdown, backticks, \
    links, code. Do not complete it, explain it or add to it. Reply with the corrected text only, \
    without quotes.";

/// What the model is told before text to rephrase, unless the settings give their own.
const REPHRASE: &str = "You rephrase text so that it reads naturally. The user's message is part of \
    a document{language}. Keep its meaning and its language, and keep its markup and line breaks \
    exactly as they are: Markdown, backticks, links, code. Correct its spelling and grammar too. Do \
    not complete it, explain it or add to it. Reply with the rephrased text only, without quotes.";

/// Connects to an endpoint by HTTP: what the binary hands the server.
#[derive(Debug, Clone, Copy)]
pub struct Http {
    timeout: Duration,
}

impl Default for Http {
    fn default() -> Http {
        Http { timeout: TIMEOUT }
    }
}

impl Http {
    /// A connector whose rewrites give up after `timeout`, for a test that cannot wait 20 s.
    #[cfg(test)]
    fn timing_out_after(timeout: Duration) -> Http {
        Http { timeout }
    }
}

impl Connector for Http {
    fn connect(&self, endpoint: &Endpoint) -> Result<Arc<dyn Rewriter>, ScoreError> {
        Ok(Arc::new(HttpRewriter::new(endpoint, self.timeout)?))
    }
}

/// A sentence rewritten by the model an OpenAI-compatible endpoint serves, through its
/// `/chat/completions`, at temperature 0.
#[derive(Debug)]
struct HttpRewriter {
    /// `<endpoint>/chat/completions`, checked by the guard.
    url: String,
    model: String,
    allow_remote: bool,
    api_key: Option<ApiKey>,
    prompts: Prompts,
    agent: ureq::Agent,
}

impl HttpRewriter {
    /// A rewriter asking `endpoint`, or why it may not: checked now, so that a refused endpoint
    /// is said when the settings name it, and again before every request.
    fn new(endpoint: &Endpoint, timeout: Duration) -> Result<HttpRewriter, ScoreError> {
        let url = format!("{}/chat/completions", endpoint.url.trim_end_matches('/'));
        guard::checked(&url, endpoint.allow_remote)?;
        if endpoint.model.trim().is_empty() {
            return Err(ScoreError(format!(
                "no model is named for {}",
                endpoint.url
            )));
        }
        let agent = ureq::Agent::config_builder()
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            // Never a proxy from the environment, and never where a redirect points: the guard
            // checks one URL, and the text goes to that URL or nowhere.
            .proxy(None)
            .max_redirects(0)
            // An error's body says why, `model "qwen3:4b" not found`: read it rather than lose it.
            .http_status_as_error(false)
            .timeout_global(Some(timeout))
            .build()
            .into();
        Ok(HttpRewriter {
            url,
            model: endpoint.model.clone(),
            allow_remote: endpoint.allow_remote,
            api_key: endpoint.api_key.clone(),
            prompts: endpoint.prompts.clone(),
            agent,
        })
    }

    /// The request: the prompt for `how`, the settings' or orthra's, the text, and no randomness.
    fn body(&self, text: &str, lang: &str, how: Rewrite) -> Value {
        let language = if lang == "und" {
            String::new()
        } else {
            format!(", written in {lang}")
        };
        let prompt = self.prompts.of(how).unwrap_or(match how {
            Rewrite::Correct => CORRECT,
            Rewrite::Rephrase => REPHRASE,
        });
        json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": prompt.replace("{language}", &language) },
                { "role": "user", "content": text },
            ],
            "temperature": 0,
            "stream": false,
        })
    }
}

impl Rewriter for HttpRewriter {
    fn rewrite(&self, text: &str, lang: &str, how: Rewrite) -> Result<String, ScoreError> {
        // Here, before the request, and not only when connected: this is where text leaves.
        let uri = guard::checked(&self.url, self.allow_remote)?;
        let failed = |why: String| ScoreError(format!("{}: {why}", self.url));
        let mut request = self
            .agent
            .post(uri)
            .header("content-type", "application/json");
        if let Some(ApiKey(key)) = &self.api_key {
            request = request.header("authorization", format!("Bearer {key}"));
        }
        let mut response = request
            .send(self.body(text, lang, how).to_string())
            .map_err(|err| failed(err.to_string()))?;
        let status = response.status();
        let body = response
            .body_mut()
            .with_config()
            .limit(RESPONSE_LIMIT)
            .read_to_string()
            .map_err(|err| failed(err.to_string()))?;
        if !status.is_success() {
            return Err(failed(format!("answered {status}{}", said(&body))));
        }
        answer::read(text, &body).map_err(|err| failed(err.0))
    }
}

/// What an endpoint's error says, after a colon: OpenAI's `{"error": {"message": ...}}`, or
/// llama.cpp's and older Ollamas' `{"error": "..."}`. Nothing when it says nothing readable.
fn said(body: &str) -> String {
    let Ok(error) = serde_json::from_str::<Value>(body) else {
        return String::new();
    };
    let message = error["error"]["message"]
        .as_str()
        .or_else(|| error["error"].as_str());
    message.map_or_else(String::new, |message| {
        format!(": {}", message.chars().take(200).collect::<String>())
    })
}
