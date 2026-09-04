//! Which port a project's dashboard belongs on, and how its URL is spelled.
//!
//! A machine-global table maps canonical project root to a port in [`RANGE`],
//! assigned in ascending order the first time a root is seen, so a project's URL
//! is the same today as it was last week regardless of which checkouts started
//! first. That is what lets the URL be stated before the server exists.
//!
//! **The table is a hint, never an authority.** It says where a dashboard for a
//! root should prefer to live, never whether one is running there. Every caller
//! verifies it against the network before acting on it, which is why losing the
//! table costs stability and never correctness.
//!
//! It lives outside the project deliberately: ports are global to the machine,
//! while `.openspec-doc/` is per-project and routinely deleted — by
//! `git clean -xdf`, by `rm -rf .openspec-doc`, and by this project's own
//! verification.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// The ports a project's dashboard may be assigned.
///
/// Bounded because a dashboard is enumerated by probing the whole range, and an
/// unbounded range cannot be enumerated. Thirty-two rather than ten because an
/// assignment is consumed by every project *ever opened*, not by every project
/// running, and a working machine passes ten checkouts inside a year.
pub const RANGE: RangeInclusive<u16> = 4321..=4352;

/// Overrides the directory the registry lives in.
///
/// This is a correctness requirement rather than a convenience: without it the
/// test suite writes real assignments into the developer's own registry, and two
/// concurrent `cargo test` runs corrupt each other's.
pub const STATE_DIR_ENV: &str = "OPENSPEC_DOC_STATE_DIR";

/// The route a dashboard reports its identity on.
///
/// Off the page namespace, so it cannot collide with a change or session name.
pub const IDENTITY_PATH: &str = "/api/identity";

/// The route a dashboard is asked to stop on.
///
/// Beside [`IDENTITY_PATH`] for the same reason that one is here: the server
/// routes it and the CLI writes the request bytes by hand, so the two have to
/// agree about the spelling and neither owns it.
pub const SHUTDOWN_PATH: &str = "/api/shutdown";

/// The header a shutdown request names the root it believes it is stopping in.
///
/// One header doing two jobs the design asks for separately.
///
/// It names the root, so the server — the only participant that knows for
/// certain what it is — can refuse a request meant for a dashboard that has
/// since been replaced on this port.
///
/// And it is a header no cross-origin HTML form can set, so a page in the
/// operator's own browser cannot POST a dashboard down on a guessable local port
/// without first clearing a CORS preflight it will not pass. No token and no
/// nonce: a secret needs somewhere to live, and per-server state is the design
/// this work rejected.
///
/// A header rather than a JSON body because the CLI writes these bytes by hand,
/// and a header value carries a path with no body, no length, and no parser.
pub const SHUTDOWN_ROOT_HEADER: &str = "x-openspec-doc-root";

const APP_DIR: &str = "openspec-doc";
const REGISTRY_FILE: &str = "ports.json";

/// What a dashboard reports about itself: which project it serves, and the
/// process serving it.
///
/// Lives here rather than beside either the route or the probe because the two
/// have to agree about it, and because the probe is in a crate with no JSON
/// parser of its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// The canonical project root this dashboard serves.
    pub root: PathBuf,
    pub pid: u32,
}

/// The identity in an HTTP `response`, or `None` when what answered is not a
/// dashboard.
///
/// Deliberately incurious about everything else in the response. A probe asks one
/// question — is a dashboard answering here — and anything that is not an
/// identity is a no, whatever its status line said.
pub fn identity_in(response: &str) -> Option<Identity> {
    let body = response.split_once("\r\n\r\n")?.1;

    serde_json::from_str(body).ok()
}

/// What the registry holds: canonical root to port, and nothing else.
///
/// The next free port is *derived* from these values rather than stored as a
/// counter, because a stored counter is a second source of truth that drifts
/// from the table it describes.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Registry {
    ports: BTreeMap<String, u16>,
}

/// The dashboard's URL on `host` and `port`.
///
/// One spelling, used by the server's startup print, `serve url`, the
/// exploration hook, and the browser-opening step. Four spellings of
/// `http://{host}:{port}` is four chances to disagree about the trailing slash.
pub fn url(host: &str, port: u16) -> String {
    format!("http://{host}:{port}")
}

/// The URL of `session_id`'s review page on the dashboard at `host` and `port`.
pub fn session_page(host: &str, port: u16, session_id: &str) -> String {
    format!("{}/sessions/{session_id}", url(host, port))
}

