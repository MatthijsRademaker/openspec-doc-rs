//! Live-update plumbing: one broadcast channel per watched scope, fed by a
//! filesystem watcher, or by bounded polling when the watcher backend for that
//! scope fails to initialize.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// Updates buffered per subscriber. A subscriber that falls this far behind is
/// told it lagged, which we report as an update like any other.
const UPDATE_CAPACITY: usize = 16;

/// How often the fallback rescans a scope when no watcher is available.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// How long a scope's events must go quiet before they count as one edit.
const SETTLE: Duration = Duration::from_millis(100);

/// What one route scope watches: the directories handed to the filesystem
/// watcher, and the paths under them whose changes belong to that scope.
///
/// A scope spans more than one tree — a change's own directory and the sidecars
/// its comments and verdicts live in — so both are lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub watched_dirs: Vec<PathBuf>,
    pub relevant: Vec<PathBuf>,
}

/// The update channels behind the SSE endpoints, one per watched scope. Two
/// requests for the same scope watch the same paths, so the paths identify it.
#[derive(Default)]
pub struct Hub {
    scopes: Mutex<HashMap<Vec<PathBuf>, Watched>>,
}

struct Watched {
    updates: broadcast::Sender<()>,
    /// The scope's update source, kept alive with the entry: a dropped watcher
    /// stops reporting, and a dropped join handle loses the polling task.
    _source: Source,
}

enum Source {
    Watcher { _watcher: RecommendedWatcher },
    Polling { _task: JoinHandle<()> },
}

impl Hub {
    /// Receive an update every time `target`'s relevant path changes, starting
    /// the scope's watcher on the first subscription.
    pub fn subscribe(&self, target: &Target) -> broadcast::Receiver<()> {
        let mut scopes = self.scopes.lock().expect("hub lock");

        if let Some(watched) = scopes.get(&target.relevant) {
            return watched.updates.subscribe();
        }

        let (updates, receiver) = broadcast::channel(UPDATE_CAPACITY);
        let source = match watcher(target, updates.clone()) {
            Ok(watcher) => Source::Watcher { _watcher: watcher },
            Err(error) => {
                eprintln!(
                    "warning: no filesystem watcher for {} ({error}); polling every {}s instead",
                    describe(&target.relevant),
                    POLL_INTERVAL.as_secs()
                );
                Source::Polling {
                    _task: spawn_polling(target.relevant.clone(), updates.clone()),
                }
            }
        };

        scopes.insert(
            target.relevant.clone(),
            Watched {
                updates,
                _source: source,
            },
        );

        receiver
    }
}

/// Watch every one of `target.watched_dirs`, reporting only the events that
/// touch one of the scope's relevant paths.
fn watcher(
    target: &Target,
    updates: broadcast::Sender<()>,
) -> Result<RecommendedWatcher, notify::Error> {
    let (signals, bursts) = mpsc::channel(1);
    let relevant = target.relevant.clone();

    let mut watcher = notify::recommended_watcher(move |event| {
        let event: notify::Event = match event {
            Ok(event) => event,
            Err(error) => {
                eprintln!("warning: watch error for {}: {error}", describe(&relevant));
                return;
            }
        };

        // Reading a file also produces events, and the server reads the very
        // directories it watches, so only content changes may count as one.
        if !matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        ) {
            return;
        }

        let touched = event.paths.iter().any(|path| {
            relevant
                .iter()
                .any(|watched_path| path.starts_with(watched_path))
        });
        if touched {
            // A queue that is already full carries this signal just as well.
            let _ = signals.try_send(());
        }
    })?;

    for dir in &target.watched_dirs {
        // A sidecar directory is only created when its first record is written,
        // and a directory that does not exist yet cannot be watched — so the
        // scope's own state directory is created rather than watched blind.
        fs::create_dir_all(dir).map_err(notify::Error::io)?;
        watcher.watch(dir, RecursiveMode::Recursive)?;
    }
    tokio::spawn(coalesce(bursts, updates));

    Ok(watcher)
}

