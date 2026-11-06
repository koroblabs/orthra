//! `orthra check`.

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{Arc, Mutex, PoisonError},
    time::Instant,
};

use orthra_engine::context::{self, Runtime, Scorer, Window};
use orthra_project::Config;
use orthra_text::line_index::LineIndex;

use orthra_project::Project;

use lspkit_ratchet::{Baseline, Counts, Outcome};

/// What `--ratchet` does with the counts. Ordered, so that `--update-baseline` wins over
/// `--ratchet` whichever comes first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Ratchet {
    Off,
    /// Compare, and lower the baseline when a count fell.
    Compare,
    /// Write what was found, rise or not: a deliberate step, whose commit says why.
    Update,
}

pub(crate) fn run(args: &[String], runtime: Option<Arc<dyn Runtime>>) -> ExitCode {
    let mut languages = Vec::new();
    let mut paths = Vec::new();
    // Suggestions for each unknown word, for measuring them (`just typo-eval`). Slow on purpose:
    // the editor asks for one word at a time and never while diagnosing.
    let mut suggest = 0;
    // The context tier's model, reordering those suggestions as the server does when a code action
    // is asked for (`just eval-context`).
    let mut with_context = false;
    let mut ratchet = Ratchet::Off;
    // Built-in settings only, for an instrument whose numbers must not depend on whose machine or
    // which checkout it runs in: no user's settings, no project's.
    let mut isolated = false;
    // The files a check reads, one a line, and nothing checked: what `just compare` hands every
    // other spell checker, so each is asked about exactly the files orthra is.
    let mut list_files = false;
    let mut baseline: Option<PathBuf> = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--suggest" {
            match args.next().and_then(|n| n.parse().ok()) {
                Some(n) => suggest = n,
                None => return usage("--suggest needs a count"),
            }
        } else if arg == "--isolated" {
            isolated = true;
        } else if arg == "--context" {
            with_context = true;
        } else if arg == "--files" {
            list_files = true;
        } else if arg == "--ratchet" {
            ratchet = ratchet.max(Ratchet::Compare);
        } else if arg == "--update-baseline" {
            ratchet = Ratchet::Update;
        } else if arg == "--baseline" {
            match args.next() {
                Some(file) => baseline = Some(PathBuf::from(file)),
                None => return usage("--baseline needs a file"),
            }
            ratchet = ratchet.max(Ratchet::Compare);
        } else if arg == "--lang" {
            match args.next() {
                Some(tag) => languages.push(tag.clone()),
                None => return usage("--lang needs a tag, such as en-US"),
            }
        } else {
            paths.push(arg.clone());
        }
    }
    if paths.is_empty() {
        return usage("nothing to check");
    }
    let roots: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let mut config = crate::config(&roots[0], isolated);
    config.add_languages(languages);
    if !config.problems().is_empty() {
        for problem in config.problems() {
            eprintln!("orthra: {problem}");
        }
        return ExitCode::from(2);
    }
    let mut engine = match config.engine() {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("orthra: {err}");
            return ExitCode::from(2);
        }
    };
    // A language the settings name and nothing checks would pass a CI run that checked nothing:
    // it is installed first, as the server installs it, or the run stops.
    if !engine.missing_packs().is_empty() {
        let Some(packs) = config.packs().filter(|_| config.auto_install()) else {
            for tag in engine.missing_packs() {
                eprintln!("orthra: {tag} is not installed: `orthra pack install {tag}`");
            }
            return ExitCode::from(2);
        };
        for tag in engine.missing_packs() {
            if let Err(err) = crate::pack::fetch_and_install(tag, packs) {
                eprintln!("orthra: {err}");
                return ExitCode::from(2);
            }
        }
        engine = match config.engine() {
            Ok(engine) => engine,
            Err(err) => {
                eprintln!("orthra: {err}");
                return ExitCode::from(2);
            }
        };
    }

    // The project first: what its manifests name and what its files declare is its vocabulary, so
    // every file is learned before any is checked. Then each is read again, checked and dropped, on
    // the project's threads: what it found is kept as what to print, not its text, and printed in
    // the project's order once every file is done.
    let project = Project::discover(&roots, &engine, &config);
    if list_files {
        for (file, _) in project.documents() {
            println!("{}", file.display());
        }
        return ExitCode::SUCCESS;
    }
    let scorer = match with_context.then(|| scorer(runtime)) {
        Some(Ok(scorer)) => Some(scorer),
        Some(Err(why)) => {
            eprintln!("orthra check --context: {why}");
            return ExitCode::from(2);
        }
        None => None,
    };
    project.learn(&mut engine);
    let unclaimed = project.unclaimed();
    let reports = Mutex::new(Vec::new());
    project.check(&engine, |at, file, found| {
        let found = found.map(|(text, findings)| {
            let index = LineIndex::new(text);
            findings
                .iter()
                .map(|finding| Found {
                    position: crate::position(&index, text, finding.range.start()),
                    code: finding.code.0,
                    word: text[finding.range].to_owned(),
                    fixes: finding.fixes.clone(),
                    window: (with_context && finding.code.0 == "unknown-word")
                        .then(|| Window::around(text, finding.range)),
                })
                .collect::<Vec<_>>()
        });
        let name = file.to_string_lossy().into_owned();
        reports
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((at, name, found));
    });
    let mut reports = reports.into_inner().unwrap_or_else(PoisonError::into_inner);
    reports.sort_unstable_by_key(|(at, ..)| *at);

    let (mut checked, mut findings) = (0, 0);
    let mut reordered: Vec<f64> = Vec::new();
    let mut by_code = Counts::new();
    for (_, name, found) in reports {
        let Some(found) = found else {
            eprintln!("orthra: {name}: not readable as UTF-8");
            continue;
        };
        checked += 1;
        for finding in found {
            findings += 1;
            if ratchet != Ratchet::Off {
                *by_code.entry(finding.code.to_owned()).or_default() += 1;
                continue;
            }
            let mut fixes = finding.fixes;
            if fixes.is_empty() && suggest > 0 {
                fixes = engine.suggest(&finding.word, suggest);
                if let (Some(scorer), Some(window)) = (&scorer, &finding.window)
                    && fixes.len() > 1
                {
                    let started = Instant::now();
                    match context::rerank(scorer.as_ref(), window, fixes.clone()) {
                        Ok(order) => fixes = order,
                        Err(err) => eprintln!("orthra: the context tier could not score: {err}"),
                    }
                    reordered.push(started.elapsed().as_secs_f64() * 1e3);
                }
            }
            let fixes = if fixes.is_empty() {
                String::new()
            } else {
                format!(" -> {}", fixes.join(", "))
            };
            println!(
                "{name}:{}: {}: {}{fixes}",
                finding.position, finding.code, finding.word
            );
        }
    }
    eprintln!(
        "orthra: {findings} findings in {checked} files checked; {unclaimed} files no host claims"
    );
    if let Some(scorer) = &scorer {
        reordered.sort_by(f64::total_cmp);
        let at = |share: f64| {
            reordered
                .get(((reordered.len() as f64 - 1.0) * share).round() as usize)
                .copied()
                .unwrap_or_default()
        };
        eprintln!(
            "orthra: the context tier reordered {} words' corrections, p50 {:.1} ms, p95 {:.1} ms, \
             with a model of {:.1} MB",
            reordered.len(),
            at(0.5),
            at(0.95),
            scorer.bytes() as f64 / 1e6
        );
    }
    if ratchet != Ratchet::Off {
        let file = baseline.unwrap_or_else(|| default_baseline(&paths[0]));
        return compare(&file, &by_code, ratchet);
    }
    if findings > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// A finding as `orthra check` prints it, kept from its file's text until it is printed.
