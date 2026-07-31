use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use openspec_doc_core::comments::ScopeKey;
use openspec_doc_core::hook::Agent;

/// Inspect, serve, and annotate an OpenSpec project's documentation.
#[derive(Debug, Parser)]
#[command(name = "openspec-doc", version, about)]
pub struct Cli {
    /// Project root to use instead of discovering one from the current directory
    #[arg(long, global = true, value_name = "PATH")]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print the resolved project root, its changes, and its specs
    Summary,

    /// Serve the documentation dashboard over HTTP
    Serve {
        /// Interface to bind
        #[arg(long, value_name = "HOST", default_value = "127.0.0.1")]
        host: String,

        /// Port to bind; omitted or `0` selects an available port
        #[arg(long, value_name = "PORT", default_value_t = 0)]
        port: u16,

        /// Print the URL without opening a browser
        #[arg(long)]
        no_open: bool,
    },

    /// Bridge agent hook events into the documentation dashboard
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },

    /// Manage anchored review comments on specs and changes
    Comment {
        #[command(subcommand)]
        command: CommentCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum CommentCommand {
    /// Anchor a new comment to selected text in an artifact
    Add {
        #[command(flatten)]
        scope: ScopeArgs,

        /// Artifact to comment on, relative to the project root
        #[arg(long, value_name = "PATH")]
        artifact: String,

        /// Text to anchor the comment to, exactly as it appears in the artifact
        #[arg(long, value_name = "TEXT")]
        selected_text: String,

        /// The comment itself
        #[arg(long, value_name = "TEXT")]
        body: String,
    },

    /// List a scope's comments with their status and anchor state
    List {
        #[command(flatten)]
        scope: ScopeArgs,
    },

    /// Append a reply to an existing comment
    Reply {
        #[command(flatten)]
        scope: ScopeArgs,

        /// Comment to reply to
        #[arg(long, value_name = "ID")]
        comment: String,

        /// The reply itself
        #[arg(long, value_name = "TEXT")]
        body: String,
    },

    /// Mark an existing comment resolved
    Resolve {
        #[command(flatten)]
        scope: ScopeArgs,

        /// Comment to resolve
        #[arg(long, value_name = "ID")]
        comment: String,
    },
}

/// Which comment sidecar to operate on: exactly one of the two keys it can be
/// filed under.
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct ScopeArgs {
    /// Change whose comments to use, for an artifact that has been formalized
    #[arg(long, value_name = "NAME")]
    pub change: Option<String>,

    /// Session whose comments to use, for an artifact still being explored
    #[arg(long, value_name = "ID")]
    pub session: Option<String>,
}

impl ScopeArgs {
    pub fn key(self) -> ScopeKey {
        match (self.change, self.session) {
            (Some(change), None) => ScopeKey::Change(change),
            (None, Some(session)) => ScopeKey::Session(session),
            // The argument group already requires exactly one of the two.
            other => unreachable!("clap admitted {other:?}"),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum HookCommand {
    /// Decide, from the agent's turn-end payload on stdin, whether its turn may
    /// end; prints that agent's decision on stdout
    Stop {
        /// Coding agent whose hook wire format to speak [possible values:
        /// claude, pi]
        #[arg(long, value_name = "AGENT")]
        agent: Agent,
    },
}
