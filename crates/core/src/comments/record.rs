//! What a comment sidecar is made of: the events appended to it, and the thread
//! state reconstructed from replaying them.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::anchor::Anchor;

/// Whether a comment still wants attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Resolved,
}

/// A comment anchored to a span of an artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub anchor: Anchor,
    pub body: String,
    pub created_at: String,
}

/// A response added to an existing comment's thread.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub id: String,
    pub comment_id: String,
    pub body: String,
    pub created_at: String,
}

/// A change to an existing comment's status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusUpdate {
    pub id: String,
    pub comment_id: String,
    pub status: Status,
    pub created_at: String,
}

/// A record that the artifact every anchor names moved, written when a sidecar
/// follows its artifact's scope key from a session id to a change name. Without
/// it, the anchors in the relocated events would all point at the vacated path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relocation {
    pub from_artifact_path: String,
    pub to_artifact_path: String,
    pub created_at: String,
}

/// One line of a comment sidecar. The file is append-only, so the current state
/// of a comment is whatever replaying every event in order says it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Event {
    Comment { comment: Comment },
    Reply { reply: Reply },
    Status { status: StatusUpdate },
    Relocate { relocate: Relocation },
}

/// A comment and everything that happened to it since, as replayed from a
/// sidecar. The comment's anchor reflects any relocation recorded after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thread {
    pub comment: Comment,
    /// The status the last status update left the comment in.
    pub status: Status,
    pub replies: Vec<Reply>,
    pub status_history: Vec<StatusUpdate>,
}

/// An identifier for a new record, unique without having to read the sidecar
/// first — an append must not depend on what is already in the file.
pub(super) fn new_id() -> String {
    Uuid::new_v4().to_string()
}
