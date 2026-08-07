//! JSON representations consumed by the embedded Vue dashboard.

use std::time::SystemTime;

use chrono::{DateTime, SecondsFormat, Utc};
use openspec_doc_core::Project;
use openspec_doc_core::comments::{self, AnchorState, Comment, Reply, ScopeKey, Status};
use openspec_doc_core::hook;
use openspec_doc_core::scratch;
use openspec_doc_core::verdict::{self, Record, Verdict};
use serde::Serialize;

use crate::error::Error;
use crate::markdown;
use crate::scope::{Resolved, Summary};

/// Every discovered scope, split by the regime that addresses it.
#[derive(Serialize)]
pub struct Index {
    pub sessions: Vec<Scope>,
    pub changes: Vec<Scope>,
}

impl Index {
    pub fn new(sessions: &[Summary], changes: &[Summary]) -> Self {
        Self {
            sessions: sessions.iter().map(Scope::from).collect(),
            changes: changes.iter().map(Scope::from).collect(),
        }
    }
}

/// One scope as the index lists it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    key: String,
    title: Option<String>,
    modified_at: Option<String>,
    open_comments: usize,
    verdict: Option<Verdict>,
    most_recently_active: bool,
}

impl From<&Summary> for Scope {
    fn from(summary: &Summary) -> Self {
        Self {
            key: summary.key.clone(),
            title: summary.title.clone(),
            modified_at: summary.modified.map(rfc3339),
            open_comments: summary.open_comments,
            verdict: summary.verdict,
            most_recently_active: summary.most_recently_active,
        }
    }
}

/// Complete review state for one session or change.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    kind: ScopeKind,
    key: String,
    title: Option<String>,
    artifacts: Vec<Artifact>,
    comments: Vec<Thread>,
    comment_counts: CommentCounts,
    verdicts: Vec<Record>,
    standing_verdict: Option<StandingVerdict>,
}

impl Detail {
    pub fn new(project: &Project, resolved: &Resolved) -> Result<Self, Error> {
        let artifacts: Vec<Artifact> = resolved
            .artifacts
            .iter()
            .map(|artifact| Artifact {
                path: artifact.path.clone(),
                blocks: markdown::blocks(&artifact.markdown),
            })
            .collect();

        let mut threads = Vec::new();
        for thread in comments::read(&project.root, &resolved.key)? {
            let resolution =
                comments::resolve_anchor(&project.root, thread.comment.anchor.as_ref())?;
            let block_id = thread
                .comment
                .anchor
                .as_ref()
                .zip(resolution.offset)
                .and_then(|(anchor, offset)| {
                    artifacts
                        .iter()
                        .find(|artifact| artifact.path == anchor.artifact_path)
                        .and_then(|artifact| {
                            artifact
                                .blocks
                                .iter()
                                .find(|block| {
                                    block.range.start <= offset && offset < block.range.end
                                })
                                .map(|block| block.id.clone())
                        })
                });

            threads.push(Thread {
                comment: thread.comment,
                status: thread.status,
                replies: thread.replies,
                anchor_state: resolution.state,
                block_id,
            });
        }

        let counts = comments::counts(&project.root, &resolved.key)?;
        let verdicts = verdict::read(&project.root, &resolved.key)?;
        let standing_verdict = standing_verdict(project, &resolved.key, verdicts.last())?;
        let (kind, key) = match &resolved.key {
            ScopeKey::Session(session_id) => (ScopeKind::Session, session_id.clone()),
            ScopeKey::Change(name) => (ScopeKind::Change, name.clone()),
        };

        Ok(Self {
            kind,
            key,
            title: resolved.title.clone(),
            artifacts,
            comments: threads,
            comment_counts: CommentCounts {
                open: counts.open,
                addressed: counts.addressed,
                resolved: counts.resolved,
            },
            verdicts,
            standing_verdict,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum ScopeKind {
    Session,
    Change,
}

#[derive(Serialize)]
pub struct Artifact {
    path: String,
    blocks: Vec<markdown::Block>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    comment: Comment,
    status: Status,
    replies: Vec<Reply>,
    anchor_state: AnchorState,
    block_id: Option<String>,
}

#[derive(Serialize)]
pub struct CommentCounts {
    open: usize,
    addressed: usize,
    resolved: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandingVerdict {
    id: String,
    verdict: Verdict,
    created_at: String,
    directive_delivered: bool,
    directive_pending: bool,
}

fn standing_verdict(
    project: &Project,
    key: &ScopeKey,
    latest: Option<&Record>,
) -> Result<Option<StandingVerdict>, Error> {
    let Some(latest) = latest else {
        return Ok(None);
    };

    let untranslated = verdict::untranslated(&project.root, key)?.is_some();
    let directive = if untranslated {
        None
    } else if let Some(session_id) = directive_session(project, key)? {
        hook::load_directive(&project.root, &session_id)?
    } else {
        None
    };
    let directive_pending = directive.as_ref().is_some_and(|record| record.pending);
    let directive_delivered = directive
        .as_ref()
        .is_some_and(|record| !record.pending && record.consumed_at.is_some());

    Ok(Some(StandingVerdict {
        id: latest.id.clone(),
        verdict: latest.verdict,
        created_at: latest.created_at.clone(),
        directive_delivered,
        directive_pending,
    }))
}

/// Session whose hook consumes this scope's verdict. Change verdicts return to
/// the session whose exploration was promoted into that change.
fn directive_session(project: &Project, key: &ScopeKey) -> Result<Option<String>, Error> {
    match key {
        ScopeKey::Session(session_id) => Ok(Some(session_id.clone())),
        ScopeKey::Change(name) => {
            for session_id in hook::sessions(&project.root)? {
                if scratch::promoted_to(&project.root, &session_id)?.as_deref() == Some(name) {
                    return Ok(Some(session_id));
                }
            }
            Ok(None)
        }
    }
}

fn rfc3339(at: SystemTime) -> String {
    DateTime::<Utc>::from(at).to_rfc3339_opts(SecondsFormat::Secs, true)
}
