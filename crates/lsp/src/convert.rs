//! Between the protocol's types and the engine's. Positions are converted by lspkit's
//! `lspkit_text::lsp`, which phrost's server uses too, and nowhere else (`CLAUDE.md` §8.1).

use lspkit_runtime::protocol::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};
use orthra_engine::Finding;
use orthra_text::line_index::{Encoding, LineIndex};

/// The `source` every diagnostic carries, so a reader can tell the dictionary's certainty from a
/// model's opinion when the context tier arrives (`docs/04-context.md`).
pub(crate) const SOURCE: &str = "orthra";

/// The protocol's severity for a setting's: `warning` when it names none.
pub(crate) fn severity(setting: &str) -> DiagnosticSeverity {
    match setting {
        "error" => DiagnosticSeverity::ERROR,
        "information" => DiagnosticSeverity::INFORMATION,
        "hint" => DiagnosticSeverity::HINT,
        _ => DiagnosticSeverity::WARNING,
    }
}

/// A finding as the client shows it, with the severity the settings ask for. A spelling is not a
/// fault in the code, but Zed lists only errors and warnings among a project's problems, so a
/// warning is the default (open question 3, `docs/03-roadmap.md`).
pub(crate) fn diagnostic(
    index: &LineIndex,
    text: &str,
    finding: &Finding,
    encoding: Encoding,
    severity: DiagnosticSeverity,
) -> Option<Diagnostic> {
    let range = lspkit_text::lsp::range(index, text, finding.range, encoding)?;
    let word = &text[finding.range];
    let message = match finding.code.0 {
        "known-typo" => match finding.fixes.first() {
            Some(fix) => format!("`{word}` is a misspelling of `{fix}`"),
            None => format!("`{word}` is a known misspelling"),
        },
        "mixed-script" => format!("`{word}` mixes letters from more than one script"),
        _ => format!("`{word}` is not a known word"),
    };
    Some(Diagnostic {
        range,
        severity: Some(severity),
        code: Some(NumberOrString::String(finding.code.0.to_owned())),
        source: Some(SOURCE.to_owned()),
        message,
        ..Diagnostic::default()
    })
}