/// Where the registry lives: the platform state directory where there is one,
/// and the local data directory otherwise.
///
/// `dirs` rather than a hand-rolled `cfg` tree. The rule being applied is not
/// *never add a dependency*, it is that a dependency must be smaller than the
/// correctness risk of doing it by hand — and hand-rolling three platforms'
/// directory conventions is how a tool ends up writing to
/// `~/Library/Application Support` on Linux, invisibly, because the author
/// develops on one platform. `dirs::state_dir` is `None` on macOS and Windows,
/// which have no state-directory convention, so the local data directory stands
/// in there.
pub fn registry_path() -> Result<PathBuf, Error> {
    if let Some(dir) = std::env::var_os(STATE_DIR_ENV) {
        return Ok(PathBuf::from(dir).join(REGISTRY_FILE));
    }

    let dir = dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .ok_or(Error::NoStateDir)?;

    Ok(dir.join(APP_DIR).join(REGISTRY_FILE))
}

/// The port assigned to `root`, assigning the lowest unassigned port in
/// [`RANGE`] the first time a root is seen.
///
/// `root` must already be canonical — [`crate::Project`] canonicalizes it — so
/// the key is exact rather than heuristic, and a project that moved on disk is a
/// new project with a new number.
pub fn assign(root: &Path) -> Result<u16, Error> {
    let path = registry_path()?;
    let mut registry = load(&path)?;
    let key = key(root)?;

    if let Some(&port) = registry.ports.get(&key) {
        return Ok(port);
    }

    let port = match unassigned(&registry) {
        Some(port) => port,
        // Reclamation is lazy: eager collection churns assignments for no
        // benefit, and an existing assignment stays put while any port remains.
        None => {
            reclaim(&mut registry).ok_or_else(|| Error::PortRangeFull { path: path.clone() })?
        }
    };

    registry.ports.insert(key, port);
    store(&path, &registry)?;

    Ok(port)
}

/// The port assigned to `root`, or `None` when the root has never been seen.
/// Assigns nothing and writes nothing.
pub fn assigned(root: &Path) -> Result<Option<u16>, Error> {
    let registry = load(&registry_path()?)?;

    Ok(registry.ports.get(&key(root)?).copied())
}

/// Record `port` as `root`'s assignment, replacing whatever was there.
///
/// This is the repair path. A caller that finds the dashboard on a port other
/// than the assignment, or that falls forward off a squatted one, writes back
/// what it learned so the project is stable there from then on.
pub fn record(root: &Path, port: u16) -> Result<(), Error> {
    let path = registry_path()?;
    let mut registry = load(&path)?;
    registry.ports.insert(key(root)?, port);

    store(&path, &registry)
}

/// Every assignment the registry holds, canonical root to port.
///
/// Assignments and nothing else: an entry says where a root's dashboard belongs,
/// never that one is running there. A caller reporting what is running asks the
/// network for that and joins this on top.
pub fn assignments() -> Result<BTreeMap<PathBuf, u16>, Error> {
    let registry = load(&registry_path()?)?;

    Ok(registry
        .ports
        .into_iter()
        .map(|(root, port)| (PathBuf::from(root), port))
        .collect())
}

/// Drop `root`'s assignment, returning the port it held, or `None` when the root
/// had none.
///
/// The way out of a full range: an assignment is consumed by every root ever
/// opened, and lazy reclamation only frees the ones whose directory is gone.
pub fn forget(root: &Path) -> Result<Option<u16>, Error> {
    let path = registry_path()?;
    let mut registry = load(&path)?;

    let Some(port) = registry.ports.remove(&key(root)?) else {
        return Ok(None);
    };
    store(&path, &registry)?;

    Ok(Some(port))
}

/// The registry as it is on disk.
///
/// A missing file is normal and silent — every project is unseen once. A file
/// that will not parse is an error, so that nothing here rewrites or deletes a
/// file it could not read; callers report it and continue without an assignment.
fn load(path: &Path) -> Result<Registry, Error> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Registry::default()),
        Err(source) => return Err(Error::io(path, source)),
    };

    serde_json::from_str(&contents).map_err(|source| Error::PortRegistry {
        path: path.to_owned(),
        source,
    })
}

/// Replace the registry with `registry`: a temp file in the same directory,
/// renamed over the original, so an interrupted write cannot leave half a table
/// behind.
fn store(path: &Path, registry: &Registry) -> Result<(), Error> {
    let dir = path.parent().expect("the registry path has a parent");
    fs::create_dir_all(dir).map_err(|source| Error::write(dir, source))?;

    let contents =
        serde_json::to_string_pretty(registry).map_err(|source| Error::PortRegistry {
            path: path.to_owned(),
            source,
        })?;

    // Named for this process so two of them writing at once cannot rename each
    // other's half-written file into place.
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&temp, contents).map_err(|source| Error::write(&temp, source))?;

    fs::rename(&temp, path).map_err(|source| Error::Rename {
        from: temp,
        to: path.to_owned(),
        source,
    })
}

/// The lowest port in [`RANGE`] no root holds.
fn unassigned(registry: &Registry) -> Option<u16> {
    RANGE
        .clone()
        .find(|port| !registry.ports.values().any(|held| held == port))
}

