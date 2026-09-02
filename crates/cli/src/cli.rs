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
        #[command(subcommand)]
        command: Option<ServeCommand>,

        /// Interface to bind
        #[arg(long, value_name = "HOST", default_value = "127.0.0.1")]
        host: String,

        /// Port to bind exactly; omitted binds the port assigned to this project
        #[arg(long, value_name = "PORT")]
        port: Option<u16>,

        /// Print the URL without opening a browser
        #[arg(long)]
        no_open: bool,

        /// Exit once no page is open and no hook has asked for the dashboard
        #[arg(long)]
        idle_exit: bool,
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

    /// Wire this project's agent hooks, extension, and instruction block up
    Init {
        /// Agent harness to configure, repeatable; any given replaces detection
        /// outright [possible values: claude, pi]
        #[arg(long, value_name = "AGENT")]
        agent: Vec<Agent>,

        /// Perform the planned writes; without it nothing is written
        #[arg(long)]
        yes: bool,

        /// Leave AGENTS.md alone
        #[arg(long)]
        skip_instructions: bool,
    },

    /// Run this project's configured agent hooks and report whether they work
    Doctor,

    /// Manage the exploration scratch note
    Scratch {
        #[command(subcommand)]
        command: ScratchCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ServeCommand {
    /// Print this project's dashboard URL and whether a dashboard is serving it
    Url,

    /// List every dashboard serving on the assignable port range, whatever
    /// project it serves, beside the ports assigned with nothing serving them
    List,

    /// Drop a project root's port assignment, freeing its port for another
    Forget {
        /// Project root to forget, canonical or not
        #[arg(value_name = "PATH")]
        root: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum ScratchCommand {
    /// Record which change a session's exploration became, so the next turn
    /// boundary promotes its scratch note onto that change
    Claim {
        /// Session whose exploration was formalized
        #[arg(long, value_name = "ID")]
        session: String,

        /// Change the exploration became
        #[arg(long, value_name = "NAME")]
        change: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum CommentCommand {
    /// Add a comment, anchored to selected text in an artifact or, with neither
    /// named, to the scope as a whole
    Add {
        #[command(flatten)]
        scope: ScopeArgs,

        /// Artifact to comment on, relative to the project root
        #[arg(long, value_name = "PATH", requires = "selected_text")]
        artifact: Option<String>,

        /// Text to anchor the comment to, exactly as it appears in the artifact
        #[arg(long, value_name = "TEXT", requires = "artifact")]
        selected_text: Option<String>,

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

    /// Replace an existing comment's body, keeping the original on the record
    Edit {
        #[command(flatten)]
        scope: ScopeArgs,

        /// Comment to edit
        #[arg(long, value_name = "ID")]
        comment: String,

        /// The body that replaces the current one
        #[arg(long, value_name = "TEXT")]
        body: String,
    },

    /// Mark an existing comment addressed: work responding to it has been done
    Address {
        #[command(flatten)]
        scope: ScopeArgs,

        /// Comment the work responded to
        #[arg(long, value_name = "ID")]
        comment: String,
    },

    /// Mark an existing comment resolved: the reviewer accepts the response
    Resolve {
        #[command(flatten)]
        scope: ScopeArgs,

        /// Comment to resolve
        #[arg(long, value_name = "ID")]
        comment: String,
    },

    /// Move an addressed or resolved comment back to open
    Reopen {
        #[command(flatten)]
        scope: ScopeArgs,

        /// Comment to reopen
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

    /// Deliver any standing review directive for the session named on stdin as
    /// context for the turn the submitted prompt starts; prints nothing when
    /// there is none, and never fails in a way that could refuse the prompt
    Prompt {
        /// Coding agent whose hook wire format to speak [possible values:
        /// claude, pi]
        #[arg(long, value_name = "AGENT")]
        agent: Agent,
    },

    /// Start an exploration for the session named on stdin: create its scratch
    /// note and print, for the agent to act on, where to keep the exploration
    Explore {
        /// Coding agent whose hook wire format to speak [possible values:
        /// claude, pi]
        #[arg(long, value_name = "AGENT")]
        agent: Agent,
    },
}
