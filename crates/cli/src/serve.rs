use openspec_doc_core::Project;
use openspec_doc_server::ServeOptions;

use crate::error::Error;

/// Start the dashboard server and block until it stops.
pub fn run(project: Project, host: String, port: u16, no_open: bool) -> Result<(), Error> {
    Ok(openspec_doc_server::serve(
        project,
        ServeOptions {
            host,
            port,
            open_browser: !no_open,
        },
    )?)
}
