//! What a comment sidecar is made of: the events appended to it, and the thread
//! state reconstructed from replaying them.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::anchor::Anchor;

/// Where a comment stands. `Addressed` is a claim that work responding to it has
/// been done, which an agent can honestly make; `Resolved` is the reviewer's
/// judgement that the work is right, which only they make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Addressed,
    Resolved,
}

impl fmt::Display for Status {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Open => "open",
            Self::Addressed => "addressed",
            Self::Resolved => "resolved",
        })
    }
}

/// A comment, either anchored to a span of an artifact or scoped to the session
/// or change as a whole.
///
/// The anchor is absent rather than empty for an unanchored comment: not all
/// feedback is about a passage, and a blank selection would be a comment
/// claiming to point at nothing. A record written with an anchor reads back
/// unchanged, since a present field deserializes into the option as it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    #[serde(default)]
    pub anchor: Option<Anchor>,
    pub body: String,
    pub created_at: String,
}

/// Who added a response to a comment thread.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplyAuthor {
    Reviewer,
    /// Replies written before authorship was recorded came from the agent-facing
    /// CLI, so missing fields in existing sidecars retain that meaning.
    #[default]
    Agent,
}

/// A response added to an existing comment's thread.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub id: String,
    pub comment_id: String,
    #[serde(default)]
    pub author: ReplyAuthor,
    pub body: String,
    pub created_at: String,
}

/// A replacement for an existing comment's body.
///
/// Only a comment can be edited, never a reply: a reply is part of the
/// conversation's append-only record, and rewriting it would remove words one
/// participant already contributed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
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
    Edit { edit: Edit },
    Status { status: StatusUpdate },
    Relocate { relocate: Relocation },
}

/// A comment and everything that happened to it since, as replayed from a
/// sidecar. The comment's anchor reflects any relocation recorded after it, and
/// its body the last edit.
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

#[cfg(test)]
mod tests {
    use super::{Event, ReplyAuthor};

    #[test]
    fn a_reply_written_before_authorship_existed_remains_an_agent_reply() {
        let event: Event = serde_json::from_str(
            r#"{"type":"reply","reply":{"id":"reply-a","commentId":"comment-a","body":"Done.","createdAt":"2026-08-07T10:00:00Z"}}"#,
        )
        .expect("legacy reply");

        let Event::Reply { reply } = event else {
            panic!("expected reply event")
        };
        assert_eq!(reply.author, ReplyAuthor::Agent);
    }
}
