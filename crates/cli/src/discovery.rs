//! Finding the dashboard that serves a project, starting one when none does, and
//! asking one to stop.
//!
//! Two questions look like one and are not:
//!
//! | Question | Answered by |
//! | --- | --- |
//! | Is a dashboard already serving this root? | the network, always |
//! | Which port should this project's dashboard prefer? | a recorded hint |
//!
//! The first is asked of the network and never read off a disk. The obvious
//! alternative — a `.openspec-doc/serve.json` recording host, port and pid — is
//! rejected: it has to be gitignored, so `git clean -xdf` and `rm -rf
//! .openspec-doc` delete it while its server keeps running, and every such
//! deletion produces another dashboard that is alive, undiscoverable, and
//! unkillable by anything this project could ship. Probing survives file
//! deletion, reboot, and `SIGKILL`.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use openspec_doc_core::Project;
use openspec_doc_core::dashboard::{self, Identity};

use crate::error::Error;

/// The interface a dashboard is reached on. The port range is localhost's, and
/// the registry records a port because a host is not what varies.
pub const HOST: &str = "127.0.0.1";

/// A dashboard's output, kept so a start that fails can explain itself.
const LOG_FILE: &str = ".openspec-doc/serve.log";

/// Short enough that a hung port cannot stall a hook: a probe runs inside a
/// turn boundary, up to thirty-two times.
const CONNECT_TIMEOUT: Duration = Duration::from_millis(150);
const READ_TIMEOUT: Duration = Duration::from_millis(250);

/// How long a freshly started dashboard has to answer, and how often it is asked.
const START_TIMEOUT: Duration = Duration::from_secs(2);
const START_POLL: Duration = Duration::from_millis(50);

/// How long a dashboard asked to stop has to stop answering, and how often it is
/// asked. A graceful shutdown answers the request and closes the listener after,
/// so the instant the request returns is exactly too early to ask.
const STOP_TIMEOUT: Duration = Duration::from_secs(2);
const STOP_POLL: Duration = Duration::from_millis(25);

/// A response longer than this is not an identity, and reading it is not this
/// probe's job.
const MAX_RESPONSE: u64 = 8 * 1024;

/// The port this project's dashboard prefers, assigning one the first time the
/// root is seen.
///
/// A registry that cannot be read is reported and continued past with no
/// assignment, following the precedent `report_promotion` sets: a session must
/// not lose its review feedback to an unrelated failure. The caller then falls
/// through to the sweep, which is what finds a dashboard whether or not anything
/// remembered where it should be.
pub fn preferred(project: &Project) -> Option<u16> {
    match dashboard::assign(&project.root) {
        Ok(port) => Some(port),
        Err(error) => {
            crate::report(
                "could not read this project's assigned port; continuing without an assignment",
            );
            crate::eprint_chain(&error);
            None
        }
    }
}

/// The port a dashboard for `root` is serving on, or `None` when none is.
pub fn find(root: &Path, assigned: Option<u16>) -> Option<u16> {
    find_in(root, assigned, &range())
}

/// Every dashboard answering in the port range, ascending by port.
///
/// The same sweep [`find`] filters, with nothing thrown away. A dashboard on a
/// port outside the range does not answer here and cannot: the range is what
/// makes the sweep bounded, and a scan of all 65535 ports is not the trade this
/// makes. A caller showing this to a person names the range it covered.
pub fn enumerate() -> Vec<(u16, Identity)> {
    enumerate_in(&range())
}

/// What asking the dashboard on a port to stop actually achieved.
///
/// Every variant is read off a probe, never off the answer to the request. A
/// delivered request is not a stopped dashboard, and the second one is what a
/// caller was asking about.
#[derive(Debug, PartialEq, Eq)]
pub enum Stopped {
    /// Nothing serves the port any more.
    Yes,
    /// Nothing was listening when the request was sent, so it had already
    /// exited — on its own idle deadline, or under somebody else, between the
    /// enumeration and the request. Not the same thing as having been stopped,
    /// and reporting it as one claims credit for the idle deadline's work.
    AlreadyGone,
    /// It answered the request and kept serving. The wedged dashboard this
    /// deliberately does not escalate for, so the one thing owed is saying so.
    StillServing,
    /// Refused, so nothing was stopped, beside whatever answers there now.
    ///
    /// The expected cause is the port having changed hands between the
    /// enumeration and the request: the dashboard that was found there exited,
    /// and another project's fell forward onto its port. The server is the only
    /// participant that could have noticed, which is the second thing asking
    /// over HTTP buys over sending a signal.
    Refused {
        status: u16,
        serving: Option<PathBuf>,
    },
}

