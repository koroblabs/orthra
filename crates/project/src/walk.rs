//! The files of a project, in a fixed order.

use std::path::{Path, PathBuf};

use lspkit_walk::{Override, Walk};

#[cfg(test)]
mod tests;

/// Every file under `path`, or `path` itself, sorted. What the project leaves out of git is left
/// out here too (`.gitignore`, `.ignore`, `.git/info/exclude`), with hidden directories and the
/// build and dependency directories nobody writes by hand, `target` and `node_modules`, whether
/// or not they are ignored. The walk is lspkit's, shared with phrost.
///
/// The user's global git excludes are not read: they differ from one machine to the next, and a
/// project's findings must not (`CLAUDE.md` §1.2). A `.gitignore` counts without a `.git`, so a
/// project unpacked from a tarball reads as it does checked out. What the project's `ignore`
/// setting names is pruned too.
pub(crate) fn files(path: &Path, ignore: &Override, out: &mut Vec<PathBuf>) {
    out.extend(
        Walk::new([path])
            .overrides(ignore.clone())
            .prune(|entry| {
                entry.is_dir
                    && matches!(
                        entry.path.file_name().and_then(|name| name.to_str()),
                        Some("target" | "node_modules")
                    )
            })
            .files(),
    );
}
