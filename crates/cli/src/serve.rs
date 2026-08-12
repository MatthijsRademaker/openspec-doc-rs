use openspec_doc_core::Project;
use openspec_doc_core::dashboard;
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
