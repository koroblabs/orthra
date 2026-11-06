//! `orthra explain <file>:<line>:<column>`: which span, which word, what each pack said, the verdict,
//! the finding, and suggestions. phrost's `query`, for this domain: the first tool to reach for when
//! a finding, or a silence, looks wrong.

use std::{fs, path::Path, process::ExitCode};

use orthra_engine::{Silenced, Why};
use orthra_project::Project;

use orthra_text::{
    TextSize,
    line_index::{Encoding, LineCol, LineIndex},
};

pub(crate) fn run(args: &[String]) -> ExitCode {
    let mut languages = Vec::new();
    let mut target = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--lang" {
            languages.extend(args.next().cloned());
        } else {
            target = Some(arg.clone());
        }
    }
    let Some((file, line, column)) = target.as_deref().and_then(parse) else {
        eprintln!("usage: orthra explain [--lang <tag>]... <file>:<line>:<column>");
        return ExitCode::from(2);
    };
    // The project the file belongs to, set up as `orthra check` sets it up: its settings, and what
    // it declares, so a word the project accepts is explained as accepted and not as unknown.
    let root = Project::root_of(Path::new(file));
    let mut config = crate::config(&root, false);
    config.add_languages(languages);
    for problem in config.problems() {
        eprintln!("orthra: {problem}");
    }
    let mut engine = match config.engine() {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("orthra: {err}");
            return ExitCode::from(2);
        }
    };
    for tag in engine.missing_packs() {
        eprintln!(
            "orthra: {tag} is not installed, so nothing in it is checked: `orthra pack install {tag}`"
        );
    }
    let Ok(text) = fs::read_to_string(file) else {
        eprintln!("orthra: {file}: not readable as UTF-8");
        return ExitCode::from(2);
    };
    if engine.language_for_path(file).is_none() {
        println!("{file}: no host claims this file, so nothing in it is checked");
        return ExitCode::SUCCESS;
    }
    if config.is_ignored(Path::new(file)) {
        println!(
            "{file}: the project's orthra.toml ignores this file, so nothing in it is checked"
        );
        return ExitCode::SUCCESS;
    }
    Project::discover(std::slice::from_ref(&root), &engine, &config).learn(&mut engine);
    let index = LineIndex::new(&text);
    let pos = LineCol {
        line: line.saturating_sub(1),
        col: column.saturating_sub(1),
    };
    let Some(at) = index.offset_in(&text, pos, Encoding::Utf32) else {
        eprintln!("orthra: {file} has no line {line}");
        return ExitCode::from(2);
    };
    let explanation = engine
        .explain(file, &text, at)
        .expect("the host was found by path");
    println!("{file}:{line}:{column}");
    println!("  host       {}", explanation.host);
    let Some(span) = explanation.span else {
        println!(
            "  span       none: this is not prose the host hands on (code, markup, a comment line)"
        );
        return ExitCode::SUCCESS;
    };
    println!(
        "  span       {:?} {}",
        span.kind,
        range(&index, &text, span.range.start(), span.range.end())
    );
    let Some(word) = explanation.word else {
        println!(
            "  word       none: segmentation made no word here (a URL, a path, a token with digits,"
        );
        println!(
            "             an acronym, fewer than three letters, an escape, code in backticks, or a"
        );
        println!("             comment line naming people, such as `Author:`)");
        return ExitCode::SUCCESS;
    };
    println!(
        "  word       `{}` {:?}, {:?}{}",
        word.text,
        word.word.shape,
        word.word.script,
        if word.word.code {
            ", from an identifier"
        } else {
            ""
        }
    );
    if word.answers.is_empty() {
        println!("  packs      none enabled for its script, so it is not checked");
    }
    for answer in &word.answers {
        let typo = answer
            .known_typo
            .as_ref()
            .map(|fixes| format!("; known typo -> {}", fixes.join(", ")))
            .unwrap_or_default();
        println!(
            "  {:<10} {}{typo}",
            answer.tag,
            match (answer.accepts, answer.accepts_rare) {
                (true, _) => "accepts it",
                (false, true) => "accepts it as a rare word, unless it is a known typo",
                (false, false) => "does not accept it",
            }
        );
    }
    println!(
        "  verdict    {:?}{}",
        word.verdict,
        match word.why {
            Why::WordsFile => ": a words file lists it, the project's .orthra/words.txt or yours",
            Why::Directive => ": this file's `orthra: ignore` lists it",
            Why::HostVocabulary => ": it is a word of the file's language",
            Why::WholeRun => ": the whole run it was cut from is a word",
            Why::DeclaredHere => ": this file declares it, and this is not the declaration",
            Why::DeclaredByProject =>
                ": the project declares it, in a manifest or another file, and this is not the declaration",
            Why::Packs => "",
        }
    );
    if let Some(silenced) = word.silenced {
        println!(
            "  unreported {}",
            match silenced {
                Silenced::Code =>
                    "it is code, which is held to known typos only: an unknown name is somebody's choice",
                Silenced::TestData =>
                    "it is a string in a test file: test data, which may misspell on purpose",
                Silenced::Short =>
                    "it has fewer than four letters: abbreviations far more often than typos",
            }
        );
    }
    match &word.finding {
        Some(finding) if finding.fixes.is_empty() => println!("  finding    {}", finding.code.0),
        Some(finding) => println!(
            "  finding    {} -> {}",
            finding.code.0,
            finding.fixes.join(", ")
        ),
        None => println!("  finding    none"),
    }
    if !word.suggestions.is_empty() {
        println!("  suggest    {}", word.suggestions.join(", "));
    }
    ExitCode::SUCCESS
}

/// `path:line:column`, splitting from the right so a path may hold a colon.
fn parse(target: &str) -> Option<(&str, u32, u32)> {
    let (rest, column) = target.rsplit_once(':')?;
    let (file, line) = rest.rsplit_once(':')?;
    Some((file, line.parse().ok()?, column.parse().ok()?))
}

fn range(index: &LineIndex, text: &str, start: TextSize, end: TextSize) -> String {
    format!(
        "{}..{}",
        crate::position(index, text, start),
        crate::position(index, text, end)
    )
}
