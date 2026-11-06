//! The budgets. This file is the source of truth for them; `CLAUDE.md` §3 quotes it.
//!
//! Each is a ceiling a change must stay under, measured by `orthra perf` on real files, not a
//! target to aim at. Most were set before anything was measured, and say so: a measurement that
//! sits far under one is a reason to lower it, and the headroom is the point.

use lspkit_perf::Budget;

pub(crate) const BUDGETS: &[Budget] = &[
    // Most of `initialize` → first diagnostics: reading the built-in packs.
    Budget::ms("pack load", 100.0),
    // The first document of a language: its grammar loaded and its query compiled, once per
    // process. The slowest language's. Measured on 2026-11-05: TypeScript 14 ms, PHP 9, Rust 8,
    // YAML 6, Python 6, Go 1, JSON 0.1, TOML 0.1.
    Budget::ms("grammar load, slowest", 50.0),
    // A 2 000-line file opened with an empty verdict cache, its grammar already loaded.
    Budget::ms("check per 2 000 lines, cold", 10.0),
    // A letter typed in the middle of an open document, checked again with what its last check
    // kept, the verdicts warm: one keystroke a file, and the p95 over the files. The target was
    // always per keystroke (`CLAUDE.md` §3: "diagnostics after one keystroke"). Until 2026-11-06
    // this row was the keystrokes' total per 2 000 lines, which multiplies a small file's fixed
    // cost: 55 µs a keystroke in starter workflows' 34-line files read as 3.3 ms, over, and a
    // 4 000-line file's slow keystroke hid among the small ones. The per-lines figure is still
    // printed. Before 2026-11-05 the row was a whole second check of each file.
    Budget::ms("keystroke p95", 2.0),
    // One word's suggestions, for one code action.
    Budget::ms("suggest p95", 15.0),
    Budget::mb("RSS after the run", 40.0),
];
