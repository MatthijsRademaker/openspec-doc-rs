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

    #[error(
        "a dashboard for {} is serving on port {port}: stop it before forgetting its assignment, \
         or the port is handed to another project with a server still on it",
        root.display()
    )]
    DashboardServing { root: PathBuf, port: u16 },

    #[error(
        "no agent harness was detected in this project: neither .claude/ nor .pi/ exists. \
         Name one explicitly with --agent <claude|pi>"
    )]
    NoHarnessDetected,

    #[error("failed to read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("{} is not valid JSON, so it cannot be merged into and will not be overwritten", path.display())]
    Settings {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to create the throwaway project the hook probes run against")]
    ProbeRoot {
        #[source]
        source: io::Error,
    },

    #[error("failed to run `{command}`")]
    Probe {
        command: String,
        #[source]
        source: io::Error,
    },

    #[error("{failed} of {total} checks failed; the report above says which")]
    DoctorFailed { failed: usize, total: usize },

    #[error(
        "{} carries one of the openspec-doc block markers and not the other; repair it by hand \
         rather than letting init append a second block",
        path.display()
    )]
    HalfDelimitedInstructions { path: PathBuf },
}