/// Free the lowest-numbered assignment whose root no longer exists on disk, and
/// return the port it held.
///
/// "The directory is gone" is a fact any operator can check and reproduce.
/// Reclaiming the least recently used assignment instead would put a project's
/// port back at the mercy of the order things happened in, which is the property
/// this whole mechanism exists to remove.
///
/// One cost, stated rather than discovered: an unmounted drive makes a live
/// project look deleted, so its entry can be reclaimed. That costs stability and
/// never correctness, because a dashboard actually running for that root is
/// still found by the sweep.
fn reclaim(registry: &mut Registry) -> Option<u16> {
    let (gone, port) = registry
        .ports
        .iter()
        .filter(|(root, _)| !Path::new(root).exists())
        .min_by_key(|(_, port)| **port)
        .map(|(root, port)| (root.clone(), *port))?;

    registry.ports.remove(&gone);

    Some(port)
}

/// How a root is keyed in the registry.
fn key(root: &Path) -> Result<String, Error> {
    root.to_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::NonUtf8Path {
            path: root.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    use tempfile::TempDir;

    /// The state-directory override is process-wide, so registry tests take a
    /// turn each rather than racing over one environment variable.
    static ENV: Mutex<()> = Mutex::new(());

    /// A state directory nothing else is writing to, plus the roots to key on.
    struct Fixture {
        _guard: MutexGuard<'static, ()>,
        state: TempDir,
        roots: TempDir,
    }

    fn fixture() -> Fixture {
        let guard = ENV.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let state = TempDir::new().expect("state dir");
        // SAFETY: `ENV` makes this the only test touching the variable.
        unsafe { std::env::set_var(STATE_DIR_ENV, state.path()) };

        Fixture {
            _guard: guard,
            state,
            roots: TempDir::new().expect("roots dir"),
        }
    }

    impl Fixture {
        /// A directory standing in for a canonical project root.
        fn root(&self, name: &str) -> PathBuf {
            let root = self.roots.path().join(name);
            fs::create_dir_all(&root).expect("create root");
            root
        }

        fn registry(&self) -> PathBuf {
            self.state.path().join(REGISTRY_FILE)
        }

        /// Fill the whole range, one entry per port, with roots that exist.
        fn fill_range(&self) {
            for (index, port) in RANGE.clone().enumerate() {
                assert_eq!(
                    assign(&self.root(&format!("filler-{index}"))).expect("assign filler"),
                    port
                );
            }
        }
    }

    #[test]
    fn a_newly_seen_root_takes_the_lowest_free_port() {
        let fixture = fixture();

        assert_eq!(assign(&fixture.root("a")).expect("assign"), 4321);
        assert_eq!(assign(&fixture.root("b")).expect("assign"), 4322);
    }

    #[test]
    fn the_same_root_keeps_its_port_across_other_projects_being_seen() {
        let fixture = fixture();
        let first = fixture.root("a");

        let port = assign(&first).expect("assign");
        assign(&fixture.root("b")).expect("assign");
        assign(&fixture.root("c")).expect("assign");

        assert_eq!(assign(&first).expect("reassign"), port);
        assert_eq!(assigned(&first).expect("read back"), Some(port));
    }

    /// The override is what keeps the suite out of the developer's own registry.
    #[test]
    fn the_state_directory_override_is_honoured() {
        let fixture = fixture();

        assign(&fixture.root("a")).expect("assign");

        assert!(fixture.registry().is_file(), "wrote outside the override");
        assert_eq!(
            registry_path().expect("registry path"),
            fixture.registry(),
            "the override must not be composed with the platform tree"
        );
    }

    #[test]
    fn an_unseen_root_has_no_assignment_and_reading_it_writes_nothing() {
        let fixture = fixture();

        assert_eq!(assigned(&fixture.root("a")).expect("read"), None);
        assert!(!fixture.registry().exists(), "a read created the registry");
    }

    /// A vanished root keeps its port while any port is free: reclamation is for
    /// a full range, and churning assignments early is what makes ports move.
    #[test]
    fn assignments_hold_while_ports_remain_including_for_roots_that_are_gone() {
        let fixture = fixture();
        let gone = fixture.root("gone");
        let port = assign(&gone).expect("assign");
        fs::remove_dir_all(&gone).expect("remove root");

        assert_eq!(assign(&fixture.root("new")).expect("assign"), port + 1);
        assert_eq!(
            assigned(&gone).expect("read back"),
            Some(port),
            "a vanished root's entry was collected while ports were still free"
        );
    }

    #[test]
    fn a_full_range_reclaims_a_vanished_root() {
        let fixture = fixture();
        let gone = fixture.root("gone");
        let port = assign(&gone).expect("assign");
        for index in 0..(RANGE.clone().count() - 1) {
            assign(&fixture.root(&format!("filler-{index}"))).expect("assign filler");
        }
        fs::remove_dir_all(&gone).expect("remove root");

        assert_eq!(assign(&fixture.root("new")).expect("assign"), port);
        assert_eq!(
            assigned(&gone).expect("read back"),
            None,
            "the reclaimed entry must be gone, not duplicated"
        );
    }

    #[test]
    fn a_full_range_of_live_roots_fails_naming_the_registry() {
        let fixture = fixture();
        fixture.fill_range();

        let error = assign(&fixture.root("new")).expect_err("exhausted range");

        assert!(matches!(error, Error::PortRangeFull { .. }), "{error:?}");
        let message = error.to_string();
        assert!(
            message.contains(&fixture.registry().display().to_string()),
            "the error must name the registry: {message}"
        );
        assert!(
            message.contains("serve forget"),
            "the error must name how to forget an entry: {message}"
        );
    }

    /// Loud and non-fatal: the error names the file so the operator can delete
    /// it, and the file itself is left exactly as it was.
    #[test]
    fn a_corrupt_registry_is_reported_and_left_alone() {
        let fixture = fixture();
        fs::write(fixture.registry(), "{ not json").expect("write corrupt registry");

        let error = assign(&fixture.root("a")).expect_err("corrupt registry");

        assert!(matches!(error, Error::PortRegistry { .. }), "{error:?}");
        assert!(
            error
                .to_string()
                .contains(&fixture.registry().display().to_string()),
            "{error}"
        );
        assert_eq!(
            fs::read_to_string(fixture.registry()).expect("read back"),
            "{ not json",
            "a file that would not parse must not be rewritten"
        );
    }

    /// The way out of a full range: the freed port is the one a newly seen root
    /// then takes.
    #[test]
    fn forgetting_a_root_frees_its_port_for_a_newly_seen_one() {
        let fixture = fixture();
        let forgotten = fixture.root("forgotten");
        let port = assign(&forgotten).expect("assign");
        for index in 0..(RANGE.clone().count() - 1) {
            assign(&fixture.root(&format!("filler-{index}"))).expect("assign filler");
        }

        assert_eq!(forget(&forgotten).expect("forget"), Some(port));

        assert_eq!(assigned(&forgotten).expect("read back"), None);
        assert_eq!(assign(&fixture.root("new")).expect("assign"), port);
    }

    #[test]
    fn forgetting_an_unassigned_root_reports_that_there_was_nothing_to_forget() {
        let fixture = fixture();

        assert_eq!(forget(&fixture.root("unseen")).expect("forget"), None);
        assert!(
            !fixture.registry().exists(),
            "forgetting nothing created the registry"
        );
    }

    #[test]
    fn every_assignment_is_readable_at_once() {
        let fixture = fixture();
        let first = fixture.root("a");
        let second = fixture.root("b");
        assign(&first).expect("assign");
        assign(&second).expect("assign");

        assert_eq!(
            assignments().expect("assignments"),
            BTreeMap::from([(first, 4321), (second, 4322)])
        );
    }

    #[test]
    fn a_recorded_port_replaces_the_assignment() {
        let fixture = fixture();
        let root = fixture.root("a");
        assign(&root).expect("assign");

        record(&root, 4330).expect("record");

        assert_eq!(assigned(&root).expect("read back"), Some(4330));
    }

    /// The registry holds a mapping and nothing else, so a hand-edit or a
    /// version written by another build reads back without a migration.
    #[test]
    fn the_registry_holds_only_roots_and_ports() {
        let fixture = fixture();
        let root = fixture.root("a");

        assign(&root).expect("assign");

        let stored: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(fixture.registry()).expect("read"))
                .expect("parse");
        assert_eq!(
            stored,
            serde_json::json!({ "ports": { root.to_str().unwrap(): 4321 } })
        );
    }

    #[test]
    fn the_url_is_spelled_one_way() {
        assert_eq!(url("127.0.0.1", 4321), "http://127.0.0.1:4321");
    }

    #[test]
    fn an_identity_response_reads_back_as_the_root_it_names() {
        let identity = identity_in(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n\
             {\"root\":\"/repos/a\",\"pid\":41}",
        )
        .expect("identity");

        assert_eq!(identity.root, Path::new("/repos/a"));
        assert_eq!(identity.pid, 41);
    }

    /// Something else listening on a probed port is a no, not a dashboard.
    #[test]
    fn a_response_that_is_not_an_identity_is_not_one() {
        for response in [
            "",
            "HTTP/1.1 200 OK\r\n\r\n{\"unrelated\":true}",
            "HTTP/1.1 404 Not Found\r\n\r\nnot json",
            "SSH-2.0-OpenSSH_9.0",
        ] {
            assert_eq!(identity_in(response), None, "accepted {response:?}");
        }
    }
}
