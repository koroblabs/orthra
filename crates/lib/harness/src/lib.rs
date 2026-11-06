//! The invariant dimensions every fixture runs through. See `docs/02-testing.md` level 4.
//!
//! A dimension is not a test. It is a question asked of *every* fixture, so one new fixture buys
//! all of them. The harness is generic over [`subject::Subject`] and knows nothing about spelling.
//! That keeps it below the engine in the crate graph, so the engine's own tests can use it.

pub mod dimensions;
pub mod subject;

#[cfg(test)]
mod tests;
