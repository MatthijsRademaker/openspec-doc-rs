use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] openspec_doc_core::Error),

    #[error(transparent)]
    Server(#[from] openspec_doc_server::Error),

    #[error("failed to determine the current directory")]
    CurrentDir {
        #[source]
        source: io::Error,
    },

    #[error("failed to read the hook payload from stdin")]
    Stdin {
        #[source]
        source: io::Error,
    },

    #[error("failed to locate this executable to start the dashboard with")]
    CurrentExe {
        #[source]
        source: io::Error,
    },

    #[error("failed to open the dashboard's log at {}", path.display())]
    Log {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to start the dashboard")]
    Spawn {
        #[source]
        source: io::Error,
    },

    #[error("the dashboard did not answer within {seconds}s; its output is in {}", log.display())]
    DashboardDidNotStart { seconds: u64, log: PathBuf },

    #[error("no port is assigned to this project and no dashboard is serving it")]
    NoAssignedPort,
}
