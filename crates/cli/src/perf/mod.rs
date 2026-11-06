//! `orthra perf <path>...`: every budget in [`budget`], measured on real files, with exit status 1
//! when one is exceeded.
//!
//! Run it on an idle machine and on a release build. A concurrent `cargo build` reads exactly like
//! a regression (`CLAUDE.md` §3).

mod budget;

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use lspkit_perf::{Samples, Verdict, rss};
use orthra_engine::Parsed;
use orthra_project::{Config, Project};
use orthra_text::{TextRange, TextSize};

use self::budget::BUDGETS;

pub(crate) fn run(args: &[String]) -> ExitCode {
    if args.is_empty() {
        eprintln!("usage: orthra perf <path>...");
        return ExitCode::from(2);
    }
    let started = Instant::now();
    // Built in only, like every instrument: the budgets are orthra's, not anyone's settings.
    let config = Config::builtin(Path::new("."));
    let mut engine = match config.engine() {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("orthra: {err}");
            return ExitCode::from(2);
        }
    };
    // A check reads a pack's dictionaries on the first word of its script; here they are read at
    // once, so the row is what a first check pays for them.
    engine.load_packs();
    let load = ms(started);

    let roots: Vec<PathBuf> = args.iter().map(PathBuf::from).collect();
    // Paths only: a server holds the documents that are open, not the project, so neither does
    // this. Each file is read, checked and dropped, and only the check is timed.
    let documents: Vec<(String, &str)> = Project::discover(&roots, &engine, &config)
        .documents()
        .iter()
        .map(|(file, language)| (file.to_string_lossy().into_owned(), *language))
        .collect();
    // Each language's grammar loads on its first document, once per process, like the packs; it
    // is timed apart so that a small project's cold check does not carry it.
    let mut languages: Vec<&str> = documents.iter().map(|(_, language)| *language).collect();
    languages.sort_unstable();
    languages.dedup();
    let grammar = languages
        .iter()
        .map(|language| {
            let started = Instant::now();
            engine.check(language, "");
            ms(started)
        })
        .fold(0.0, f64::max);
    let read = |name: &str| fs::read_to_string(name).unwrap_or_default();
    let mut lines = 0;
    let mut cold = 0.0;
    let mut unknown = Vec::new();
    engine.bump_epoch();
    for (name, language) in &documents {
        let text = read(name);
        lines += text.lines().count();
        let started = Instant::now();
        let findings = engine.check(language, &text);
        cold += ms(started);
        for finding in findings {
            if finding.code.0 == "unknown-word" && unknown.len() < 300 {
                unknown.push(text[finding.range].to_owned());
            }
        }
    }
    if lines == 0 {
        eprintln!("orthra perf: no file under {args:?} has a host");
        return ExitCode::from(2);
    }
    // A keystroke in an open document: checked once and its parse kept, as the server keeps the
    // parse of the document being typed in, then a letter typed in the middle and checked again.
    let mut keystroke = 0.0;
    let mut keystrokes = Samples::new("keystroke");
    for (name, language) in &documents {
        let mut text = read(name);
        let mut parsed = Parsed::default();
        engine.check_reusing(language, &text, &mut parsed);
        let Some(at) = middle_letter(&text) else {
            continue;
        };
        let letter = text[at..at + 1].to_owned();
        let range = TextRange::empty(TextSize::try_from(at).expect("a file under 4 GiB"));
        parsed.edit(&text, range, &letter);
        text.insert_str(at, &letter);
        let started = Instant::now();
        engine.check_reusing(language, &text, &mut parsed);
        keystrokes.record(started.elapsed(), true, name);
        keystroke += ms(started);
    }

    let mut suggest = Samples::new("suggest");
    for word in &unknown {
        let started = Instant::now();
        engine.suggest(word, 5);
        suggest.record(started.elapsed(), true, word);
    }
    let in_ms = |duration: Option<Duration>| duration.map(|taken| taken.as_secs_f64() * 1000.0);

    let per = |total: f64| total / lines as f64 * 2000.0;
    let measured = [
        Some(load),
        Some(grammar),
        Some(per(cold)),
        in_ms(keystrokes.percentile(95)),
        in_ms(suggest.percentile(95)),
        rss::current().map(rss::megabytes),
    ];
    println!(
        "orthra perf: a keystroke in each file: p50 {:.2} ms, max {:.2} ms{}; {:.2} ms per 2 000 lines",
        in_ms(keystrokes.percentile(50)).unwrap_or_default(),
        in_ms(keystrokes.max()).unwrap_or_default(),
        keystrokes
            .worst()
            .map(|worst| format!(" in {}", worst.location))
            .unwrap_or_default(),
        per(keystroke)
    );
    println!(
        "orthra perf: {} files, {lines} lines, {} words suggested for (p50 {:.2} ms, max {:.2} ms{})",
        documents.len(),
        suggest.len(),
        in_ms(suggest.percentile(50)).unwrap_or_default(),
        in_ms(suggest.max()).unwrap_or_default(),
        suggest
            .worst()
            .map(|worst| format!(", `{}`", worst.location))
            .unwrap_or_default()
    );
    let mut verdict = Verdict::new();
    for (budget, value) in BUDGETS.iter().zip(measured) {
        let judged = verdict.judge(budget, value);
        println!(
            "  {:<30} {:>9.2} {:<2}  budget {:>6.1} {:<2}  {}",
            budget.what,
            value.unwrap_or(f64::NAN),
            budget.unit,
            budget.limit,
            budget.unit,
            judged.word()
        );
    }
    match verdict.into_result() {
        Ok(()) => ExitCode::SUCCESS,
        Err(over) => {
            eprintln!("orthra perf: {over}");
            ExitCode::from(1)
        }
    }
}

/// The first ASCII letter at or after the middle of `text`: where a keystroke doubles it, as
/// typing does.
fn middle_letter(text: &str) -> Option<usize> {
    let middle = text.len() / 2;
    text.as_bytes()[middle..]
        .iter()
        .position(u8::is_ascii_alphabetic)
        .map(|offset| middle + offset)
}

fn ms(since: Instant) -> f64 {
    since.elapsed().as_secs_f64() * 1000.0
}
