use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::hook::Agent;
use crate::verdict::Verdict;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no OpenSpec project found: no openspec/config.yaml in {} or any parent directory", start.display())]
    NoProjectFound { start: PathBuf },

    #[error("not an OpenSpec project root: no config file at {}", config_path.display())]
    MissingConfig { config_path: PathBuf },

    #[error("path is not valid UTF-8: {}", path.display())]
    NonUtf8Path { path: PathBuf },

    #[error("failed to read {}", path.display())]
    Io {
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

    #[error("unknown agent `{value}`; expected one of {}", Agent::VALUES.join(", "))]
    UnknownAgent { value: String },

    #[error("failed to parse the {agent} hook payload from stdin")]
    HookPayload {
        agent: Agent,
        #[source]
        source: serde_json::Error,
    },

    #[error("malformed directive file {}", path.display())]
    Directive {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("invalid session id {session_id:?}: must not be empty or contain a path separator")]
    InvalidSessionId { session_id: String },

    #[error("malformed scratch snapshot file {}", path.display())]
    Snapshot {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to move {} to {}", from.display(), to.display())]
    Rename {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("selected text is required to anchor a comment")]
    EmptySelection,

    #[error("selected text {selected_text:?} was not found in {artifact_path}")]
    SelectionNotFound {
        artifact_path: String,
        selected_text: String,
    },

    #[error("invalid artifact path {artifact_path:?}: must be a path inside the project root")]
    InvalidArtifactPath { artifact_path: String },

    #[error("no artifact to comment on at {artifact_path}")]
    MissingArtifact { artifact_path: String },

    #[error("malformed comment sidecar {}", path.display())]
    CommentSidecar {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("no comment {comment_id} in {}", path.display())]
    UnknownComment { path: PathBuf, comment_id: String },

    #[error("refusing to relocate comments onto the existing sidecar {}", path.display())]
    SidecarExists { path: PathBuf },

    #[error(
        "a {verdict} verdict does not apply to {}",
        if *session_scoped { "a session" } else { "a change" }
    )]
    MisscopedVerdict {
        verdict: Verdict,
        session_scoped: bool,
    },

    #[error("a keep-exploring verdict needs notes saying what is still open")]
    EmptyVerdictNotes,

    #[error("malformed verdict sidecar {}", path.display())]
    VerdictSidecar {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to run `openspec validate {change}`")]
    Validate {
        change: String,
        #[source]
        source: io::Error,
    },
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn write(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Write {
            path: path.into(),
            source,
        }
    }
}
