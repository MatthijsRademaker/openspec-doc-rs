use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use openspec_doc_core::Project;
use openspec_doc_core::dashboard::{self, Identity};
use openspec_doc_server::{PortChoice, ServeOptions};

use crate::discovery;
use crate::error::Error;

/// Start the dashboard server and block until it stops.
pub fn run(
    project: Project,
    host: String,
    port: Option<u16>,
    no_open: bool,
    idle_exit: bool,
) -> Result<(), Error> {
    // An explicit `--port` neither reads nor writes the assignment: someone who
    // names a port wants that port for this invocation, not a change to where the
    // project lives.
    let port = match port {
        Some(port) => PortChoice::Exact(port),
        None => PortChoice::Assigned(discovery::preferred(&project)),
    };

    Ok(openspec_doc_server::serve(
        project,
        ServeOptions {
            host,
            port,
            open_browser: !no_open,
            idle_exit: idle_exit.then_some(openspec_doc_server::IDLE_EXIT_AFTER),
        },
    )?)
}

/// Print this project's dashboard URL and whether a dashboard is serving it.
///
/// Answering with nothing running is the capability the port assignment exists to
/// provide: before it, nothing could name a project's URL until something had
/// bound a port.
pub fn url(project: Project) -> Result<(), Error> {
    let assigned = discovery::preferred(&project);

    // A dashboard found somewhere other than its assignment is reported where it
    // actually is, not where it was supposed to be.
    if let Some(port) = discovery::find(&project.root, assigned) {
        println!("{} (serving)", dashboard::url(discovery::HOST, port));

        return Ok(());
    }

    match assigned {
        Some(port) => {
            println!("{} (not running)", dashboard::url(discovery::HOST, port));

            Ok(())
        }
        // With no assignment readable and nothing running there is no URL to give,
        // and inventing one would point the reviewer at another project's queue.
        // `preferred` has already reported why.
        None => Err(Error::NoAssignedPort),
    }
}

/// Print every dashboard serving on the port range, and every port assigned with
/// nothing serving it.
///
/// Needs no project: dashboards are global to the machine and started detached by
/// hooks, so the question this answers — which of these is which — belongs to the
/// machine rather than to whichever checkout the operator happens to be standing
/// in.
pub fn list() -> Result<(), Error> {
    print!("{}", table(&discovery::enumerate(), &assignments()));

    Ok(())
}

/// Drop `root`'s port assignment.
///
/// By root rather than by port: the operator's question is "I do not use that
/// checkout any more", and answering it by port makes them look up something they
/// did not ask about.
pub fn forget(root: &Path) -> Result<(), Error> {
    let root = openspec_doc_core::canonicalize(root)?;

    // Asked of the network before anything is dropped. Handing the port to
    // another project while a server still sits on it leaves the forgotten root
    // to rediscover its own dashboard on a port that now belongs to someone else.
    if let Some(port) = discovery::find(&root, dashboard::assigned(&root)?) {
        return Err(Error::DashboardServing { root, port });
    }

    match dashboard::forget(&root)? {
        Some(port) => println!("forgot {} (was assigned port {port})", root.display()),
        None => println!("{} has no assigned port: nothing to forget", root.display()),
    }

    Ok(())
}

/// The assignments, or none of them: a registry that cannot be read is reported
/// and carried past, because the dashboards it does not describe are still
/// running and still worth listing.
///
/// Assignments only. A row is marked running because a probe answered, never
/// because a file said so — reading a running state off the disk is the failure
/// the whole discovery design exists to prevent, and this is where it would
/// re-enter the project.
fn assignments() -> BTreeMap<PathBuf, u16> {
    match dashboard::assignments() {
        Ok(assignments) => assignments,
        Err(error) => {
            crate::report("could not read the assigned ports; listing what is running");
            crate::eprint_chain(&error);
            BTreeMap::new()
        }
    }
}

/// What `list` prints: the dashboards `running` answered on, the assigned ports
/// nothing answered on, and the range that was asked.
fn table(running: &[(u16, Identity)], assignments: &BTreeMap<PathBuf, u16>) -> String {
    let mut rows: Vec<Row> = running
        .iter()
        .map(|(port, identity)| Row::running(*port, identity, assignments))
        .chain(idle(assignments, running))
        .collect();
    // One ascending list rather than the running ones and then the idle ones: the
    // operator is reading down a column of ports.
    rows.sort_by_key(|row| row.port);

    let mut out = if rows.is_empty() {
        "no dashboards running\n".to_owned()
    } else {
        rendered(&rows)
    };

    // Always, and last: an operator who does not find the server they started
    // reads why on the line under the answer, rather than concluding the tool is
    // broken. A dashboard on a port outside this range cannot be found by
    // probing, and saying so is the whole mitigation.
    out.push_str(&format!(
        "probed ports {}-{} on {}\n",
        dashboard::RANGE.start(),
        dashboard::RANGE.end(),
        discovery::HOST
    ));

    out
}

/// One line of the table.
struct Row {
    port: u16,
    root: PathBuf,
    pid: Option<u32>,
    state: &'static str,
}

