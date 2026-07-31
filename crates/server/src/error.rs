use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] openspec_doc_core::Error),

    #[error("failed to list {}", path.display())]
    ReadDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to build the async runtime")]
    Runtime {
        #[source]
        source: io::Error,
    },

    #[error("failed to resolve the address {host}:{port}")]
    ResolveHost {
        host: String,
        port: u16,
        #[source]
        source: io::Error,
    },

    #[error("{host}:{port} resolved to no address")]
    UnresolvedHost { host: String, port: u16 },

    #[error("failed to bind {address}")]
    Bind {
        address: SocketAddr,
        #[source]
        source: io::Error,
    },

    #[error("failed to read the address of the bound listener")]
    LocalAddr {
        #[source]
        source: io::Error,
    },

    #[error("the HTTP server stopped with an error")]
    Serve {
        #[source]
        source: io::Error,
    },
}

/// `error` and its source chain on one line, for reporting a failure that
/// happens inside a request handler rather than propagating out of `serve`.
pub(crate) fn chain(error: &dyn std::error::Error) -> String {
    let mut description = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        description.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    description
}
