//! The landing page: every discovered session and active change, with enough of
//! each on its row to choose between them.
//!
//! A list of bare session ids is a list of coin flips. Every field here is
//! already on disk in the scratch, comment and verdict sidecars; the row reads
//! it rather than making the reviewer open each page to find out.

use std::time::SystemTime;

use crate::scope::Summary;

use super::{document, escape};

pub fn index(sessions: &[Summary], changes: &[Summary]) -> String {
    let body = format!(
        "<h1>openspec-doc</h1>\n{}\n{}",
        list("Sessions", "sessions", sessions),
        list("Changes", "changes", changes)
    );

    document("openspec-doc", &body, None)
}

fn list(heading: &str, prefix: &str, scopes: &[Summary]) -> String {
    if scopes.is_empty() {
        return format!("<h2>{heading}</h2>\n<p class=\"empty\">None discovered.</p>");
    }

    let rows: String = scopes.iter().map(|scope| row(prefix, scope)).collect();

    format!("<h2>{heading}</h2>\n<ul class=\"scopes\">\n{rows}</ul>")
}

/// One scope's row. The link is addressed by identifier and never by title: the
/// title is read from a file the agent rewrites, so a URL built from it would
/// break the moment the exploration's topic shifted.
fn row(prefix: &str, scope: &Summary) -> String {
    format!(
        "<li>\n<a href=\"/{prefix}/{key}\">{name}</a>\n<p class=\"meta\">{meta}</p>\n</li>\n",
        key = escape(&scope.key),
        name = escape(scope.title.as_deref().unwrap_or(&scope.key)),
        meta = meta(scope).join(" · ")
    )
}

/// The row's second line. The identifier is on it whether or not the row leads
/// with a title, because it is what an operator pastes into
/// `openspec-doc comment list`.
fn meta(scope: &Summary) -> Vec<String> {
    let mut parts = vec![format!("<code>{}</code>", escape(&scope.key))];

    if let Some(modified) = scope.modified {
        parts.push(format!("modified {}", age(modified)));
    }
    parts.push(match scope.open_comments {
        0 => "no open comments".to_owned(),
        1 => "1 open comment".to_owned(),
        count => format!("{count} open comments"),
    });
    if let Some(verdict) = scope.verdict {
        parts.push(format!("verdict: {verdict}"));
    }
    if scope.most_recently_active {
        // Deliberately not "live": nothing here says the session is still
        // running, only that it is the one that spoke to the reviewer last.
        parts.push("most recently active".to_owned());
    }

    parts
}

/// How long ago `at` was, at the granularity a reviewer is choosing on. Relative
/// so the page needs neither a timezone nor a date library, and because "20m ago"
/// is the actual question being asked of the timestamp.
fn age(at: SystemTime) -> String {
    // A file dated in the future is a clock that disagrees with itself, not an
    // error worth failing a page over.
    let Ok(elapsed) = SystemTime::now().duration_since(at) else {
        return "just now".to_owned();
    };

    let seconds = elapsed.as_secs();
    match seconds {
        0..60 => "just now".to_owned(),
        60..3_600 => format!("{}m ago", seconds / 60),
        3_600..86_400 => format!("{}h ago", seconds / 3_600),
        _ => format!("{}d ago", seconds / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openspec_doc_core::verdict::Verdict;

    const SESSION_ID: &str = "0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74";

    fn titled() -> Summary {
        Summary {
            key: SESSION_ID.to_owned(),
            title: Some("Exploring: the dashboard's look and feel".to_owned()),
            modified: Some(SystemTime::now()),
            open_comments: 2,
            verdict: Some(Verdict::KeepExploring),
            most_recently_active: true,
        }
    }

    fn untitled() -> Summary {
        Summary {
            key: SESSION_ID.to_owned(),
            title: None,
            modified: None,
            open_comments: 0,
            verdict: None,
            most_recently_active: false,
        }
    }

    #[test]
    fn a_titled_row_leads_with_its_title_and_still_shows_its_id() {
        let html = index(&[titled()], &[]);

        assert!(
            html.contains(">Exploring: the dashboard&#39;s look and feel</a>"),
            "{html}"
        );
        assert!(
            html.contains(&format!("<code>{SESSION_ID}</code>")),
            "the id is not on the row:\n{html}"
        );
        assert!(html.contains("2 open comments"), "{html}");
        assert!(html.contains("verdict: keep-exploring"), "{html}");
        assert!(html.contains("most recently active"), "{html}");
        assert!(
            !html.contains(">live<"),
            "there is no heartbeat behind this claim:\n{html}"
        );
    }

    /// The whole point of keeping the id in the URL: a title is display text read
    /// from a file the agent rewrites, and every link has to survive that.
    #[test]
    fn every_row_links_to_its_scope_by_identifier() {
        for summary in [titled(), untitled()] {
            let html = index(&[summary], &[]);

            assert!(
                html.contains(&format!("href=\"/sessions/{SESSION_ID}\"")),
                "{html}"
            );
        }
        assert!(
            index(&[], &[titled()]).contains(&format!("href=\"/changes/{SESSION_ID}\"")),
            "a change row is addressed the same way"
        );
    }

    #[test]
    fn an_untitled_row_leads_with_its_id_and_still_renders() {
        let html = index(&[untitled()], &[]);

        assert!(html.contains(&format!(">{SESSION_ID}</a>")), "{html}");
        assert!(html.contains("no open comments"), "{html}");
    }

    #[test]
    fn nothing_discovered_says_so_rather_than_rendering_an_empty_list() {
        let html = index(&[], &[]);

        assert!(html.contains("None discovered."), "{html}");
        assert!(!html.contains("<ul"), "{html}");
    }
}
