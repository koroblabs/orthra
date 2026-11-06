//! A project: the files a check reads, and what the project declares.
//!
//! The command line and the server both set their engine up from here, so that `orthra check`
//! reports what the editor would (`CLAUDE.md` §8.3): the same files, the same manifests, the same
//! declarations learned before anything is checked.

use std::{
    fs,
    num::NonZero,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use orthra_engine::{Declared, Engine, Finding};

pub mod config;
mod manifest;
mod walk;

pub use config::Config;

/// How many threads read a project's files, to learn it and to check it. Measured on phrost's
/// 1 395 files with one open, as the time to its diagnostics and the server's RSS (2026-11-05): one
/// thread 0.93–1.37 s at 28 MB, two 0.47–0.62 s, four 0.31 s at 29.5 MB, eight 0.20–0.23 s at 31
/// MB, twelve 0.19–0.28 s at 34–36 MB. Past four, each tenth of a second costs more memory than the
/// one before.
const THREADS: usize = 4;

/// The files of a project that a host claims, with their language, and the paths it was found
/// from.
#[derive(Debug)]
pub struct Project {
    paths: Vec<PathBuf>,
    documents: Vec<(PathBuf, &'static str)>,
    unclaimed: usize,
}

impl Project {
    /// The project at `paths`: every file under each that a host of `engine` claims and `config`
    /// does not ignore, in a fixed order.
    pub fn discover(paths: &[PathBuf], engine: &Engine, config: &Config) -> Project {
        let mut files = Vec::new();
        for path in paths {
            walk::files(path, config.ignore(), &mut files);
        }
        let mut documents = Vec::new();
        let mut unclaimed = 0;
        for file in files {
            match engine.language_for_path(&file.to_string_lossy()) {
                Some(language) => documents.push((file, language)),
                None => unclaimed += 1,
            }
        }
        Project {
            paths: paths.to_vec(),
            documents,
            unclaimed,
        }
    }

    /// The root of the project `path` belongs to: the nearest directory at or above it that holds
    /// an `orthra.toml` or a `.git`; failing that, the nearest with a manifest; failing that, its
    /// own directory. A member of a Cargo workspace has a manifest of its own, so the repository
    /// is looked for first: its settings, and every file it declares, are the member's too.
    pub fn root_of(path: &Path) -> PathBuf {
        let start = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let dir = if start.is_dir() {
            start.as_path()
        } else {
            start.parent().unwrap_or(Path::new("."))
        };
        let holding = |markers: &[&str]| {
            dir.ancestors()
                .find(|dir| markers.iter().any(|m| dir.join(m).exists()))
        };
        holding(&[config::PROJECT_FILE, ".git"])
            .or_else(|| holding(&["Cargo.toml", "package.json", "composer.json"]))
            .unwrap_or(dir)
            .to_path_buf()
    }

    /// The files a host claims, with the language each is read as.
    pub fn documents(&self) -> &[(PathBuf, &'static str)] {
        &self.documents
    }

    /// How many files no host claims.
    pub fn unclaimed(&self) -> usize {
        self.unclaimed
    }

    /// Checks every file of the project, on [`THREADS`] threads with a fork of `engine` each: a
    /// file is read, checked, handed to `found` with its place in [`Project::documents`], its text
    /// and its findings, and dropped, so memory follows the threads and not the project. `found`
    /// is called from those threads, in no set order; the place says where a file goes. Learn the
    /// project first. A file that cannot be read as UTF-8 is handed over with `None`.
    pub fn check(
        &self,
        engine: &Engine,
        found: impl Fn(usize, &Path, Option<(&str, Vec<Finding>)>) + Sync,
    ) {
        self.on_threads(
            || engine.fork(),
            |fork, at, file| match fs::read_to_string(file) {
                Ok(text) => {
                    let findings = fork.check_file(&file.to_string_lossy(), &text);
                    found(at, file, Some((&text, findings)));
                }
                Err(_) => found(at, file, None),
            },
        );
    }

    /// Teaches `engine` what the project declares: what its manifests name, and every name its
    /// files declare. Each file is read, learned and dropped, so memory follows the project's
    /// distinct words and not its size.
    ///
    /// Nothing is reported until the project is learned, so this is the wait before an editor's
    /// first diagnostic, and the files are read on several threads. What a file declares does not
    /// depend on any other, and a set does not depend on the order it was filled in.
    pub fn learn(&self, engine: &mut Engine) {
        for name in manifest::names(&self.paths) {
            engine.declare_name(&name);
        }
        let shared: &Engine = engine;
        let read = self.on_threads(Declared::default, |declared, at, file| {
            if let Ok(text) = fs::read_to_string(file) {
                shared.declarations(self.documents[at].1, &text, declared);
            }
        });
        for declared in read {
            engine.declare(declared);
        }
    }

    /// Hands every file to `work` on [`THREADS`] threads, each taking the next file as it is free,
    /// with a state of its own that `start` makes. The states, once every file is done.
    fn on_threads<T: Send>(
        &self,
        start: impl Fn() -> T + Sync,
        work: impl Fn(&mut T, usize, &Path) + Sync,
    ) -> Vec<T> {
        let threads = thread::available_parallelism().map_or(1, NonZero::get);
        let next = AtomicUsize::new(0);
        thread::scope(|scope| {
            let workers: Vec<_> = (0..threads.min(THREADS))
                .map(|_| {
                    scope.spawn(|| {
                        let mut state = start();
                        loop {
                            let at = next.fetch_add(1, Ordering::Relaxed);
                            let Some((file, _)) = self.documents.get(at) else {
                                return state;
                            };
                            work(&mut state, at, file);
                        }
                    })
                })
                .collect();
            workers
                .into_iter()
                .map(|worker| {
                    worker
                        .join()
                        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
                })
                .collect()
        })
    }
}