/// Ask the dashboard serving `root` on `port` to stop, and report what the port
/// does afterwards.
///
/// The answer to the request refines the report but is never the whole of it. A
/// refusal means nothing was asked to stop, which is worth distinguishing from a
/// request that was accepted and ignored — but whether a dashboard is *gone* is
/// decided by probing for it, every time.
pub fn stop(port: u16, root: &Path) -> Stopped {
    match ask_to_stop(port, root) {
        Answer::Accepted => {
            if gone(port, root) {
                Stopped::Yes
            } else {
                Stopped::StillServing
            }
        }
        Answer::Refused(status) => Stopped::Refused {
            status,
            serving: probe(port).map(|identity| identity.root),
        },
        Answer::Unanswered => Stopped::AlreadyGone,
    }
}

/// Make sure a dashboard is serving `project`, starting one if none is, and
/// return the port it is serving on.
pub fn ensure(project: &Project) -> Result<u16, Error> {
    ensure_in(project, &range())
}

/// The ports a sweep covers. Taken as an argument so a test can hand in ports the
/// operating system says are free: the real range is where the developer's own
/// dashboards are sitting, and a test that assumes it is empty fails on the one
/// machine that matters.
pub(crate) fn range() -> Vec<u16> {
    dashboard::RANGE.collect()
}

/// As [`find`], over `range`.
///
/// `assigned` is asked on its own first because it is nearly always right, so the
/// common case costs one probe rather than a sweep. The sweep behind it is
/// [`enumerate_in`]'s, filtered: anything answering with another root, or with
/// something that is not a dashboard, is left alone.
fn find_in(root: &Path, assigned: Option<u16>, range: &[u16]) -> Option<u16> {
    if let Some(port) = assigned
        && probe(port).is_some_and(|identity| identity.root == root)
    {
        return Some(port);
    }

    enumerate_in(range)
        .into_iter()
        .find(|(_, identity)| identity.root == root)
        .map(|(port, _)| port)
}

/// As [`enumerate`], over `range`.
///
/// A thread per port. Sequentially the worst case is a port that accepts a
/// connection and never answers — `CONNECT_TIMEOUT + READ_TIMEOUT` each,
/// thirty-two of them, twelve seconds of an interactive command doing nothing. That
/// argument is `enumerate`'s: `find` short-circuits on the assignment and rarely
/// gets here at all, and gets the concurrency because the sweep is shared rather
/// than because it needed it.
pub(crate) fn enumerate_in(range: &[u16]) -> Vec<(u16, Identity)> {
    let mut found: Vec<(u16, Identity)> = std::thread::scope(|scope| {
        let probes: Vec<_> = range
            .iter()
            .map(|&port| scope.spawn(move || probe(port).map(|identity| (port, identity))))
            .collect();

        probes
            .into_iter()
            .filter_map(|probe| probe.join().expect("a probe thread panicked"))
            .collect()
    });
    found.sort_by_key(|(port, _)| *port);

    found
}

/// As [`ensure`], over `range`.
fn ensure_in(project: &Project, range: &[u16]) -> Result<u16, Error> {
    let assigned = preferred(project);

    // Never bind on the strength of the registry alone. The whole range is swept
    // for this root before anything is started, and that sweep is what makes the
    // registry safe to lose: delete it, corrupt it, restore an old copy, and the
    // running dashboard is still found and still reused. An implementation that
    // trusts the entry and skips the sweep has rebuilt the rejected
    // `serve.json` design with extra steps, and fails the same way — one deleted
    // file becomes one duplicate server, and repeated deletions a fleet of them.
    if let Some(port) = find_in(&project.root, assigned, range) {
        if Some(port) != assigned {
            // Found somewhere other than its assignment: repair the entry, or the
            // project's URL stays wrong until the dashboard next restarts.
            dashboard::record(&project.root, port)?;
        }

        return Ok(port);
    }

    start(project, assigned, range)
}

