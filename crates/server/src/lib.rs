//! The `serve` subcommand's HTTP server: the built frontend and the index data
//! it renders, plus session- and change-scoped routes over a discovered OpenSpec
//! project, each with a server-sent-events endpoint fed by a filesystem watcher
//! so an open page updates without a reload.

mod api;
mod assets;
mod browser;
mod error;
mod idle;
pub mod markdown;
mod routes;
mod scope;
mod watch;

use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Duration;

use openspec_doc_core::{Project, dashboard};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

pub use browser::open as open_browser;
pub use error::Error;
pub use idle::Activity;

/// How long a dashboard asked for by nobody may go unused before it exits.
///
/// Long enough to cover a reviewer reading a page whose tab a browser may have
/// frozen; short enough that an abandoned dashboard does not outlive the working
/// day. There is deliberately no flag to tune it: a different value is
/// speculative until something wants one.
pub const IDLE_EXIT_AFTER: Duration = Duration::from_secs(30 * 60);

/// Which port to bind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortChoice {
    /// Exactly this port, failing rather than binding a different one. Someone
    /// who names a port wants that port, and silently serving somewhere else is
    /// how two tabs end up disagreeing about which dashboard they are looking at.
    Exact(u16),
    /// The port assigned to this project, falling forward through
    /// [`dashboard::RANGE`] when it is taken by something else and recording
    /// where it landed. `None` when no assignment could be read at all.
    Assigned(Option<u16>),
}

/// Where the dashboard binds, whether starting it launches a browser, and
/// whether it is allowed to give up.
#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub host: String,
    pub port: PortChoice,
    pub open_browser: bool,
    /// How long the dashboard may go unused before it exits, or `None` to serve
    /// indefinitely.
    ///
    /// A server run by hand was asked for by a person who is standing there, and
    /// exiting under them is a surprise. A hook-started one was asked for by
    /// nobody, which is what earns it a deadline — so this is what makes the
    /// process's ownership explicit rather than implicit.
    pub idle_exit: Option<Duration>,
}

/// Serve `project`'s dashboard, blocking until the server stops.
pub fn serve(project: Project, options: ServeOptions) -> Result<(), Error> {
    let runtime = Runtime::new().map_err(|source| Error::Runtime { source })?;

    runtime.block_on(run(project, options))
}

async fn run(project: Project, options: ServeOptions) -> Result<(), Error> {
    let listener = bind(&project, &options.host, options.port).await?;
    let address = listener
        .local_addr()
        .map_err(|source| Error::LocalAddr { source })?;
    let url = dashboard::url(&options.host, address.port());

    println!("serving {} at {url}", project.root.display());
    if options.open_browser {
        browser::open(&url);
    }

    let hub = Arc::new(watch::Hub::default());
    let activity = Arc::new(Activity::default());
    let serving = axum::serve(
        listener,
        routes::router(project, hub.clone(), activity.clone()),
    );

    match options.idle_exit {
        Some(window) => {
            serving
                .with_graceful_shutdown(async move { idle::unused(&hub, &activity, window).await })
                .await
        }
        None => serving.await,
    }
    .map_err(|source| Error::Serve { source })
}

/// Bind the port `choice` asks for.
///
/// The assigned port is tried first because it is nearly always free, and the
/// rest of the range is tried in ascending order after it. A port other than the
/// assignment is recorded as the project's assignment from then on, so the fall
/// forward happens once rather than at every start.
async fn bind(project: &Project, host: &str, choice: PortChoice) -> Result<TcpListener, Error> {
    let assigned = match choice {
        PortChoice::Exact(port) => return listen(host, port).await,
        PortChoice::Assigned(assigned) => assigned,
    };

    let candidates = assigned
        .into_iter()
        .chain(dashboard::RANGE.filter(|port| Some(*port) != assigned));

    for port in candidates {
        match listen(host, port).await {
            Ok(listener) => {
                if Some(port) != assigned {
                    dashboard::record(&project.root, port)?;
                }
                return Ok(listener);
            }
            // Only a port being unavailable is worth trying the next one for. A
            // host that will not resolve resolves no better on port 4322.
            Err(Error::Bind { .. }) => continue,
            Err(other) => return Err(other),
        }
    }

    Err(Error::RangeExhausted {
        host: host.to_owned(),
    })
}

