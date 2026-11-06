//! `orthra`: the one binary.
//!
//! - `orthra check [--lang <tag>]... <path>...` — every file a host claims, recursively, and exit
//!   status 1 when anything is found: CI and pre-commit. With `--ratchet`, the findings are counted
//!   by code against `orthra-baseline.toml`, and only a rise fails. With `--files`, the files it would
//!   check, one a line, and nothing checked.
//! - `orthra explain <file>:<line>:<column>` — everything decided about the word there.
//! - `orthra pack list`, `orthra pack install <tag> [--from <file>]...` — the packs orthra does not
//!   carry, installed where checks and the server read them.
//! - `orthra model list`, `orthra model install <name> [--from <file>]...` — the context tier's
//!   models, built by the in-process runtime (feature `ml-local`).
//! - `orthra perf <path>...` — every budget in `perf/budget.rs`, measured; exit 1 when one is over.
//! - `orthra serve` — the language server, on stdin and stdout. It rewrites a sentence through an
//!   endpoint the user runs when the settings name one (feature `ml-http`).

use std::{path::Path, process::ExitCode, sync::Arc};

use orthra_engine::context::{Connector, Runtime};

use orthra_project::{Config, Project};
use orthra_text::line_index::{Encoding, LineIndex};

/// Large blocks from the operating system, given back when freed past a bounded cache: what the
/// context tier's model and its activations are made of, which macOS's allocator would keep.
#[global_allocator]
static HEAP: lspkit_heap::Heap = lspkit_heap::Heap;

mod check;
mod explain;
mod model;
mod pack;
mod perf;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("orthra {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("check") => check::run(&args[1..], runtime()),
        Some("explain") => explain::run(&args[1..]),
        Some("model") => model::run(&args[1..], runtime()),
        Some("pack") => pack::run(&args[1..]),
        Some("perf") => perf::run(&args[1..]),
        Some("serve") => match orthra_lsp::run(runtime(), rewrites()) {
            Ok(code) => code,
            Err(err) => {
                eprintln!("orthra serve: {err}");
                ExitCode::from(2)
            }
        },
        _ => {
            eprintln!(
                "usage: orthra check [--lang <tag>]... [--suggest <n>] [--ratchet] <path>...\n       orthra explain [--lang <tag>]... <file>:<line>:<column>\n       orthra pack list | install <tag> [--from <file>]...\n       orthra model list | install <name> [--from <file>]...\n       orthra perf <path>...\n       orthra serve"
            );
            ExitCode::from(2)
        }
    }
}

/// The context tier's model runtime this binary was built with: candle's, in process, unless the
/// `ml-local` feature was left out.
fn runtime() -> Option<Arc<dyn Runtime>> {
    #[cfg(feature = "ml-local")]
    {
        Some(Arc::new(orthra_ml_local::Local))
    }
    #[cfg(not(feature = "ml-local"))]
    {
        None
    }
}

/// What reaches a model the user runs, for "Rewrite sentence (local model)": an OpenAI-compatible
/// endpoint over HTTP, unless the `ml-http` feature was left out.
fn rewrites() -> Option<Arc<dyn Connector>> {
    #[cfg(feature = "ml-http")]
    {
        Some(Arc::new(orthra_ml_http::Http::default()))
    }
    #[cfg(not(feature = "ml-http"))]
    {
        None
    }
}

/// `line:column`, one-based, the column in characters, as compilers print it.
fn position(index: &LineIndex, text: &str, at: orthra_text::TextSize) -> String {
    match index.line_col_in(text, at, Encoding::Utf32) {
        Some(pos) => format!("{}:{}", pos.line + 1, pos.col + 1),
        None => "?:?".to_owned(),
    }
}

/// The settings for the project `path` belongs to: every layer, or built in only when `isolated`.
fn config(path: &Path, isolated: bool) -> Config {
    let root = Project::root_of(path);
    if isolated {
        Config::builtin(&root)
    } else {
        Config::load(&root, Config::user_dir().as_deref())
    }
}
