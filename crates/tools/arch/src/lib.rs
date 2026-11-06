//! orthra's architecture rules, as data. See `CLAUDE.md` §1.1 and §1.2.
//!
//! The engine that reads the workspace and reports is `lspkit-arch`, shared with phrost. The rules
//! are orthra's and live here:
//!
//! - [`dependency_rules`]: which of our crates may depend on which, and which one crate may name
//!   each foreign engine.
//! - [`no_fallbacks`]: every filesystem read on the path a request takes, each with its reason.
//! - [`baseline`]: what lspkit asks of every server's workspace, checked here.
//!
//! Everything here is test code; the crate builds empty.

#[cfg(test)]
mod baseline;
#[cfg(test)]
mod dependency_rules;
#[cfg(test)]
mod no_fallbacks;