struct Found {
    position: String,
    code: &'static str,
    word: String,
    fixes: Vec<String>,
    /// The line around an unknown word, for the context tier to score its corrections in.
    window: Option<Window>,
}

/// The context tier's scorer for `--context`: the installed `small` model, read by `runtime`.
fn scorer(runtime: Option<Arc<dyn Runtime>>) -> Result<Arc<dyn Scorer>, String> {
    let runtime =
        runtime.ok_or("this orthra was built without a model runtime (feature `ml-local`)")?;
    let dir = Config::models_dir().ok_or("no data directory: set XDG_DATA_HOME or HOME")?;
    let model = orthra_install::models::model("small").map_err(|err| err.0)?;
    if !model.is_installed(&dir) {
        return Err("model small is not installed: `orthra model install small`".to_owned());
    }
    let file = model.file(&dir);
    let bytes = std::fs::read(&file).map_err(|err| format!("{}: {err}", file.display()))?;
    runtime.load(&bytes).map_err(|err| err.0)
}

/// `orthra-baseline.toml` beside the project: in `path` if it is a directory, else beside it.
fn default_baseline(path: &str) -> PathBuf {
    let path = Path::new(path);
    let dir = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(Path::new("."))
    };
    dir.join("orthra-baseline.toml")
}

/// The counts found against the baseline in `file`, which is lowered when a count fell and
/// rewritten whole on `--update-baseline`. Exit 1 when a count rose and nobody said it should.
fn compare(file: &Path, found: &Counts, ratchet: Ratchet) -> ExitCode {
    let shown = file.display();
    let baseline = match Baseline::load(file) {
        Ok(baseline) => baseline.unwrap_or_else(|| Baseline::new(None, Counts::new())),
        Err(err) => {
            eprintln!("orthra: {shown}: {err}");
            return ExitCode::from(2);
        }
    };
    let comparison = baseline.compare(found);
    print!("{shown}:\n{}", comparison.report());
    let write = |why: &str| match Baseline::new(None, found.clone())
        .store(file, "orthra check --ratchet --update-baseline")
    {
        Ok(()) => {
            println!("{why}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("orthra: {shown}: {err}");
            ExitCode::from(2)
        }
    };
    // A count that falls is written back on its own, so the baseline only goes down; orthra's
    // baselines live in this repository, beside the code that moved them.
    match (comparison.outcome(), ratchet) {
        (_, Ratchet::Update) => write("baseline written"),
        (Outcome::Fell, _) => write("baseline lowered"),
        (Outcome::Held, _) => ExitCode::SUCCESS,
        (Outcome::Rose, _) => {
            println!("findings rose: fix them, or record them with --update-baseline and say why");
            ExitCode::from(1)
        }
    }
}

fn usage(why: &str) -> ExitCode {
    eprintln!(
        "orthra check: {why}\nusage: orthra check [--lang <tag>]... [--suggest <n> [--context]] [--ratchet] [--baseline <file>] [--update-baseline] [--files] <path>..."
    );
    ExitCode::from(2)
}
