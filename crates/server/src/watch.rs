//! Live-update plumbing: one broadcast channel per watched scope, fed by a
//! filesystem watcher, or by bounded polling when the watcher backend for that
//! scope fails to initialize.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
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
    pub artifacts: Vec<PathBuf>,
    pub review_state: Vec<PathBuf>,
}

impl Target {
    fn relevant(&self) -> Vec<PathBuf> {
        self.artifacts
            .iter()
            .chain(&self.review_state)
            .cloned()
            .collect()
    }
}

/// What changed in one scope update. Both flags can be true when one logical
/// operation touches an artifact and its review sidecar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    pub artifacts_changed: bool,
    pub review_state_changed: bool,
}

impl Update {
    pub const fn all() -> Self {
        Self {
            artifacts_changed: true,
            review_state_changed: true,
        }
    }

    fn merge(&mut self, other: Self) {
        self.artifacts_changed |= other.artifacts_changed;
        self.review_state_changed |= other.review_state_changed;
    }
}

/// The update channels behind the SSE endpoints, one per watched scope. Two
/// requests for the same scope watch the same paths, so the paths identify it.
#[derive(Default)]
pub struct Hub {
    scopes: Mutex<HashMap<Vec<PathBuf>, Watched>>,
}

struct Watched {
    updates: broadcast::Sender<Update>,
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
    pub fn subscribe(&self, target: &Target) -> broadcast::Receiver<Update> {
        let mut scopes = self.scopes.lock().expect("hub lock");
        let relevant = target.relevant();

        if let Some(watched) = scopes.get(&relevant) {
            return watched.updates.subscribe();
        }

        let (updates, receiver) = broadcast::channel(UPDATE_CAPACITY);
        let source = match watcher(target, updates.clone()) {
            Ok(watcher) => Source::Watcher { _watcher: watcher },
            Err(error) => {
                eprintln!(
                    "warning: no filesystem watcher for {} ({error}); polling every {}s instead",
                    describe(&relevant),
                    POLL_INTERVAL.as_secs()
                );
                Source::Polling {
                    _task: spawn_polling(target.clone(), updates.clone()),
                }
            }
        };

        scopes.insert(
            relevant,
            Watched {
                updates,
                _source: source,
            },
        );

        receiver
    }

    /// How many pages are subscribed to any of this server's scopes right now.
    ///
    /// Read off the broadcast senders the hub already keeps, rather than a second
    /// count maintained alongside them: a receiver dropped when a tab closes
    /// leaves this figure correct with nothing to remember to update.
    pub fn subscribers(&self) -> usize {
        self.scopes
            .lock()
            .expect("hub lock")
            .values()
            .map(|watched| watched.updates.receiver_count())
            .sum()
    }
}

/// Watch every one of `target.watched_dirs`, reporting only the events that
/// touch one of the scope's relevant paths.
fn watcher(
    target: &Target,
    updates: broadcast::Sender<Update>,
) -> Result<RecommendedWatcher, notify::Error> {
    let (signals, bursts) = mpsc::channel(1);
    let target = target.clone();
    let watched_dirs = target.watched_dirs.clone();
    let event_target = target.clone();
    let relevant = target.relevant();

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

        if let Some(update) = classify(&event_target, &event.paths) {
            // A queue that is already full carries this signal just as well.
            let _ = signals.try_send(update);
        }
    })?;

    for dir in &watched_dirs {
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
async fn coalesce(mut bursts: mpsc::Receiver<Update>, updates: broadcast::Sender<Update>) {
    // Ends when the scope's watcher is dropped and takes the sender with it.
    while let Some(mut update) = bursts.recv().await {
        tokio::time::sleep(SETTLE).await;
        while let Ok(next) = bursts.try_recv() {
            update.merge(next);
        }

        // Fails only when nobody is watching that scope's page right now.
        let _ = updates.send(update);
    }
}

/// Start polling `target`, taking the baseline fingerprint before the task is
/// spawned so a change made right after this call cannot be missed.
fn spawn_polling(target: Target, updates: broadcast::Sender<Update>) -> JoinHandle<()> {
    let baseline = fingerprint(&target.relevant());
    tokio::spawn(poll(target, baseline, updates))
}

async fn poll(target: Target, baseline: Fingerprint, updates: broadcast::Sender<Update>) {
    let mut previous = baseline;
    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    ticker.tick().await; // The first tick completes immediately.

    loop {
        ticker.tick().await;

        let current = fingerprint(&target.relevant());
        if current != previous {
            let changed = changed_paths(&previous, &current);
            previous = current;
            if let Some(update) = classify(&target, &changed) {
                let _ = updates.send(update);
            }
        }
    }
}

type Fingerprint = HashMap<PathBuf, (u64, Option<SystemTime>)>;

/// A stand-in for the content under `paths`: every file's size and modified
/// time. A file that vanishes mid-walk is simply absent, which is itself the
/// change the next comparison reports.
fn fingerprint(paths: &[PathBuf]) -> Fingerprint {
    let mut files = HashMap::new();
    for path in paths {
        collect(path, &mut files);
    }
    files
}

fn collect(path: &Path, files: &mut Fingerprint) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };

    if !metadata.is_dir() {
        files.insert(path.to_owned(), (metadata.len(), metadata.modified().ok()));
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect(&entry.path(), files);
    }
}

