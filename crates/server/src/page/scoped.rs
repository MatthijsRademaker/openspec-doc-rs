//! The two scoped pages. Both render their artifacts, the shared review-state
//! fragment, and the same select-to-comment composer; they differ only in which
//! verdicts a reviewer can reach from them.

use crate::scope::Artifact;

use super::review::Review;
use super::{Endpoints, artifact, escape, review};

/// A session's page: its scratch note, and the phase verdict that decides
/// whether the exploration keeps going or gets formalized.
pub fn session(
    session_id: &str,
    title: Option<&str>,
    artifacts: &[Artifact],
    state: &Review,
) -> String {
    let endpoints = endpoints("sessions", session_id);
    let body = format!(
        "{heading}{artifacts}{composer}{review}{verdicts}",
        heading = heading("Session", session_id, title),
        artifacts = artifact::list(artifacts),
        composer = composer(&endpoints.comments),
        review = review::fragment(state),
        verdicts = session_verdicts(&endpoints.verdict)
    );

    super::document(title.unwrap_or(session_id), &body, Some(&endpoints))
}

/// A change's page: its proposal, design, tasks and spec deltas, and the verdict
/// that sends its open comments back to the agent.
pub fn change(name: &str, title: Option<&str>, artifacts: &[Artifact], state: &Review) -> String {
    let endpoints = endpoints("changes", name);
    let body = format!(
        "{heading}{artifacts}{composer}{review}{verdicts}",
        heading = heading("Change", name, title),
        artifacts = artifact::list(artifacts),
        composer = composer(&endpoints.comments),
        review = review::fragment(state),
        verdicts = change_verdicts(&endpoints.verdict)
    );

    super::document(title.unwrap_or(name), &body, Some(&endpoints))
}

/// The page's heading: the scope's title where it has one, with the identifier
/// underneath it either way — the id is what gets pasted into
/// `openspec-doc comment list`, so it stays on the page once the title leads.
fn heading(kind: &str, key: &str, title: Option<&str>) -> String {
    format!(
        "<h1>{name}</h1>\n<p class=\"meta\">{kind} <code>{key}</code></p>\n",
        name = escape(title.unwrap_or(key)),
        key = escape(key)
    )
}

fn endpoints(prefix: &str, key: &str) -> Endpoints {
    Endpoints {
        events: format!("/{prefix}/{key}/events"),
        review: format!("/{prefix}/{key}/review"),
        comments: format!("/{prefix}/{key}/comments"),
        verdict: format!("/{prefix}/{key}/verdict"),
    }
}

/// The select-to-comment composer: a plain form the client fills the selection
/// into and reveals. It posts and redirects like any form, so the comment is
/// recorded whether or not the selection script ran.
fn composer(action: &str) -> String {
    format!(
        "<form class=\"composer\" method=\"post\" action=\"{action}\" hidden data-composer>\n\
         <h2>Comment on the selection</h2>\n\
         <p class=\"meta\" data-composer-context></p>\n\
         <input type=\"hidden\" name=\"artifact_path\" data-composer-artifact required>\n\
         <input type=\"hidden\" name=\"selected_text\" data-composer-selection required>\n\
         <textarea name=\"body\" rows=\"4\" required placeholder=\"What about it?\"></textarea>\n\
         <button type=\"submit\">Add comment</button>\n\
         <button type=\"button\" data-composer-cancel>Cancel</button>\n\
         </form>\n",
        action = escape(action)
    )
}

/// A session's two phase verdicts. Keep-exploring carries the reviewer's open
/// questions, which is the whole content of that verdict, so its notes are
/// required by the form as well as by the record.
fn session_verdicts(action: &str) -> String {
    format!(
        "<h2>Phase verdict</h2>\n\
         <form class=\"verdict\" method=\"post\" action=\"{action}\">\n\
         <input type=\"hidden\" name=\"verdict\" value=\"keep-exploring\">\n\
         <label>Open questions or resolutions\n\
         <textarea name=\"notes\" rows=\"4\" required></textarea></label>\n\
         <button type=\"submit\">Keep exploring</button>\n\
         </form>\n\
         <form class=\"verdict\" method=\"post\" action=\"{action}\">\n\
         <input type=\"hidden\" name=\"verdict\" value=\"move-to-proposal\">\n\
         <button type=\"submit\">Move to proposal</button>\n\
         </form>\n",
        action = escape(action)
    )
}

fn change_verdicts(action: &str) -> String {
    format!(
        "<h2>Phase verdict</h2>\n\
         <form class=\"verdict\" method=\"post\" action=\"{action}\">\n\
         <input type=\"hidden\" name=\"verdict\" value=\"comment-resolution\">\n\
         <button type=\"submit\">Send to agent: address open comments</button>\n\
         </form>\n",
        action = escape(action)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note() -> Vec<Artifact> {
        vec![Artifact {
            path: ".openspec-doc/scratch/_session/session-a.md".to_owned(),
            markdown: "# Exploration\n\nAn idea worth keeping.\n".to_owned(),
        }]
    }

    fn empty() -> Review {
        Review {
            comments: Vec::new(),
            verdicts: Vec::new(),
        }
    }

    #[test]
    fn a_session_page_renders_its_note_and_both_phase_verdicts() {
        let html = session("session-a", None, &note(), &empty());

        assert!(html.contains("An idea worth keeping."), "{html}");
        assert!(html.contains("value=\"keep-exploring\""), "{html}");
        assert!(html.contains("value=\"move-to-proposal\""), "{html}");
        assert!(
            !html.contains("value=\"comment-resolution\""),
            "a session has no change to resolve comments on: {html}"
        );
    }

    #[test]
    fn a_change_page_offers_only_the_comment_resolution_verdict() {
        let html = change("add-a", None, &[], &empty());

        assert!(html.contains("value=\"comment-resolution\""), "{html}");
        assert!(!html.contains("value=\"keep-exploring\""), "{html}");
    }

    #[test]
    fn a_titled_page_leads_with_its_title_and_keeps_its_id_on_the_page() {
        let html = session("session-a", Some("Exploring: titles"), &note(), &empty());

        assert!(html.contains("<h1>Exploring: titles</h1>"), "{html}");
        assert!(html.contains("<title>Exploring: titles</title>"), "{html}");
        assert!(
            html.contains("Session <code>session-a</code>"),
            "the id is what gets pasted into the CLI and has to stay:\n{html}"
        );
    }

    #[test]
    fn an_untitled_page_leads_with_its_id() {
        let html = session("session-a", None, &note(), &empty());

        assert!(html.contains("<h1>session-a</h1>"), "{html}");
        assert!(html.contains("Session <code>session-a</code>"), "{html}");
    }

    #[test]
    fn both_pages_post_their_comments_to_their_own_scope() {
        assert!(
            session("session-a", None, &note(), &empty())
                .contains("action=\"/sessions/session-a/comments\""),
        );
        assert!(
            change("add-a", None, &[], &empty()).contains("action=\"/changes/add-a/comments\"")
        );
    }

    #[test]
    fn both_pages_subscribe_to_their_own_event_and_review_endpoints() {
        let html = change("add-a", None, &[], &empty());

        assert!(
            html.contains("data-events=\"/changes/add-a/events\""),
            "{html}"
        );
        assert!(
            html.contains("data-review=\"/changes/add-a/review\""),
            "{html}"
        );
    }
}
