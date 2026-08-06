//! The review state both scoped pages show: the scope's comment threads and the
//! verdicts recorded for it.
//!
//! This is one fragment rather than two so that a live update is one refetch,
//! and it is rendered by the same function whether it is being embedded in a
//! full page or served on its own to a page that is already open.

use openspec_doc_core::comments::{AnchorState, Resolution, Thread};
use openspec_doc_core::verdict::Record;

use super::escape;

/// The element a page's review state lives in, and the one the client swaps when
/// an update arrives.
pub(super) const CONTAINER_ID: &str = "review-state";

/// A comment thread together with where its anchor lands in the artifact as that
/// artifact stands right now.
pub struct Anchored {
    pub thread: Thread,
    pub resolution: Resolution,
}

/// Everything about a scope that a review changes: its comment threads and the
/// verdicts recorded for it.
pub struct Review {
    pub comments: Vec<Anchored>,
    pub verdicts: Vec<Record>,
}

/// The scope's comments and verdicts, wrapped in the element the client swaps.
pub fn fragment(review: &Review) -> String {
    format!(
        "<div id=\"{CONTAINER_ID}\">\n{}\n{}</div>\n",
        comments(&review.comments),
        recorded_verdicts(&review.verdicts)
    )
}

fn comments(anchored: &[Anchored]) -> String {
    if anchored.is_empty() {
        return "<h2>Comments</h2>\n<p class=\"empty\">No comments yet.</p>".to_owned();
    }

    let items: String = anchored
        .iter()
        .map(|anchored| comment(&anchored.thread, &anchored.resolution))
        .collect();

    format!(
        "<h2>Comments ({count})</h2>\n<ul class=\"comments\">\n{items}</ul>",
        count = anchored.len()
    )
}

/// One comment thread, reporting how confidently its anchor still resolves —
/// a comment whose text has been rewritten out from under it is shown as such
/// rather than presented as if it were still firmly placed.
///
/// A comment scoped to the session or change as a whole has no artifact to name
/// and no passage to quote, so it renders with neither rather than with an empty
/// path and an empty quotation.
fn comment(thread: &Thread, resolution: &Resolution) -> String {
    let status = thread.status;
    let replies: String = thread
        .replies
        .iter()
        .map(|reply| format!("<li>{}</li>\n", escape(&reply.body)))
        .collect();

    format!(
        "<li class=\"comment\" data-anchor-state=\"{state}\">\n\
         <p class=\"meta\">{where_}{status} · anchor {state}{at}</p>\n\
         {quotation}\
         <p class=\"body\">{body}</p>\n\
         {replies}</li>\n",
        state = resolution.state,
        where_ = match &thread.comment.anchor {
            Some(anchor) => format!("{} · ", escape(&anchor.artifact_path)),
            None => String::new(),
        },
        at = match resolution.offset {
            Some(offset) if resolution.state != AnchorState::Exact =>
                format!(" at offset {offset}"),
            _ => String::new(),
        },
        quotation = match &thread.comment.anchor {
            Some(anchor) => format!(
                "<blockquote>{}</blockquote>\n",
                escape(&anchor.selected_text)
            ),
            None => String::new(),
        },
        body = escape(&thread.comment.body),
        replies = if replies.is_empty() {
            String::new()
        } else {
            format!("<ul class=\"replies\">\n{replies}</ul>\n")
        }
    )
}