async fn listen(host: &str, port: u16) -> Result<TcpListener, Error> {
    let address = (host, port)
        .to_socket_addrs()
        .map_err(|source| Error::ResolveHost {
            host: host.to_owned(),
            port,
            source,
        })?
        .next()
        .ok_or_else(|| Error::UnresolvedHost {
            host: host.to_owned(),
            port,
        })?;

    TcpListener::bind(address)
        .await
        .map_err(|source| Error::Bind { address, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    use openspec_doc_core::project_at;
    use tempfile::TempDir;

    /// The state-directory override is process-wide, so the tests that bind on
    /// an assignment take a turn each rather than racing over one variable.
    static ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct Fixture {
        _guard: std::sync::MutexGuard<'static, ()>,
        _state: TempDir,
        roots: TempDir,
    }

    fn fixture() -> Fixture {
        let guard = ENV.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let state = TempDir::new().expect("state dir");
        // SAFETY: `ENV` makes this the only test touching the variable.
        unsafe { std::env::set_var(dashboard::STATE_DIR_ENV, state.path()) };

        Fixture {
            _guard: guard,
            _state: state,
            roots: TempDir::new().expect("roots dir"),
        }
    }

    impl Fixture {
        /// A project root the registry can be keyed on.
        fn project(&self, name: &str) -> Project {
            let root = self.roots.path().join(name);
            fs::create_dir_all(root.join("openspec")).expect("create openspec dir");
            fs::write(root.join("openspec/config.yaml"), "").expect("write config");
            project_at(&root).expect("project root")
        }
    }

    /// A port the operating system says is free, held open so nothing else takes
    /// it before the test does.
    async fn squatted() -> (TcpListener, u16) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let port = listener.local_addr().expect("local addr").port();
        (listener, port)
    }

    async fn bound_port(project: &Project, choice: PortChoice) -> Result<u16, Error> {
        let listener = bind(project, "127.0.0.1", choice).await?;
        Ok(listener.local_addr().expect("local addr").port())
    }

    fn assigned(root: &Path) -> Option<u16> {
        dashboard::assigned(root).expect("read assignment")
    }

    /// A port from the operating system, released, so the assignment under test
    /// names something bindable.
    ///
    /// Deliberately not a port from the real range. A test that assigns 4321 and
    /// then binds it passes only while no dashboard is on 4321 — and after this
    /// change, the developer's own dashboard is on exactly that port on exactly
    /// the machine that matters. The claim being tested is "the default binds
    /// whatever the assignment names", and that does not need the assignment to be
    /// a real-range number.
    async fn free_port() -> u16 {
        let (listener, port) = squatted().await;
        drop(listener);
        port
    }

    #[tokio::test]
    async fn the_default_binds_the_projects_assignment() {
        let fixture = fixture();
        let project = fixture.project("a");
        let port = free_port().await;
        dashboard::record(&project.root, port).expect("record the assignment");

        assert_eq!(
            bound_port(&project, PortChoice::Assigned(assigned(&project.root)))
                .await
                .expect("bind"),
            port,
            "the default must bind the port the assignment names"
        );
    }

    #[tokio::test]
    async fn two_projects_do_not_collide() {
        let fixture = fixture();
        let first = fixture.project("a");
        let second = fixture.project("b");

        assert_ne!(
            dashboard::assign(&first.root).expect("assign first"),
            dashboard::assign(&second.root).expect("assign second"),
            "two roots must not be handed the same port"
        );

        // And each binds its own, on ports this machine will actually give up.
        let (first_port, second_port) = (free_port().await, free_port().await);
        dashboard::record(&first.root, first_port).expect("record first");
        dashboard::record(&second.root, second_port).expect("record second");
        let held = bind(
            &first,
            "127.0.0.1",
            PortChoice::Assigned(assigned(&first.root)),
        )
        .await
        .expect("bind first");

        assert_eq!(
            bound_port(&second, PortChoice::Assigned(assigned(&second.root)))
                .await
                .expect("bind second"),
            second_port,
            "the second project was pushed off its own assignment"
        );
        assert_eq!(
            held.local_addr().expect("local addr").port(),
            first_port,
            "the first project was pushed off its own assignment"
        );
    }

    /// Order-independence is the whole point: a project's port must not depend on
    /// which checkout started first.
    #[tokio::test]
    async fn the_same_project_binds_the_same_port_with_others_served_in_between() {
        let fixture = fixture();
        let project = fixture.project("a");

        let first_run = bound_port(&project, PortChoice::Assigned(assigned(&project.root)))
            .await
            .expect("first bind");
        dashboard::assign(&fixture.project("b").root).expect("serve another project");
        dashboard::assign(&fixture.project("c").root).expect("serve another project");
        let second_run = bound_port(&project, PortChoice::Assigned(assigned(&project.root)))
            .await
            .expect("second bind");

        assert_eq!(first_run, second_run);
    }

    #[tokio::test]
    async fn a_squatted_assignment_falls_forward_and_records_where_it_landed() {
        let fixture = fixture();
        let project = fixture.project("a");
        let (_squatter, squatted) = squatted().await;

        let port = bound_port(&project, PortChoice::Assigned(Some(squatted)))
            .await
            .expect("bind");

        assert_ne!(port, squatted);
        assert!(dashboard::RANGE.contains(&port), "left the range: {port}");
        assert_eq!(
            assigned(&project.root),
            Some(port),
            "the fall forward must be recorded, or it happens again next start"
        );
    }

    /// With no assignment to read — a corrupt registry, reported and continued
    /// past — the range is still where the dashboard belongs.
    #[tokio::test]
    async fn no_assignment_binds_in_the_range_and_records_it() {
        let fixture = fixture();
        let project = fixture.project("a");

        let port = bound_port(&project, PortChoice::Assigned(None))
            .await
            .expect("bind");

        assert!(dashboard::RANGE.contains(&port), "left the range: {port}");
        assert_eq!(assigned(&project.root), Some(port));
    }

    #[tokio::test]
    async fn an_explicit_taken_port_fails_rather_than_falling_forward() {
        let fixture = fixture();
        let project = fixture.project("a");
        let (_squatter, squatted) = squatted().await;

        let error = bound_port(&project, PortChoice::Exact(squatted))
            .await
            .expect_err("taken port");

        assert!(matches!(error, Error::Bind { .. }), "{error:?}");
    }

    #[tokio::test]
    async fn an_explicit_port_leaves_the_assignment_untouched() {
        let fixture = fixture();
        let project = fixture.project("a");
        let (squatter, port) = squatted().await;
        drop(squatter);

        bound_port(&project, PortChoice::Exact(port))
            .await
            .expect("bind");

        assert_eq!(
            assigned(&project.root),
            None,
            "an explicit port is not a statement about where the project lives"
        );
    }

    /// The flag is what makes the process's ownership explicit: a server a person
    /// started in a terminal must not vanish under them.
    #[tokio::test]
    async fn idleness_ends_the_server_only_when_it_was_asked_for() {
        let fixture = fixture();
        let project = fixture.project("a");
        let window = Duration::from_millis(200);
        let asked = ServeOptions {
            host: "127.0.0.1".to_owned(),
            port: PortChoice::Assigned(None),
            open_browser: false,
            idle_exit: Some(window),
        };
        let unasked = ServeOptions {
            idle_exit: None,
            ..asked.clone()
        };

        assert!(
            tokio::time::timeout(window * 20, run(project.clone(), asked))
                .await
                .expect("an idle server with --idle-exit never exited")
                .is_ok()
        );
        assert!(
            tokio::time::timeout(window * 5, run(project, unasked))
                .await
                .is_err(),
            "a server nobody asked to exit gave up under its operator"
        );
    }

    #[tokio::test]
    async fn an_exhausted_range_fails_naming_the_range() {
        let fixture = fixture();
        let project = fixture.project("a");
        let mut held = Vec::new();
        for port in dashboard::RANGE {
            // The developer's own dashboards sit in this range, so a port that is
            // already taken is the state this test wants anyway.
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)).await {
                held.push(listener);
            }
        }

        let error = bound_port(&project, PortChoice::Assigned(None))
            .await
            .expect_err("exhausted range");

        assert!(matches!(error, Error::RangeExhausted { .. }), "{error:?}");
        let message = error.to_string();
        assert!(
            message.contains(&dashboard::RANGE.start().to_string())
                && message.contains(&dashboard::RANGE.end().to_string()),
            "the error must name the range: {message}"
        );
    }
}
