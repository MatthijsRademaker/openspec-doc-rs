//! `serve kill`: ending a dashboard on purpose.
//!
//! Over HTTP on the dashboard's own port, never by signalling its pid. A pid is
//! a number that may have been reused between learning it and acting on it,
//! while a port that stops answering is the evidence directly — and asking a
//! server to stop itself lets it check what it is, which a signal cannot. It also
//! costs no dependency: Rust's std cannot send a signal, and a hand-rolled `POST`
//! beside the hand-rolled `GET` in `discovery` costs nothing.
//!
//! The price, stated where an operator meets it: a dashboard so wedged it no
//! longer answers HTTP cannot be stopped from here. That is the dashboard one
//! most wants to kill, and no fallback is built for it because no such dashboard
//! has been observed — the pid `serve list` prints is the manual way out until
//! one is.

use std::path::PathBuf;

use openspec_doc_core::Project;
use openspec_doc_core::dashboard::Identity;

use crate::discovery::{self, Stopped};
use crate::error::Error;

/// Which dashboards to stop. One of these is always given, because none of the
/// three is safe to imply.
pub enum Target {
    /// The one serving this project.
    Project(Project),
    /// Whatever is answering on this port.
    Port(u16),
    /// Every one answering in the range.
    All,
}

impl Target {
    /// How the target reads back in a report that nothing matched it.
    fn named(&self) -> String {
        match self {
            Self::Project(project) => format!("a dashboard for {}", project.root.display()),
            Self::Port(port) => format!("a dashboard on port {port}"),
            Self::All => "any dashboard in the port range".to_owned(),
        }
    }
}

/// Stop the dashboards `target` names, and report what is actually gone.
pub fn run(target: Target) -> Result<(), Error> {
    run_in(&target, &discovery::range())
}

/// As [`run`], over `range`, which is the seam the tests need: the real range is
/// where the developer's own dashboards are sitting, and a test for a command
/// that stops things must not be pointed at them.
fn run_in(target: &Target, range: &[u16]) -> Result<(), Error> {
    // Enumerated here rather than taken from a table the operator was shown some
    // seconds ago, which narrows the window between resolving a target and acting
    // on it to milliseconds. The server refusing a request that names a root it
    // does not serve is what closes the rest of it.
    let matched = matching(target, discovery::enumerate_in(range));
    if matched.is_empty() {
        return Err(Error::NothingToStop {
            target: target.named(),
        });
    }

    // The port assignments are deliberately untouched. Stopping a dashboard says
    // nothing about where its project belongs, and a kill that also forgot the
    // assignment would move the project on its next start — the instability this
    // line of work removed. `serve forget` is that act, kept separate.
    let outcomes: Vec<Outcome> = matched
        .into_iter()
        .map(|(port, identity)| Outcome {
            port,
            stopped: discovery::stop(port, &identity.root),
            root: identity.root,
        })
        .collect();

    print!("{}", report(&outcomes));

    let stopped = outcomes
        .iter()
        .filter(|outcome| outcome.stopped == Stopped::Yes)
        .count();
    if stopped < outcomes.len() {
        return Err(Error::NotEveryDashboardStopped {
            stopped,
            total: outcomes.len(),
        });
    }

    Ok(())
}

/// One dashboard asked to stop, and what came of it.
struct Outcome {
    port: u16,
    root: PathBuf,
    stopped: Stopped,
}

/// The enumerated dashboards `target` names.
fn matching(target: &Target, running: Vec<(u16, Identity)>) -> Vec<(u16, Identity)> {
    running
        .into_iter()
        .filter(|(port, identity)| match target {
            Target::Project(project) => identity.root == project.root,
            Target::Port(wanted) => port == wanted,
            Target::All => true,
        })
        .collect()
}

/// What `kill` prints: a line per dashboard it asked to stop, saying what
/// happened to it rather than that a request went out.
fn report(outcomes: &[Outcome]) -> String {
    outcomes
        .iter()
        .map(|outcome| {
            format!(
                "{} {}: {}\n",
                outcome.port,
                outcome.root.display(),
                described(&outcome.stopped)
            )
        })
        .collect()
}

