//! Anchored review comments: a comment attached to a span of markdown, and the
//! append-only sidecar the comments for one scope are recorded in.
//!
//! The commented artifact is any markdown file under the project root, named by
//! its relative path — an explore-phase scratch note and a change's `proposal.md`
//! are treated identically. A sidecar is keyed by the same scope key as the
//! artifact it discusses, and [`relocate`] moves it when that key changes.

mod anchor;
mod artifact;
mod record;
mod sidecar;

/// Shared so a scratch note's title is the same thing a comment anchor calls a
/// heading, rather than a second opinion about what markdown means.
pub(crate) use anchor::parse_heading;
pub use anchor::{Anchor, AnchorState, Resolution};
pub use artifact::{read as read_artifact, resolve as resolve_anchor};
pub use record::{Comment, Edit, Event, Relocation, Reply, Status, StatusUpdate, Thread};
pub use sidecar::{
    ScopeKey, StatusCounts, add, add_unanchored, counts, edit, read, relocate, reply, set_status,
};

/// The sidecar directory holding every scope's comment stream.
const COMMENTS_DIR: &str = ".openspec-doc/comments";

/// Session-keyed sidecars sit one level deeper, so a session id can never
/// collide with a change name.
const SESSION_DIR: &str = "_session";