fn recorded_verdicts(verdicts: &[Record]) -> String {
    if verdicts.is_empty() {
        return "<h2>Verdicts</h2>\n<p class=\"empty\">No verdict submitted yet.</p>".to_owned();
    }

    // Most recent first: the last verdict submitted is the one that stands.
    let items: String = verdicts
        .iter()
        .rev()
        .map(|record| {
            format!(
                "<li><strong>{verdict}</strong> <span class=\"meta\">{created}</span>{notes}</li>\n",
                verdict = record.verdict,
                created = escape(&record.created_at),
                notes = if record.notes.is_empty() {
                    String::new()
                } else {
                    format!("\n<p class=\"body\">{}</p>", escape(&record.notes))
                }
            )
        })
        .collect();

    format!("<h2>Verdicts</h2>\n<ul class=\"verdicts\">\n{items}</ul>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use openspec_doc_core::comments::{Anchor, Status};
    use openspec_doc_core::verdict::Verdict;

    fn thread(body: &str, selected: &str, status: Status) -> Thread {
        Thread {
            comment: openspec_doc_core::comments::Comment {
                id: "c1".to_owned(),
                anchor: Some(Anchor {
                    artifact_path: "note.md".to_owned(),
                    selected_text: selected.to_owned(),
                    heading_path: vec!["Why".to_owned()],
                    before_text: String::new(),
                    after_text: String::new(),
                    start_offset: 0,
                    end_offset: selected.len(),
                }),
                body: body.to_owned(),
                created_at: "2026-07-31T08:00:00Z".to_owned(),
            },
            status,
            replies: Vec::new(),
            status_history: Vec::new(),
        }
    }

    /// A comment scoped to the session or change rather than to a passage of it.
    fn unanchored_thread(body: &str) -> Thread {
        let mut thread = thread(body, "unused", Status::Open);
        thread.comment.id = "c2".to_owned();
        thread.comment.anchor = None;
        thread
    }

    fn exact() -> Resolution {
        Resolution {
            state: AnchorState::Exact,
            offset: Some(0),
        }
    }

    fn review(comments: Vec<Anchored>, verdicts: Vec<Record>) -> Review {
        Review { comments, verdicts }
    }

    fn record(verdict: Verdict, notes: &str) -> Record {
        Record {
            id: "v1".to_owned(),
            verdict,
            notes: notes.to_owned(),
            created_at: "2026-07-31T08:00:00Z".to_owned(),
        }
    }

    #[test]
    fn a_comment_renders_its_anchor_state_status_and_body() {
        let html = fragment(&review(
            vec![Anchored {
                thread: thread("Needs a rationale.", "Because of X.", Status::Open),
                resolution: exact(),
            }],
            vec![],
        ));

        assert!(html.contains("data-anchor-state=\"exact\""), "{html}");
        assert!(html.contains("note.md · open · anchor exact"), "{html}");
        assert!(html.contains("Because of X."), "{html}");
        assert!(html.contains("Needs a rationale."), "{html}");
    }

    /// An anchor the artifact has moved out from under is reported as such: the
    /// point of the state is that it is not hidden from the reviewer.
    #[test]
    fn a_fuzzy_anchor_reports_where_it_landed() {
        let html = fragment(&review(
            vec![Anchored {
                thread: thread("Body.", "Selected.", Status::Open),
                resolution: Resolution {
                    state: AnchorState::Fuzzy,
                    offset: Some(42),
                },
            }],
            vec![],
        ));

        assert!(html.contains("anchor fuzzy at offset 42"), "{html}");
    }

    /// This page is on its way out, but not yet, and a scope holding both kinds
    /// of comment must render rather than take the reviewer's dashboard down.
    #[test]
    fn an_anchored_and_an_unanchored_comment_both_render() {
        let html = fragment(&review(
            vec![
                Anchored {
                    thread: thread("Needs a rationale.", "Because of X.", Status::Open),
                    resolution: exact(),
                },
                Anchored {
                    thread: unanchored_thread("The whole framing is off."),
                    resolution: Resolution {
                        state: AnchorState::Unanchored,
                        offset: None,
                    },
                },
            ],
            vec![],
        ));

        assert!(html.contains("Comments (2)"), "{html}");
        assert!(html.contains("note.md · open · anchor exact"), "{html}");
        assert!(html.contains("Needs a rationale."), "{html}");
        assert!(html.contains("The whole framing is off."), "{html}");
        assert!(
            html.contains("<p class=\"meta\">open · anchor unanchored</p>"),
            "an unanchored comment claims no artifact: {html}"
        );
        assert!(
            html.matches("<blockquote>").count() == 1,
            "an unanchored comment quotes nothing: {html}"
        );
    }

    #[test]
    fn a_verdict_renders_with_its_notes() {
        let html = fragment(&review(
            vec![],
            vec![record(Verdict::KeepExploring, "Still unclear.")],
        ));

        assert!(html.contains("keep-exploring"), "{html}");
        assert!(html.contains("Still unclear."), "{html}");
    }

    #[test]
    fn the_most_recent_verdict_is_listed_first() {
        let html = fragment(&review(
            vec![],
            vec![
                record(Verdict::KeepExploring, "One more pass."),
                record(Verdict::MoveToProposal, ""),
            ],
        ));

        let latest = html.find("move-to-proposal").expect("latest verdict");
        let earlier = html.find("keep-exploring").expect("earlier verdict");
        assert!(latest < earlier, "the standing verdict leads, got {html}");
    }

    #[test]
    fn an_empty_scope_says_so_for_both_comments_and_verdicts() {
        let html = fragment(&review(vec![], vec![]));

        assert!(html.contains("No comments yet."), "{html}");
        assert!(html.contains("No verdict submitted yet."), "{html}");
    }

    /// Comment bodies and verdict notes are reviewer-supplied text.
    #[test]
    fn reviewer_supplied_text_is_escaped() {
        let html = fragment(&review(
            vec![Anchored {
                thread: thread("<script>alert(1)</script>", "sel", Status::Open),
                resolution: exact(),
            }],
            vec![record(Verdict::KeepExploring, "<img onerror=x>")],
        ));

        assert!(!html.contains("<script>"), "{html}");
        assert!(!html.contains("<img"), "{html}");
    }
}