/// Start a detached dashboard for `project` and wait for one to answer.
fn start(project: &Project, assigned: Option<u16>, range: &[u16]) -> Result<u16, Error> {
    let log_path = project.root.join(LOG_FILE);
    let log = open_log(&log_path)?;
    let executable = std::env::current_exe().map_err(|source| Error::CurrentExe { source })?;

    let mut command = Command::new(executable);
    command
        .arg("--root")
        .arg(&project.root)
        .args(["serve", "--host", HOST, "--no-open", "--idle-exit"])
        .stdin(Stdio::null())
        // Not `/dev/null`: a server that cannot bind would then be a start that
        // appears to succeed, a poll that times out, and no way to learn why —
        // and the next turn boundary would repeat it identically, forever.
        .stdout(log.try_clone().map_err(|source| Error::Log {
            path: log_path.clone(),
            source,
        })?)
        .stderr(log);
    #[cfg(windows)]
    prevent_stdio_inheritance().map_err(|source| Error::Spawn { source })?;
    detach(&mut command);

    command.spawn().map_err(|source| Error::Spawn { source })?;

    // Poll for *a dashboard for this root*, not for our own child. Two sessions
    // ending at the same instant both start one; one loses the bind and exits
    // into its own log, and the survivor serves both. Checking that a dashboard
    // is there makes the loser of that race a non-event instead of an error path.
    let deadline = Instant::now() + START_TIMEOUT;
    loop {
        if let Some(port) = find_in(&project.root, assigned, range) {
            if Some(port) != assigned {
                dashboard::record(&project.root, port)?;
            }

            return Ok(port);
        }

        if Instant::now() >= deadline {
            return Err(Error::DashboardDidNotStart {
                seconds: START_TIMEOUT.as_secs(),
                log: log_path,
            });
        }

        std::thread::sleep(START_POLL);
    }
}

fn open_log(path: &Path) -> Result<std::fs::File, Error> {
    let dir = path.parent().expect("the log path has a parent");
    std::fs::create_dir_all(dir).map_err(|source| Error::Log {
        path: dir.to_owned(),
        source,
    })?;

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| Error::Log {
            path: path.to_owned(),
            source,
        })
}

/// Put the dashboard outside this process's process group.
///
/// An agent kills a hook that overruns its timeout, and a child in the hook's
/// process group is killed with it. This is the difference between a dashboard
/// that works perfectly when tested by hand and one that survives a real hook
/// invocation.
#[cfg(unix)]
fn detach(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    // In std, so no `libc` dependency for `setsid`.
    command.process_group(0);
}

/// Prevent a detached dashboard from keeping a hook runner's pipes open.
///
/// `Command` inherits every inheritable Windows handle, not only its configured
/// stdio handles. A hook runner captures its output, so the dashboard's copies
/// would keep the hook's stdout and stderr open until the dashboard exits.
#[cfg(windows)]
fn prevent_stdio_inheritance() -> std::io::Result<()> {
    use std::os::windows::io::RawHandle;

    const HANDLE_FLAG_INHERIT: u32 = 0x0000_0001;
    const INVALID_HANDLE_VALUE: RawHandle = -1isize as RawHandle;
    const STD_INPUT_HANDLE: u32 = (-10i32) as u32;
    const STD_OUTPUT_HANDLE: u32 = (-11i32) as u32;
    const STD_ERROR_HANDLE: u32 = (-12i32) as u32;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetHandleInformation(handle: RawHandle, flags: *mut u32) -> i32;
        fn GetStdHandle(which: u32) -> RawHandle;
        fn SetHandleInformation(handle: RawHandle, mask: u32, flags: u32) -> i32;
    }

    for which in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        let handle = unsafe { GetStdHandle(which) };
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            continue;
        }

        let mut flags = 0;
        if unsafe { GetHandleInformation(handle, &mut flags) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        if flags & HANDLE_FLAG_INHERIT != 0
            && unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0
        {
            return Err(std::io::Error::last_os_error());
        }
    }

    Ok(())
}