fn described(stopped: &Stopped) -> String {
    match stopped {
        Stopped::Yes => "stopped".to_owned(),
        Stopped::AlreadyGone => {
            "was not running: it had already exited before the request".to_owned()
        }
        // The limit this command accepts rather than escalating past, so the one
        // thing owed is naming the way out where the operator meets it.
        Stopped::StillServing => "still running: it answered the request and kept serving. \
             Stop it by the pid `openspec-doc serve list` prints"
            .to_owned(),
        Stopped::Refused {
            status,
            serving: Some(root),
        } => format!(
            "not stopped: refused with {status}, because the port now serves {}",
            root.display()
        ),
        Stopped::Refused {
            status,
            serving: None,
        } => format!("not stopped: refused with {status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::Path;
    use std::sync::Arc as StdArc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use openspec_doc_core::dashboard;

    /// How a fake dashboard behaves when it is asked to stop.
    ///
    /// A fake rather than the real server, and the seam is deliberate: what is
    /// under test here is the command's half of the conversation — that it
    /// enumerates late, reports off a probe rather than off the answer, and
    /// leaves the dashboards it was not pointed at alone. The server's half is
    /// tested against the real router in `openspec-doc-server`, and only a fake
    /// can be told to answer and then ignore the request.
    enum Behaviour {
        /// Answers and stops, which is what a dashboard does.
        Obedient,
        /// Answers and keeps serving: the wedged dashboard this command cannot
        /// escalate past.
        Wedged,
        /// Answers one identity probe and exits, so the request that follows the
        /// enumeration finds nothing listening.
        Vanishing,
        /// Answers its first identity probe as its own root and everything after
        /// as this one, refusing a shutdown that names anybody else: the port
        /// changed hands between the enumeration and the request.
        HandedOver(PathBuf),
    }

    /// A fake dashboard for `root` on a port the operating system says is free.
    ///
    /// Never a port from the real range. That range is where the developer's own
    /// dashboards are sitting, and a test for a command whose job is stopping
    /// things must not be pointed at them.
    fn dashboard(root: &Path, behaviour: Behaviour) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("local addr").port();
        let root = root.to_owned();
        let probes = StdArc::new(AtomicUsize::new(0));
        let vanishing = matches!(behaviour, Behaviour::Vanishing);
        let wedged = matches!(behaviour, Behaviour::Wedged);

        std::thread::spawn(move || {
            for connection in listener.incoming() {
                let Ok(mut stream) = connection else { break };
                let request = read_request(&stream);
                if request.is_empty() {
                    // A connection that asked nothing, which is how a test waits
                    // for this to be up. Counting it as a probe would spend a
                    // `Vanishing` dashboard's one answer, or hand a
                    // `HandedOver` port over before the enumeration ran.
                    continue;
                }
                let serving = match &behaviour {
                    Behaviour::HandedOver(other) if probes.load(Ordering::Relaxed) > 0 => {
                        other.clone()
                    }
                    _ => root.clone(),
                };

                if request.starts_with(&format!("POST {}", dashboard::SHUTDOWN_PATH)) {
                    let named = header_in(&request, dashboard::SHUTDOWN_ROOT_HEADER);
                    if named.as_deref() != Some(serving.display().to_string().as_str()) {
                        let _ = stream.write_all(b"HTTP/1.0 409 Conflict\r\n\r\n");
                        continue;
                    }

                    let _ = stream.write_all(b"HTTP/1.0 202 Accepted\r\n\r\n");
                    drop(stream);
                    if wedged {
                        continue;
                    }

                    break;
                }

                probes.fetch_add(1, Ordering::Relaxed);
                let body = serde_json::json!({
                    "root": serving.display().to_string(),
                    "pid": std::process::id(),
                })
                .to_string();
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\n\
                         Content-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
                drop(stream);
                if vanishing {
                    break;
                }
            }
        });

        // Answered once here, so a `Vanishing` dashboard is past its one probe
        // only after the enumeration has seen it rather than before.
        if !vanishing {
            wait_until_answering(port);
        }

        port
    }

    fn read_request(stream: &TcpStream) -> String {
        let mut reader = BufReader::new(stream);
        let mut request = String::new();
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                break;
            }
            request.push_str(&line);
        }

        request
    }

    fn header_in(request: &str, name: &str) -> Option<String> {
        request
            .lines()
            .filter_map(|line| line.split_once(": "))
            .find(|(field, _)| field.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.trim().to_owned())
    }

    fn wait_until_answering(port: u16) {
        for _ in 0..100 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("the fake dashboard on {port} never came up");
    }

    fn still_serving(port: u16) -> bool {
        TcpStream::connect(("127.0.0.1", port)).is_ok()
    }

    /// A project root the enumeration can key on. Resolved the way every command
    /// resolves one, so the root the fake dashboard reports is spelled exactly
    /// the way a real one would spell it.
    fn project() -> (tempfile::TempDir, Project) {
        let dir = tempfile::TempDir::new().expect("project root");
        std::fs::create_dir_all(dir.path().join("openspec")).expect("create openspec dir");
        std::fs::write(dir.path().join("openspec/config.yaml"), "").expect("write config");
        let project = openspec_doc_core::project_at(dir.path()).expect("project");

        (dir, project)
    }

    /// Two checkouts, which is the situation the command exists for: the review
    /// open on the other one must survive.
    #[test]
    fn a_targeted_kill_stops_one_and_leaves_the_others_serving() {
        let (_mine_dir, mine) = project();
        let (_theirs_dir, theirs) = project();
        let ports = [
            dashboard(&mine.root, Behaviour::Obedient),
            dashboard(&theirs.root, Behaviour::Obedient),
        ];

        run_in(&Target::Project(mine), &ports).expect("kill");

        assert!(
            !still_serving(ports[0]),
            "the targeted dashboard is still there"
        );
        assert!(
            still_serving(ports[1]),
            "another checkout's dashboard was stopped too"
        );
    }

    #[test]
    fn a_kill_by_port_stops_whatever_is_on_that_port_and_nothing_else() {
        let first = dashboard(Path::new("/repos/a"), Behaviour::Obedient);
        let second = dashboard(Path::new("/repos/b"), Behaviour::Obedient);

        run_in(&Target::Port(second), &[first, second]).expect("kill");

        assert!(still_serving(first));
        assert!(!still_serving(second));
    }

    #[test]
    fn all_stops_every_dashboard_in_the_range() {
        let ports = [
            dashboard(Path::new("/repos/a"), Behaviour::Obedient),
            dashboard(Path::new("/repos/b"), Behaviour::Obedient),
            dashboard(Path::new("/repos/c"), Behaviour::Obedient),
        ];

        run_in(&Target::All, &ports).expect("kill");

        for port in ports {
            assert!(!still_serving(port), "{port} survived --all");
        }
    }

    /// The nothing-matched case must not read as having stopped something, and a
    /// zero exit beside a dashboard that is still up is exactly that reading.
    #[test]
    fn a_target_matching_nothing_is_reported_rather_than_called_success() {
        let other = dashboard(Path::new("/repos/somebody-else"), Behaviour::Obedient);

        let (_dir, mine) = project();
        let root = mine.root.display().to_string();

        let error = run_in(&Target::Project(mine), &[other])
            .expect_err("a target that matched nothing reported success");

        assert!(matches!(error, Error::NothingToStop { .. }), "{error:?}");
        assert!(error.to_string().contains(&root), "{error}");
        assert!(still_serving(other), "an unmatched dashboard was stopped");
    }

    /// The wedged case the design declines to escalate for. It has to be visible
    /// when it happens rather than reported as stopped, which is the whole
    /// reason the outcome is read off a probe.
    #[test]
    fn a_dashboard_that_survives_the_request_is_reported_as_still_running() {
        let wedged = dashboard(Path::new("/repos/wedged"), Behaviour::Wedged);

        let error = run_in(&Target::Port(wedged), &[wedged])
            .expect_err("a dashboard that kept serving was reported as stopped");

        assert!(
            matches!(
                error,
                Error::NotEveryDashboardStopped {
                    stopped: 0,
                    total: 1
                }
            ),
            "{error:?}"
        );
        assert!(still_serving(wedged));
        assert_eq!(
            discovery::stop(wedged, Path::new("/repos/wedged")),
            Stopped::StillServing
        );
        assert!(
            described(&Stopped::StillServing).contains("serve list"),
            "the manual way out has to be named where the operator meets it"
        );
    }

    /// Reporting a dashboard the idle deadline already reaped as killed is
    /// claiming credit for somebody else's work.
    #[test]
    fn one_that_had_already_exited_is_reported_as_not_running() {
        let gone = dashboard(Path::new("/repos/gone"), Behaviour::Vanishing);

        let error = run_in(&Target::Port(gone), &[gone])
            .expect_err("a dashboard that had already exited was reported as stopped");

        assert!(
            matches!(
                error,
                Error::NotEveryDashboardStopped {
                    stopped: 0,
                    total: 1
                }
            ),
            "{error:?}"
        );
        assert!(
            described(&Stopped::AlreadyGone).contains("not running"),
            "{}",
            described(&Stopped::AlreadyGone)
        );
    }

    /// The fall-forward race, and the case a signal could not have caught: by
    /// the time the request arrived the port held another project's dashboard,
    /// and only the server knew.
    #[test]
    fn a_port_that_changed_hands_between_enumeration_and_request_is_not_stopped() {
        let port = dashboard(
            Path::new("/repos/was-here"),
            Behaviour::HandedOver(PathBuf::from("/repos/fell-forward")),
        );

        let error = run_in(&Target::Port(port), &[port])
            .expect_err("a port that changed hands was reported as stopped");

        assert!(
            matches!(
                error,
                Error::NotEveryDashboardStopped {
                    stopped: 0,
                    total: 1
                }
            ),
            "{error:?}"
        );
        assert!(
            still_serving(port),
            "the dashboard that had fallen forward onto the port was stopped"
        );
    }

    /// A kill that also forgot the assignment would move the project on its next
    /// start, which is the instability this line of work removed. `serve forget`
    /// is that act, and it is deliberately a separate one.
    #[test]
    fn a_killed_project_keeps_its_port_assignment() {
        static ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = ENV.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let state = tempfile::TempDir::new().expect("state dir");
        // SAFETY: `ENV` makes this the only test here touching the variable.
        unsafe { std::env::set_var(dashboard::STATE_DIR_ENV, state.path()) };
        let (_dir, mine) = project();
        let assigned = dashboard::assign(&mine.root).expect("assign");
        let port = dashboard(&mine.root, Behaviour::Obedient);
        let root = mine.root.clone();

        run_in(&Target::Project(mine), &[port]).expect("kill");

        assert!(!still_serving(port));
        assert_eq!(
            dashboard::assigned(&root).expect("read back"),
            Some(assigned),
            "stopping a dashboard dropped its project's port assignment"
        );
    }

    #[test]
    fn the_report_names_the_port_and_the_root_of_every_dashboard_it_asked() {
        let outcomes = [
            Outcome {
                port: 4321,
                root: PathBuf::from("/repos/a"),
                stopped: Stopped::Yes,
            },
            Outcome {
                port: 4322,
                root: PathBuf::from("/repos/b"),
                stopped: Stopped::Refused {
                    status: 409,
                    serving: Some(PathBuf::from("/repos/c")),
                },
            },
        ];

        let report = report(&outcomes);

        assert!(report.contains("4321 /repos/a: stopped"), "{report}");
        assert!(report.contains("4322 /repos/b: not stopped"), "{report}");
        assert!(
            report.contains("/repos/c"),
            "a refusal has to name what is actually there: {report}"
        );
    }
}