impl Row {
    fn running(port: u16, identity: &Identity, assignments: &BTreeMap<PathBuf, u16>) -> Self {
        Self {
            port,
            root: identity.root.clone(),
            pid: Some(identity.pid),
            state: match assignments.get(&identity.root) {
                Some(assigned) if *assigned == port => "running",
                // The state `ensure` repairs on its next run. Printed as an
                // ordinary row it would be how a repair path that stopped firing
                // stays invisible.
                Some(_) => "running, not its assignment",
                // Started by hand with `--port`, rather than placed by a hook.
                None => "running, no assignment",
            },
        }
    }
}

/// The assigned ports nothing is serving, for roots with no dashboard anywhere.
///
/// A root found off its assignment is one row, not two: its own row already says
/// where it actually is.
fn idle<'a>(
    assignments: &'a BTreeMap<PathBuf, u16>,
    running: &'a [(u16, Identity)],
) -> impl Iterator<Item = Row> + 'a {
    assignments
        .iter()
        .filter(|(root, _)| !running.iter().any(|(_, identity)| identity.root == **root))
        .map(|(root, port)| Row {
            port: *port,
            root: root.clone(),
            pid: None,
            state: "assigned, not running",
        })
}

fn rendered(rows: &[Row]) -> String {
    let roots: Vec<String> = rows
        .iter()
        .map(|row| row.root.display().to_string())
        .collect();
    let width = roots
        .iter()
        .map(|root| root.chars().count())
        .chain(std::iter::once("ROOT".len()))
        .max()
        .expect("the header is always there");

    let mut out = format!("{:<6} {:<width$} {:<7} STATE\n", "PORT", "ROOT", "PID");
    for (row, root) in rows.iter().zip(&roots) {
        let pid = match row.pid {
            Some(pid) => pid.to_string(),
            None => "-".to_owned(),
        };
        out.push_str(&format!(
            "{:<6} {root:<width$} {pid:<7} {}\n",
            row.port, row.state
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(root: &str, pid: u32) -> Identity {
        Identity {
            root: PathBuf::from(root),
            pid,
        }
    }

    fn assignments(entries: &[(&str, u16)]) -> BTreeMap<PathBuf, u16> {
        entries
            .iter()
            .map(|(root, port)| (PathBuf::from(root), *port))
            .collect()
    }

    #[test]
    fn every_running_dashboard_is_listed_with_its_port_root_and_pid() {
        let running = [
            (4321, identity("/repos/a", 111)),
            (4322, identity("/repos/b", 222)),
        ];

        let table = table(
            &running,
            &assignments(&[("/repos/a", 4321), ("/repos/b", 4322)]),
        );

        for expected in ["4321", "/repos/a", "111", "4322", "/repos/b", "222"] {
            assert!(
                table.contains(expected),
                "{expected} missing from:\n{table}"
            );
        }
        assert_eq!(table.matches("running").count(), 2, "{table}");
    }

    #[test]
    fn the_searched_range_is_named_even_with_nothing_running() {
        let table = table(&[], &BTreeMap::new());

        assert!(table.contains("no dashboards running"), "{table}");
        assert!(table.contains("probed ports 4321-4352"), "{table}");
    }

    #[test]
    fn an_assigned_port_with_nothing_serving_it_is_shown_as_such() {
        let table = table(&[], &assignments(&[("/repos/idle", 4325)]));

        assert!(table.contains("/repos/idle"), "{table}");
        assert!(table.contains("4325"), "{table}");
        assert!(table.contains("assigned, not running"), "{table}");
        assert!(
            table.contains(" - "),
            "an idle row has no pid to print:\n{table}"
        );
    }

    /// The state `ensure` repairs on its next run, and an ordinary-looking row is
    /// how a repair path that stopped firing would stay invisible.
    #[test]
    fn a_dashboard_away_from_its_assignment_is_distinguished() {
        let running = [(4327, identity("/repos/a", 111))];

        let table = table(&running, &assignments(&[("/repos/a", 4321)]));

        assert!(
            table.contains("4327"),
            "the port it is actually on:\n{table}"
        );
        assert!(table.contains("running, not its assignment"), "{table}");
        assert!(
            !table.contains("assigned, not running"),
            "a root running off its assignment is one row, not two:\n{table}"
        );
    }

    /// Nothing in the registry may make a row look running, and nothing missing
    /// from it may hide one.
    #[test]
    fn losing_the_assignments_costs_the_idle_rows_and_nothing_else() {
        let running = [(4321, identity("/repos/a", 111))];

        let table = table(&running, &BTreeMap::new());

        assert!(table.contains("/repos/a"), "{table}");
        assert!(table.contains("111"), "{table}");
        assert!(
            !table.contains("assigned, not running"),
            "an idle row was invented from an empty registry:\n{table}"
        );
    }

    #[test]
    fn the_table_reads_in_port_order_whether_a_row_is_running_or_idle() {
        let running = [(4324, identity("/repos/late", 111))];

        let table = table(&running, &assignments(&[("/repos/early", 4322)]));

        let ports: Vec<&str> = table
            .lines()
            .skip(1)
            .filter_map(|line| line.split_whitespace().next())
            .collect();
        assert_eq!(ports, ["4322", "4324", "probed"], "{table}");
    }

    #[test]
    fn a_dashboard_nothing_assigned_is_marked_as_such() {
        let table = table(&[(4399, identity("/repos/by-hand", 111))], &BTreeMap::new());

        assert!(table.contains("running, no assignment"), "{table}");
    }
}