#[cfg(windows)]
fn detach(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x0000_0008;

    command.creation_flags(DETACHED_PROCESS);
}

/// Ask `port` what it is.
///
/// A fixed `GET` over `TcpStream`, hand-rolled on purpose. The CLI has no HTTP
/// client, and one request to one known route on localhost does not justify
/// adding one — nor should the `dirs` dependency the port registry needed be read
/// as licence to add more. Do not "fix" this into an HTTP client.
fn probe(port: u16) -> Option<Identity> {
    let mut stream = connected(port)?;

    // HTTP/1.0 so the server closes the connection after answering, which is what
    // ends the read on the normal path.
    stream
        .write_all(
            format!(
                "GET {} HTTP/1.0\r\nHost: {HOST}\r\n\r\n",
                dashboard::IDENTITY_PATH
            )
            .as_bytes(),
        )
        .ok()?;

    // The read outcome is deliberately ignored. A port that accepts a connection
    // and never answers is ended by the read timeout, and the only question this
    // asks is whether an identity arrived — not whether the socket closed
    // tidily.
    let mut response = Vec::new();
    let _ = stream.take(MAX_RESPONSE).read_to_end(&mut response);

    dashboard::identity_in(&String::from_utf8_lossy(&response))
}

/// How a dashboard answered a request to stop.
enum Answer {
    /// It took the request. Whether it acted on it is a separate question, and
    /// only a probe answers that one.
    Accepted,
    /// It refused, naming why with a status. Nothing was stopped.
    Refused(u16),
    /// Nothing answered: no listener, or a listener that is not a dashboard.
    Unanswered,
}

/// Ask `port` to stop, naming the `root` the caller believes it is stopping.
///
/// A fixed `POST` over `TcpStream`, beside the hand-rolled `GET` in [`probe`]
/// and for the same reason: the CLI has no HTTP client, and two requests to two
/// known routes on localhost do not justify adding one. Do not "fix" this into
/// an HTTP client either.
fn ask_to_stop(port: u16, root: &Path) -> Answer {
    let Some(mut stream) = connected(port) else {
        return Answer::Unanswered;
    };

    // No body, so no length worth computing and no encoder: the root travels in
    // the header, which is also the header a cross-origin form cannot set.
    let request = format!(
        "POST {} HTTP/1.0\r\nHost: {HOST}\r\n{}: {}\r\nContent-Length: 0\r\n\r\n",
        dashboard::SHUTDOWN_PATH,
        dashboard::SHUTDOWN_ROOT_HEADER,
        root.display(),
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return Answer::Unanswered;
    }

    let mut response = Vec::new();
    let _ = stream.take(MAX_RESPONSE).read_to_end(&mut response);

    match status_in(&String::from_utf8_lossy(&response)) {
        Some(status) if (200..300).contains(&status) => Answer::Accepted,
        Some(status) => Answer::Refused(status),
        None => Answer::Unanswered,
    }
}

/// Whether `root`'s dashboard has stopped answering on `port`.
///
/// Polled rather than asked once, because a graceful shutdown answers first and
/// closes its listener afterwards. A port answering for somebody else counts as
/// gone: the dashboard that was asked to stop is not there.
fn gone(port: u16, root: &Path) -> bool {
    let deadline = Instant::now() + STOP_TIMEOUT;
    loop {
        if probe(port).is_none_or(|identity| identity.root != root) {
            return true;
        }

        if Instant::now() >= deadline {
            return false;
        }

        std::thread::sleep(STOP_POLL);
    }
}

/// The status code in an HTTP `response`, or `None` when what answered was not
/// HTTP at all.
fn status_in(response: &str) -> Option<u16> {
    let mut status_line = response.lines().next()?.split_whitespace();
    status_line.next()?.starts_with("HTTP/").then_some(())?;

    status_line.next()?.parse().ok()
}