/// The scope's relevant paths, for a warning that has to name what it is about.
fn describe(relevant: &[PathBuf]) -> String {
    relevant
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Collapse the burst of events that one edit produces — a create, then a
/// write, then more — into the single update a page needs.
async fn coalesce(mut bursts: mpsc::Receiver<()>, updates: broadcast::Sender<()>) {
    // Ends when the scope's watcher is dropped and takes the sender with it.
    while bursts.recv().await.is_some() {
        tokio::time::sleep(SETTLE).await;
        while bursts.try_recv().is_ok() {}

        // Fails only when nobody is watching that scope's page right now.
        let _ = updates.send(());
    }
}

/// Start polling `relevant`, taking the baseline fingerprint before the task is
/// spawned so a change made right after this call cannot be missed.
fn spawn_polling(relevant: Vec<PathBuf>, updates: broadcast::Sender<()>) -> JoinHandle<()> {
    let baseline = fingerprint(&relevant);
    tokio::spawn(poll(relevant, baseline, updates))
}

async fn poll(relevant: Vec<PathBuf>, baseline: Fingerprint, updates: broadcast::Sender<()>) {
    let mut previous = baseline;
    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    ticker.tick().await; // The first tick completes immediately.

    loop {
        ticker.tick().await;

        let current = fingerprint(&relevant);
        if current != previous {
            previous = current;
            let _ = updates.send(());
        }
    }
}

type Fingerprint = Vec<(PathBuf, u64, Option<SystemTime>)>;

/// A stand-in for the content under `paths`: every file's size and modified
/// time. A file that vanishes mid-walk is simply absent, which is itself the
/// change the next comparison reports.
fn fingerprint(paths: &[PathBuf]) -> Fingerprint {
    let mut files = Vec::new();
    for path in paths {
        collect(path, &mut files);
    }
    files.sort();
    files
}

fn collect(path: &Path, files: &mut Fingerprint) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };

    if !metadata.is_dir() {
        files.push((path.to_owned(), metadata.len(), metadata.modified().ok()));
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect(&entry.path(), files);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn target_for(dir: &Path) -> Target {
        Target {
            watched_dirs: vec![dir.to_owned()],
            relevant: vec![dir.to_owned()],
        }
    }

    #[tokio::test]
    async fn a_write_under_the_watched_path_is_one_update() {
        let dir = TempDir::new().expect("temp dir");
        let hub = Hub::default();
        let mut updates = hub.subscribe(&target_for(dir.path()));

        fs::write(dir.path().join("a.md"), "one").expect("write file");

        let update = tokio::time::timeout(Duration::from_secs(10), updates.recv()).await;
        assert!(update.is_ok(), "the written file was never reported");
    }

    /// The server reads the very directories it watches, so a read counted as a
    /// change would leave every open page reloading itself in a loop.
    #[tokio::test]
    async fn reading_a_watched_file_is_not_an_update() {
        let dir = TempDir::new().expect("temp dir");
        let file = dir.path().join("a.md");
        fs::write(&file, "one").expect("write file");
        let hub = Hub::default();
        let mut updates = hub.subscribe(&target_for(dir.path()));

        fs::read_to_string(&file).expect("read file");
        fs::read_dir(dir.path()).expect("read dir").count();

        let update = tokio::time::timeout(Duration::from_secs(2), updates.recv()).await;
        assert!(update.is_err(), "reading was reported as a change");
    }

    /// The fallback has to stand on its own: it is the only update source when
    /// the watcher backend is unavailable.
    #[tokio::test]
    async fn polling_reports_a_change_under_the_watched_path() {
        let dir = TempDir::new().expect("temp dir");
        fs::write(dir.path().join("a.md"), "one").expect("write file");
        let (updates, mut receiver) = broadcast::channel(UPDATE_CAPACITY);

        let polling = spawn_polling(vec![dir.path().to_owned()], updates);
        fs::write(dir.path().join("a.md"), "one changed").expect("rewrite file");

        let update = tokio::time::timeout(Duration::from_secs(10), receiver.recv()).await;
        polling.abort();

        assert!(update.is_ok(), "polling never reported the rewritten file");
    }

    /// A scope spans separate trees — a change's directory and the sidecars its
    /// comments live in — and a write in either one is that scope's update.
    #[tokio::test]
    async fn a_write_in_any_watched_tree_is_an_update() {
        let dir = TempDir::new().expect("temp dir");
        let change = dir.path().join("change");
        let sidecars = dir.path().join("sidecars");
        fs::create_dir_all(&change).expect("create change dir");
        let hub = Hub::default();

        for written in [change.join("proposal.md"), sidecars.join("comments.jsonl")] {
            let mut updates = hub.subscribe(&Target {
                watched_dirs: vec![change.clone(), sidecars.clone()],
                relevant: vec![change.clone(), sidecars.clone()],
            });

            fs::write(&written, "content").expect("write file");

            let update = tokio::time::timeout(Duration::from_secs(10), updates.recv()).await;
            assert!(update.is_ok(), "{} was never reported", written.display());
        }
    }

    /// The comment and verdict sidecars only exist once a reviewer writes one,
    /// so a scope whose sidecar directory is still absent must not fall back to
    /// polling for want of a directory to watch.
    #[tokio::test]
    async fn a_sidecar_directory_that_does_not_exist_yet_is_still_watched() {
        let dir = TempDir::new().expect("temp dir");
        let sidecars = dir.path().join("comments/_session");
        let hub = Hub::default();

        let mut updates = hub.subscribe(&Target {
            watched_dirs: vec![sidecars.clone()],
            relevant: vec![sidecars.clone()],
        });
        fs::write(sidecars.join("session-a.jsonl"), "{}").expect("write sidecar");

        let update = tokio::time::timeout(Duration::from_secs(10), updates.recv()).await;
        assert!(update.is_ok(), "the first sidecar write was never reported");
    }
}
