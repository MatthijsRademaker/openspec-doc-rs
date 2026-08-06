//! The index's data as JSON: what the built frontend renders the index from.
//!
//! The fields are the ones the index has always shown, taken from the same
//! summaries the server-rendered index was built from — the derivation in
//! `scope` and in `core`'s scratch titles is unchanged, only the rendering
//! moved. Timestamps travel as RFC 3339 rather than as an epoch count, so a
//! `curl` of this endpoint is readable and the browser's `Date` parses it.

use std::time::SystemTime;

use chrono::{DateTime, SecondsFormat, Utc};
use openspec_doc_core::verdict::Verdict;
use serde::Serialize;

use crate::scope::Summary;

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

/// One scope as the index lists it. `key` is the scope's identity and its
/// address; `title` is display text and never either.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    key: String,
    title: Option<String>,
    /// `null` while none of the scope's artifacts exists — a session whose
    /// exploration has not started yet.
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

fn rfc3339(at: SystemTime) -> String {
    DateTime::<Utc>::from(at).to_rfc3339_opts(SecondsFormat::Secs, true)
}