/// A connection to `port`, with both timeouts set.
///
/// Short on purpose, and the reason belongs to the probe: a hook runs inside a
/// turn boundary and sweeps up to thirty-two ports, so a hung port must not be
/// able to stall it.
fn connected(port: u16) -> Option<TcpStream> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let stream = TcpStream::connect_timeout(&address, CONNECT_TIMEOUT).ok()?;
    stream.set_read_timeout(Some(READ_TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(READ_TIMEOUT)).ok()?;

    Some(stream)
}

/// Where a failed start explains itself, for a caller reporting one.
pub fn log_path(project: &Project) -> PathBuf {
    project.root.join(LOG_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use std::net::TcpListener;
    use std::sync::mpsc;

    /// A listener on a port the operating system says is free.
    ///
    /// Tests take their ports from the OS rather than assuming the real range is
    /// free on the machine running the suite: the developer's own dashboards are
    /// sitting in it.
    fn listener() -> (TcpListener, u16) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("local addr").port();
        (listener, port)
    }

    /// Answer identity probes with `root` until the test is done.
    fn identity_server(root: &Path) -> u16 {
        let (listener, port) = listener();
        let body = serde_json::json!({
            "root": root.display().to_string(),
            "pid": std::process::id(),
        })
        .to_string();
        std::thread::spawn(move || {
            for connection in listener.incoming() {
                let Ok(mut stream) = connection else { break };
                let mut request = String::new();
                let _ = std::io::BufReader::new(&stream).read_line(&mut request);
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
            }
        });
        port
    }

    /// A state directory and a project root nothing else is writing to.
    ///
    /// The override is process-wide, so the tests that touch the registry take a
    /// turn each rather than racing over one environment variable.
    struct Fixture {
        _guard: std::sync::MutexGuard<'static, ()>,
        _state: tempfile::TempDir,
        _root: tempfile::TempDir,
        project: Project,
    }

    fn fixture() -> Fixture {
        static ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());

        let guard = ENV.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let state = tempfile::TempDir::new().expect("state dir");
        // SAFETY: `ENV` makes this the only test touching the variable.
        unsafe { std::env::set_var(dashboard::STATE_DIR_ENV, state.path()) };

        let root = tempfile::TempDir::new().expect("project root");
        std::fs::create_dir_all(root.path().join("openspec")).expect("create openspec dir");
        std::fs::write(root.path().join("openspec/config.yaml"), "").expect("write config");
        let project = openspec_doc_core::project_at(root.path()).expect("project");

        Fixture {
            _guard: guard,
            _state: state,
            _root: root,
            project,
        }
    }

    #[test]
    fn a_dashboard_for_this_root_is_found_on_its_assignment() {
        let root = PathBuf::from("/repos/a");
        let port = identity_server(&root);

        assert_eq!(find(&root, Some(port)), Some(port));
    }

    /// A dashboard serving somebody else is left alone.
    #[test]
    fn a_foreign_root_is_not_mistaken_for_this_one() {
        let port = identity_server(Path::new("/repos/other"));

        assert_eq!(find(Path::new("/repos/a"), Some(port)), None);
    }

    #[test]
    fn a_listener_that_is_not_a_dashboard_is_skipped() {
        let (_listener, port) = listener();

        assert_eq!(find(Path::new("/repos/a"), Some(port)), None);
    }

    /// A hook runs inside a turn boundary, so a port that accepts and never
    /// answers has to end the probe rather than hold it.
    #[test]
    fn a_port_that_never_responds_times_out_rather_than_hanging() {
        let (listener, port) = listener();
        let (accepted, held) = mpsc::channel();
        std::thread::spawn(move || {
            let connection = listener.accept().expect("accept");
            accepted.send(()).expect("report the accept");
            // Held open, answering nothing, until the test is done with it.
            std::thread::sleep(Duration::from_secs(5));
            drop(connection);
        });

        let started = Instant::now();
        let found = find(Path::new("/repos/a"), Some(port));
        let elapsed = started.elapsed();

        held.recv_timeout(Duration::from_secs(1))
            .expect("the probe never connected");
        assert_eq!(found, None);
        assert!(
            elapsed < Duration::from_secs(4),
            "the probe hung for {elapsed:?}"
        );
    }

    #[test]
    fn nothing_listening_is_not_a_dashboard() {
        let (listener, port) = listener();
        drop(listener);

        assert_eq!(find(Path::new("/repos/a"), Some(port)), None);
    }

    #[test]
    fn every_dashboard_in_the_range_is_enumerated_in_port_order() {
        let first = PathBuf::from("/repos/a");
        let second = PathBuf::from("/repos/b");
        let ports = [identity_server(&first), identity_server(&second)];
        let roots = [first, second];

        let found = enumerate_in(&ports);

        assert_eq!(found.len(), 2, "{found:?}");
        let mut expected: Vec<_> = ports.into_iter().zip(roots).collect();
        expected.sort_by_key(|(port, _)| *port);
        for ((port, identity), (expected_port, root)) in found.iter().zip(expected) {
            assert_eq!(*port, expected_port);
            assert_eq!(identity.root, root);
            assert_eq!(identity.pid, std::process::id());
        }
    }

    #[test]
    fn enumeration_skips_a_listener_that_is_not_a_dashboard() {
        let (_listener, occupied) = listener();
        let dashboard = identity_server(Path::new("/repos/a"));

        let found = enumerate_in(&[occupied, dashboard]);

        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, dashboard);
    }

    /// No dashboards is the ordinary state of a machine, not a failure.
    #[test]
    fn enumerating_nothing_listening_is_an_empty_result() {
        let (listener, port) = listener();
        drop(listener);

        assert_eq!(enumerate_in(&[port]), Vec::new());
    }

    #[test]
    fn a_dashboard_on_this_projects_assignment_is_reused() {
        let fixture = fixture();
        let port = identity_server(&fixture.project.root);
        dashboard::record(&fixture.project.root, port).expect("record");

        let found = ensure_in(&fixture.project, &[port]).expect("ensure");

        assert_eq!(found, port);
        assert_eq!(
            dashboard::assigned(&fixture.project.root).expect("read back"),
            Some(port),
            "an assignment that was already right must not be rewritten to something else"
        );
    }

    /// The registry is a hint. A dashboard somewhere else is still this project's
    /// dashboard, and the entry is the thing that was wrong.
    #[test]
    fn a_dashboard_found_off_its_assignment_is_reused_and_the_entry_repaired() {
        let fixture = fixture();
        let (elsewhere, stale) = listener();
        drop(elsewhere);
        let actual = identity_server(&fixture.project.root);
        dashboard::record(&fixture.project.root, stale).expect("record the stale entry");

        let found = ensure_in(&fixture.project, &[stale, actual]).expect("ensure");

        assert_eq!(found, actual);
        assert_eq!(
            dashboard::assigned(&fixture.project.root).expect("read back"),
            Some(actual),
            "the entry must be repaired, or the project's URL stays wrong"
        );
    }

    /// The case the rejected `serve.json` design fails: the record is gone and the
    /// server is still running. The sweep is what makes losing the record cost
    /// stability and never correctness.
    #[test]
    fn a_deleted_registry_still_finds_a_running_dashboard_and_starts_nothing() {
        let fixture = fixture();
        let running = identity_server(&fixture.project.root);
        assert_eq!(
            dashboard::assigned(&fixture.project.root).expect("read"),
            None,
            "this test is about a project the registry has never heard of"
        );

        let found = ensure_in(&fixture.project, &[running]).expect("ensure");

        assert_eq!(found, running, "the running dashboard was not adopted");
        assert!(
            !log_path(&fixture.project).exists(),
            "a dashboard was started when one was already serving"
        );
    }

    /// A dashboard for somebody else on the assignment does not stop this project's
    /// own being found further along the sweep.
    #[test]
    fn a_squatting_dashboard_on_the_assignment_is_swept_past() {
        let fixture = fixture();
        let squatter = identity_server(Path::new("/repos/someone-else"));
        let mine = identity_server(&fixture.project.root);
        dashboard::record(&fixture.project.root, squatter).expect("record");

        let found = ensure_in(&fixture.project, &[squatter, mine]).expect("ensure");

        assert_eq!(found, mine);
    }
}
