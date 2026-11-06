//! The language server: transport, the document store, versions, scheduling.
//!
//! No crate below this one names a protocol type, and this one names no spelling semantics: it
//! asks `orthra-engine` and converts what comes back. A project is set up through
//! `orthra-project`, exactly as `orthra check` sets it up, so the editor and the command line
//! report the same findings (`CLAUDE.md` §8.3).

mod actions;
mod context;
mod convert;
mod documents;
mod install;
mod passes;
mod rewrite;
mod server;
mod state;

#[cfg(test)]
mod tests;

use std::{process::ExitCode, sync::Arc};

use orthra_engine::context::{Connector, Runtime};

use self::server::Backend;

/// What the server calls itself in the messages `lspkit_runtime` writes for it.
const NAME: &str = "orthra";

/// What the server tells a person whose settings enable a language that is installed, not built
/// in, and not installed yet: nothing in it is checked, which is not the same as nothing wrong.
fn not_installed(tag: &str, why: &str) -> String {
    format!(
        "orthra: {tag} is not installed, and {why}, so nothing written in it is checked. Install it \
         with `orthra pack install {tag}`."
    )
}

/// Serves the protocol over stdin and stdout until the client exits, on a runtime of its own:
/// what `orthra serve` runs. The exit status is the one the protocol asks for: 0 after `shutdown`,
/// 1 when `exit` came without it. `models` runs the context tier's model, when the binary has one
/// (`orthra-ml-local`), and `rewrites` reaches a model the user runs, when the binary can
/// (`orthra-ml-http`); this crate never names a model runtime or an HTTP client.
pub fn run(
    models: Option<Arc<dyn Runtime>>,
    rewrites: Option<Arc<dyn Connector>>,
) -> std::io::Result<ExitCode> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve(models, rewrites)))
}

/// Serves the protocol over stdin and stdout, through lspkit: a bug in one handler answers that
/// request with an error and the session carries on (`CLAUDE.md` §7), and how fast the client
/// reads is not how fast the server runs. Ends at `exit`, whether or not stdin is still open.
pub async fn serve(
    models: Option<Arc<dyn Runtime>>,
    rewrites: Option<Arc<dyn Connector>>,
) -> ExitCode {
    lspkit_runtime::serve_stdio(NAME, move |client| {
        Backend::new(client, models.clone(), rewrites.clone())
    })
    .await
    .exit_code()
}