fn changed_paths(previous: &Fingerprint, current: &Fingerprint) -> Vec<PathBuf> {
    let paths: HashSet<PathBuf> = previous.keys().chain(current.keys()).cloned().collect();
    paths
        .into_iter()
        .filter(|path| previous.get(path) != current.get(path))
        .collect()
}

fn classify(target: &Target, paths: &[PathBuf]) -> Option<Update> {
    let artifacts_changed = paths.iter().any(|path| {
        target
            .artifacts
            .iter()
            .any(|relevant| path.starts_with(relevant))
    });
    let review_state_changed = paths.iter().any(|path| {
        target
            .review_state
            .iter()
            .any(|relevant| path.starts_with(relevant))
    });

    (artifacts_changed || review_state_changed).then_some(Update {
        artifacts_changed,
        review_state_changed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// A temporary directory and the canonical form of its path.
    ///
    /// `classify` compares paths by literal prefix, and macOS resolves the
    /// system temp directory under `/var/folders/…` while reporting filesystem
    /// events at `/private/var/…` — so a test that hands the watcher the
    /// symlinked path has every event discarded as outside the scope. Tests
    /// build their paths by joining onto the canonical root, rather than
    /// canonicalizing each subpath, because a scope's sidecar directory is
    /// legitimately absent until its first record is written.
    fn temp_root() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("temp dir");
        let root = dir.path().canonicalize().expect("canonicalize temp dir");
        (dir, root)
    }

    fn target_for(dir: &Path) -> Target {
        Target {
            watched_dirs: vec![dir.to_owned()],
            artifacts: vec![dir.to_owned()],
            review_state: Vec::new(),
        }
    }

    fn classified_target(root: &Path) -> Target {
        Target {
            watched_dirs: Vec::new(),
            artifacts: vec![root.join("artifacts")],
            review_state: vec![root.join("comments.jsonl")],
        }
    }

    #[test]
    fn an_artifact_write_is_reported_as_an_artifact_change() {
        let root = Path::new("/project");
        assert_eq!(
            classify(
                &classified_target(root),
                &[root.join("artifacts/proposal.md")],
            ),
            Some(Update {
                artifacts_changed: true,
                review_state_changed: false,
            })
        );
    }

    #[test]
    fn a_comment_write_is_reported_as_review_state_change() {
        let root = Path::new("/project");
        assert_eq!(
            classify(&classified_target(root), &[root.join("comments.jsonl")]),
            Some(Update {
                artifacts_changed: false,
                review_state_changed: true,
            })
        );
    }

    #[test]
    fn writes_touching_both_are_reported_as_both() {
        let root = Path::new("/project");
        assert_eq!(
            classify(
                &classified_target(root),
                &[
                    root.join("artifacts/proposal.md"),
                    root.join("comments.jsonl"),
                ],
            ),
            Some(Update::all())
        );
    }

    #[tokio::test]
    async fn a_burst_is_one_merged_update() {
        let (signals, bursts) = mpsc::channel(2);
        let (updates, mut receiver) = broadcast::channel(UPDATE_CAPACITY);
        let task = tokio::spawn(coalesce(bursts, updates));

        signals
            .send(Update {
                artifacts_changed: true,
                review_state_changed: false,
            })
            .await
            .expect("send artifact update");
        signals
            .send(Update {
                artifacts_changed: false,
                review_state_changed: true,
            })
            .await
            .expect("send review update");
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(1), receiver.recv())
                .await
                .expect("coalescing timed out")
                .expect("coalescing closed"),
            Update::all()
        );
        assert!(
            tokio::time::timeout(
                Duration::from_millis(SETTLE.as_millis() as u64),
                receiver.recv()
            )
            .await
            .is_err(),
            "one burst produced multiple updates"
        );
        task.abort();
    }

    #[tokio::test]
    async fn a_write_under_the_watched_path_is_one_update() {
        let (_dir, root) = temp_root();
        let hub = Hub::default();
        let mut updates = hub.subscribe(&target_for(&root));

        fs::write(root.join("a.md"), "one").expect("write file");

        let update = tokio::time::timeout(Duration::from_secs(10), updates.recv()).await;
        assert!(update.is_ok(), "the written file was never reported");
    }

    /// The server reads the very directories it watches, so a read counted as a
    /// change would leave every open page reloading itself in a loop.
    #[tokio::test]
    async fn reading_a_watched_file_is_not_an_update() {
        let (_dir, root) = temp_root();
        let file = root.join("a.md");
        fs::write(&file, "one").expect("write file");
        let hub = Hub::default();
        let mut updates = hub.subscribe(&target_for(&root));

        fs::read_to_string(&file).expect("read file");
        fs::read_dir(&root).expect("read dir").count();

        let update = tokio::time::timeout(Duration::from_secs(2), updates.recv()).await;
        assert!(update.is_err(), "reading was reported as a change");
    }

    /// The fallback has to stand on its own: it is the only update source when
    /// the watcher backend is unavailable.
    #[tokio::test]
    async fn polling_reports_a_change_under_the_watched_path() {
        let (_dir, root) = temp_root();
        fs::write(root.join("a.md"), "one").expect("write file");
        let (updates, mut receiver) = broadcast::channel(UPDATE_CAPACITY);

        let polling = spawn_polling(target_for(&root), updates);
        fs::write(root.join("a.md"), "one changed").expect("rewrite file");

        let update = tokio::time::timeout(Duration::from_secs(10), receiver.recv()).await;
        polling.abort();

        assert!(update.is_ok(), "polling never reported the rewritten file");
    }

    /// A scope spans separate trees — a change's directory and the sidecars its
    /// comments live in — and a write in either one is that scope's update.
    #[tokio::test]
    async fn a_write_in_any_watched_tree_is_an_update() {
        let (_dir, root) = temp_root();
        let change = root.join("change");
        let sidecars = root.join("sidecars");
        fs::create_dir_all(&change).expect("create change dir");
        let hub = Hub::default();

        for written in [change.join("proposal.md"), sidecars.join("comments.jsonl")] {
            let mut updates = hub.subscribe(&Target {
                watched_dirs: vec![change.clone(), sidecars.clone()],
                artifacts: vec![change.clone()],
                review_state: vec![sidecars.clone()],
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
        let (_dir, root) = temp_root();
        let sidecars = root.join("comments/_session");
        let hub = Hub::default();

        let mut updates = hub.subscribe(&Target {
            watched_dirs: vec![sidecars.clone()],
            artifacts: Vec::new(),
            review_state: vec![sidecars.clone()],
        });
        fs::write(sidecars.join("session-a.jsonl"), "{}").expect("write sidecar");

        let update = tokio::time::timeout(Duration::from_secs(10), updates.recv()).await;
        assert!(update.is_ok(), "the first sidecar write was never reported");
    }
}
