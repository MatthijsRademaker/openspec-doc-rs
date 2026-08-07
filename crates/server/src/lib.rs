//! The `serve` subcommand's HTTP server: the built frontend and the index data
//! it renders, plus session- and change-scoped routes over a discovered OpenSpec
//! project, each with a server-sent-events endpoint fed by a filesystem watcher
//! so an open page updates without a reload.

mod api;
mod assets;
mod browser;
mod error;
pub mod markdown;
mod routes;
mod scope;
mod watch;

use std::net::ToSocketAddrs;

use openspec_doc_core::Project;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

pub use error::Error;

/// Where the dashboard binds, and whether starting it launches a browser.
#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub host: String,
    /// `0` selects an available port.
    pub port: u16,
    pub open_browser: bool,
}

/// Serve `project`'s dashboard, blocking until the server stops.
pub fn serve(project: Project, options: ServeOptions) -> Result<(), Error> {
    let runtime = Runtime::new().map_err(|source| Error::Runtime { source })?;

    runtime.block_on(run(project, options))
}

async fn run(project: Project, options: ServeOptions) -> Result<(), Error> {
    let listener = bind(&options.host, options.port).await?;
    let address = listener
        .local_addr()
        .map_err(|source| Error::LocalAddr { source })?;
    let url = format!("http://{address}/");

    println!("serving {} at {url}", project.root.display());
    if options.open_browser {
        browser::open(&url);
    }

    axum::serve(listener, routes::router(project))
        .await
        .map_err(|source| Error::Serve { source })
}

async fn bind(host: &str, port: u16) -> Result<TcpListener, Error> {
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

    // Port 0 leaves the choice of an available port to the operating system;
    // `local_addr` then reports the one it picked.
    TcpListener::bind(address)
        .await
        .map_err(|source| Error::Bind { address, source })
}
