//! A commit message: its lines are prose, except comment lines, trailers, and everything below the
//! scissors line that `git commit -v` adds above the diff.

use orthra_text::{TextRange, TextSize};

use crate::{Conventions, Host, Span, SpanKind};

#[derive(Debug)]
pub struct GitCommit;

impl Host for GitCommit {
    fn id(&self) -> &'static str {
        "git-commit"
    }

    fn language_ids(&self) -> &'static [&'static str] {
        &["git-commit", "gitcommit"]
    }

    fn file_names(&self) -> &'static [&'static str] {
        &["COMMIT_EDITMSG", "MERGE_MSG", "TAG_EDITMSG"]
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[]
    }

    fn conventions(&self) -> Conventions {
        Conventions::default()
    }

    fn spans(&self, text: &str, out: &mut Vec<Span>) {
        let size = |at: usize| TextSize::try_from(at).expect("a document under 4 GiB");
        let mut at = 0;
        for line in text.split_inclusive('\n') {
            let start = at;
            at += line.len();
            let content = line.trim_end_matches(['\n', '\r']);
            if content.starts_with('#') && content.contains(">8") {
                break;
            }
            if content.starts_with('#') || content.trim().is_empty() || is_trailer(content) {
                continue;
            }
            out.push(Span {
                range: TextRange::new(size(start), size(start + content.len())),
                kind: SpanKind::Prose,
            });
        }
    }
}

/// `Signed-off-by: …`, `Co-authored-by: …`, `Fixes: …`: a token, a colon, a space, a value.
fn is_trailer(line: &str) -> bool {
    let Some((key, value)) = line.split_once(": ") else {
        return false;
    };
    !value.is_empty()
        && key.starts_with(|c: char| c.is_ascii_uppercase())
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && (key.contains('-')
            || ["Fixes", "Closes", "Refs", "Resolves", "Reported", "See"].contains(&key))
}
