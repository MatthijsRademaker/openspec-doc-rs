//! The route table: a page and an update stream per scoping regime —
//! `/sessions/<session_id>` for pre-proposal sessions, `/changes/<name>` for
//! active changes.

use std::convert::Infallible;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use openspec_doc_core::comments::{self};
use openspec_doc_core::verdict::{self, Verdict};
use openspec_doc_core::{Error as CoreError, Project};
use serde::Deserialize;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::error::{self, Error};
use crate::page::{Anchored, Review};
use crate::scope::Resolved;
use crate::watch::{Hub, Target};
use crate::{page, scope};

#[derive(Clone)]
struct AppState {
    project: Project,
    hub: Arc<Hub>,
}

pub fn router(project: Project) -> Router {
    let state = AppState {
        project,
        hub: Arc::new(Hub::default()),
    };

    Router::new()
        .route("/", get(index))
        .route("/sessions/{session_id}", get(session_page))
        .route("/sessions/{session_id}/events", get(session_events))
        .route("/sessions/{session_id}/review", get(session_review))
        .route("/sessions/{session_id}/comments", post(session_comment))
        .route("/sessions/{session_id}/verdict", post(session_verdict))
        .route("/changes/{name}", get(change_page))
        .route("/changes/{name}/events", get(change_events))
        .route("/changes/{name}/review", get(change_review))
        .route("/changes/{name}/comments", post(change_comment))
        .route("/changes/{name}/verdict", post(change_verdict))
        .fallback(unknown)
        .with_state(state)
}

/// A comment submitted from a page's composer. The offsets and context an anchor
/// needs are not accepted from the client: the server re-finds `selected_text` in
/// the artifact as it stands on disk, so an anchor is never recorded against
/// markdown only the browser believed was there.
#[derive(Debug, Deserialize)]
struct NewComment {
    artifact_path: String,
    selected_text: String,
    body: String,
}

/// A phase verdict submitted from a page's verdict controls.
#[derive(Debug, Deserialize)]
struct NewVerdict {
    verdict: Verdict,
    /// Absent for a verdict whose form carries no notes field.
    #[serde(default)]
    notes: String,
}

async fn index(State(state): State<AppState>) -> Result<Html<String>, RouteError> {
    let (sessions, changes) = scope::discovered(&state.project)?;

    Ok(Html(page::index(&sessions, &changes)))
}

async fn session_page(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Html<String>, RouteError> {
    let resolved = scope::session(&state.project, &session_id)?.ok_or(RouteError::NotFound)?;
    let review = review_state(&state, &resolved)?;

    Ok(Html(page::session(
        &session_id,
        resolved.title.as_deref(),
        &resolved.artifacts,
        &review,
    )))
}

async fn change_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Html<String>, RouteError> {
    let resolved = scope::change(&state.project, &name)?.ok_or(RouteError::NotFound)?;
    let review = review_state(&state, &resolved)?;

    Ok(Html(page::change(
        &name,
        resolved.title.as_deref(),
        &resolved.artifacts,
        &review,
    )))
}

async fn session_events(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, RouteError> {
    let resolved = scope::session(&state.project, &session_id)?.ok_or(RouteError::NotFound)?;

    Ok(stream(&state.hub, &resolved.target))
}

async fn change_events(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, RouteError> {
    let resolved = scope::change(&state.project, &name)?.ok_or(RouteError::NotFound)?;

    Ok(stream(&state.hub, &resolved.target))
}

/// The review-state fragment an open page refetches when its stream pushes, so
/// the comment list and verdict state update without the page being reloaded.
async fn session_review(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Html<String>, RouteError> {
    let resolved = scope::session(&state.project, &session_id)?.ok_or(RouteError::NotFound)?;

    Ok(Html(page::review_fragment(&review_state(
        &state, &resolved,
    )?)))
}

async fn change_review(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Html<String>, RouteError> {
    let resolved = scope::change(&state.project, &name)?.ok_or(RouteError::NotFound)?;

    Ok(Html(page::review_fragment(&review_state(
        &state, &resolved,
    )?)))
}

async fn session_comment(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Form(form): Form<NewComment>,
) -> Result<Redirect, RouteError> {
    let resolved = scope::session(&state.project, &session_id)?.ok_or(RouteError::NotFound)?;
    add_comment(&state, &resolved, &form)?;

    Ok(Redirect::to(&format!("/sessions/{session_id}")))
}

async fn change_comment(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Form(form): Form<NewComment>,
) -> Result<Redirect, RouteError> {
    let resolved = scope::change(&state.project, &name)?.ok_or(RouteError::NotFound)?;
    add_comment(&state, &resolved, &form)?;

    Ok(Redirect::to(&format!("/changes/{name}")))
}

async fn session_verdict(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Form(form): Form<NewVerdict>,
) -> Result<Redirect, RouteError> {
    let resolved = scope::session(&state.project, &session_id)?.ok_or(RouteError::NotFound)?;
    add_verdict(&state, &resolved, &form)?;

    Ok(Redirect::to(&format!("/sessions/{session_id}")))
}

async fn change_verdict(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Form(form): Form<NewVerdict>,
) -> Result<Redirect, RouteError> {
    let resolved = scope::change(&state.project, &name)?.ok_or(RouteError::NotFound)?;
    add_verdict(&state, &resolved, &form)?;

    Ok(Redirect::to(&format!("/changes/{name}")))
}

async fn unknown() -> RouteError {
    RouteError::NotFound
}

/// The scope's comment threads, each with where its anchor lands in the artifact
/// as it stands now, and its verdicts.
fn review_state(state: &AppState, resolved: &Resolved) -> Result<Review, RouteError> {
    let root = &state.project.root;
    let mut comments = Vec::new();

    for thread in comments::read(root, &resolved.key)? {
        let resolution = comments::resolve_anchor(root, &thread.comment.anchor)?;
        comments.push(Anchored { thread, resolution });
    }

    Ok(Review {
        comments,
        verdicts: verdict::read(root, &resolved.key)?,
    })
}

fn add_comment(state: &AppState, resolved: &Resolved, form: &NewComment) -> Result<(), RouteError> {
    comments::add(
        &state.project.root,
        &resolved.key,
        &form.artifact_path,
        &form.selected_text,
        &form.body,
    )?;

    Ok(())
}

fn add_verdict(state: &AppState, resolved: &Resolved, form: &NewVerdict) -> Result<(), RouteError> {
    verdict::add(
        &state.project.root,
        &resolved.key,
        form.verdict,
        &form.notes,
    )?;

    Ok(())
}

/// Push an event whenever `target` changes on disk. A subscriber that lagged is
/// told the same thing: it too needs to reload.
///
/// The response borrows neither argument (`use<>`): it outlives the handler.
fn stream(hub: &Hub, target: &Target) -> impl IntoResponse + use<> {
    let updates = hub.subscribe(target);
    let events = BroadcastStream::new(updates)
        .map(|_| Ok::<_, Infallible>(Event::default().data("changed")));

    Sse::new(events).keep_alive(KeepAlive::default())
}

enum RouteError {
    NotFound,
    /// The submission itself was not usable — a selection that is no longer in
    /// the artifact, or a verdict that does not apply to the scope. The reviewer
    /// is told what was wrong with it rather than being shown a server error.
    Rejected(CoreError),
    Internal(Error),
}

impl From<Error> for RouteError {
    fn from(error: Error) -> Self {
        match error {
            Error::Core(core) => Self::from(core),
            other => Self::Internal(other),
        }
    }
}

impl From<CoreError> for RouteError {
    fn from(error: CoreError) -> Self {
        // Everything a reviewer can get wrong by submitting a form, as against a
        // project the server failed to read.
        match error {
            CoreError::EmptySelection
            | CoreError::SelectionNotFound { .. }
            | CoreError::MissingArtifact { .. }
            | CoreError::InvalidArtifactPath { .. }
            | CoreError::MisscopedVerdict { .. }
            | CoreError::EmptyVerdictNotes => Self::Rejected(error),
            other => Self::Internal(Error::Core(other)),
        }
    }
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, Html(page::not_found())).into_response(),
            Self::Rejected(error) => (
                StatusCode::BAD_REQUEST,
                Html(page::rejected(&error.to_string())),
            )
                .into_response(),
            Self::Internal(error) => {
                eprintln!("error: {}", error::chain(&error));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(page::internal_error()),
                )
                    .into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::net::SocketAddr;
    use std::path::Path;
    use std::time::Duration;

    use openspec_doc_core::comments::ScopeKey;
    use openspec_doc_core::project_at;
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    const SESSION_ID: &str = "0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74";
    const CHANGE: &str = "add-a";

    /// Text present in both the scratch note and the change's proposal, and so
    /// selectable on either page.
    const SELECTED: &str = "An idea worth keeping.";

    /// A project with one active change carrying a proposal and a spec delta, and
    /// one session with a directive record and a scratch note.
    fn project_fixture() -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        let change_dir = temp.path().join("openspec/changes").join(CHANGE);
        let sessions = temp.path().join(".openspec-doc/directives/_session");
        let notes = temp.path().join(".openspec-doc/scratch/_session");

        fs::create_dir_all(change_dir.join("specs/some-cap")).expect("create change dir");
        fs::create_dir_all(&sessions).expect("create directives dir");
        fs::create_dir_all(&notes).expect("create scratch dir");
        fs::write(temp.path().join("openspec/config.yaml"), "").expect("write config");
        fs::write(
            sessions.join(format!("{SESSION_ID}.json")),
            r#"{"pending":false,"reason":"x","createdAt":"2026-07-31T08:00:00Z","consumedAt":null}"#,
        )
        .expect("write directive");
        fs::write(
            notes.join(format!("{SESSION_ID}.md")),
            format!("# Exploration\n\n{SELECTED}\n"),
        )
        .expect("write scratch note");
        fs::write(
            change_dir.join("proposal.md"),
            format!("## Why\n\n{SELECTED}\n"),
        )
        .expect("write proposal");
        fs::write(
            change_dir.join("specs/some-cap/spec.md"),
            "## ADDED Requirements\n\nThe system SHALL do a thing.\n",
        )
        .expect("write spec delta");
        temp
    }

    fn session_scope() -> ScopeKey {
        ScopeKey::Session(SESSION_ID.to_owned())
    }

    fn change_scope() -> ScopeKey {
        ScopeKey::Change(CHANGE.to_owned())
    }

    async fn serve(root: &Path) -> SocketAddr {
        let project = project_at(root).expect("project root");
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let address = listener.local_addr().expect("local addr");

        tokio::spawn(async move {
            axum::serve(listener, router(project)).await.expect("serve");
        });

        address
    }

    async fn get(address: SocketAddr, path: &str) -> TcpStream {
        let mut stream = TcpStream::connect(address).await.expect("connect");
        stream
            .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
            .await
            .expect("write request");
        stream
    }

    /// Read until `needle` shows up, so a streaming response can be inspected
    /// without waiting for a close that never comes.
    async fn read_until(stream: &mut TcpStream, needle: &str) -> String {
        let mut response = String::new();
        let mut buffer = [0_u8; 1024];

        while !response.contains(needle) {
            let read = stream.read(&mut buffer).await.expect("read response");
            assert_ne!(
                read, 0,
                "connection closed before {needle:?} in:\n{response}"
            );
            response.push_str(&String::from_utf8_lossy(&buffer[..read]));
        }

        response
    }

    /// One request/response exchange. `Connection: close` makes the server hang
    /// up at the end of the response, so a body of any shape — a page, a bare
    /// fragment, or a redirect with none at all — is read the same way.
    async fn exchange(address: SocketAddr, request: &str) -> String {
        let mut stream = TcpStream::connect(address).await.expect("connect");
        stream
            .write_all(request.as_bytes())
            .await
            .expect("write request");

        let mut response = Vec::new();
        tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut response))
            .await
            .expect("response before timeout")
            .expect("read response");

        String::from_utf8_lossy(&response).into_owned()
    }

    async fn fetch(address: SocketAddr, path: &str) -> String {
        exchange(
            address,
            &format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
        )
        .await
    }

    /// Post `body` as a form, the way the pages' own forms submit.
    async fn post(address: SocketAddr, path: &str, body: &str) -> String {
        exchange(
            address,
            &format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\
                 Content-Type: application/x-www-form-urlencoded\r\n\
                 Content-Length: {length}\r\n\r\n{body}",
                length = body.len()
            ),
        )
        .await
    }

    fn form(fields: &[(&str, &str)]) -> String {
        fields
            .iter()
            .map(|(name, value)| format!("{name}={}", encode(value)))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Percent-encode a form value. Only what the fixtures actually contain: the
    /// pages themselves rely on the browser to encode their forms.
    fn encode(value: &str) -> String {
        value
            .bytes()
            .map(|byte| match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (byte as char).to_string()
                }
                b' ' => "+".to_owned(),
                other => format!("%{other:02X}"),
            })
            .collect()
    }

    #[tokio::test]
    async fn discovered_sessions_and_changes_have_pages() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        for path in [
            "/",
            &format!("/changes/{CHANGE}"),
            &format!("/sessions/{SESSION_ID}"),
        ] {
            let response = fetch(address, path).await;
            assert!(
                response.starts_with("HTTP/1.1 200 OK"),
                "{path} did not serve a page:\n{response}"
            );
        }
    }

    #[tokio::test]
    async fn undiscovered_ids_and_names_are_not_found() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        for path in ["/changes/no-such-change", "/sessions/no-such-session"] {
            let response = fetch(address, path).await;
            assert!(
                response.starts_with("HTTP/1.1 404 Not Found"),
                "{path} was served instead of 404:\n{response}"
            );
        }
    }

    /// The point of the SSE endpoint: the client issues one request and the
    /// server pushes, rather than the client polling for a new version.
    #[tokio::test]
    async fn a_file_change_is_pushed_to_the_scopes_event_stream() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;
        let mut stream = get(address, &format!("/changes/{CHANGE}/events")).await;

        // The handler subscribes while building the response, so keep writing:
        // a single write could land before the watcher is registered.
        let file = fixture
            .path()
            .join("openspec/changes")
            .join(CHANGE)
            .join("proposal.md");
        let writes = tokio::spawn(async move {
            for revision in 0.. {
                fs::write(&file, format!("revision {revision}")).expect("write proposal");
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        });

        let pushed = tokio::time::timeout(
            Duration::from_secs(10),
            read_until(&mut stream, "data: changed"),
        )
        .await;
        writes.abort();

        assert!(pushed.is_ok(), "no event was pushed for the changed file");
    }

    #[tokio::test]
    async fn a_session_page_renders_its_scratch_note() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let response = fetch(address, &format!("/sessions/{SESSION_ID}")).await;

        assert!(
            response.contains(SELECTED),
            "the note's content:\n{response}"
        );
        assert!(
            response.contains(&format!(
                "data-artifact-path=\".openspec-doc/scratch/_session/{SESSION_ID}.md\""
            )),
            "the note is the commentable artifact:\n{response}"
        );
    }

    #[tokio::test]
    async fn a_change_page_renders_its_artifacts_and_spec_deltas() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let response = fetch(address, &format!("/changes/{CHANGE}")).await;

        for artifact in [
            &format!("openspec/changes/{CHANGE}/proposal.md"),
            &format!("openspec/changes/{CHANGE}/specs/some-cap/spec.md"),
        ] {
            assert!(
                response.contains(&format!("data-artifact-path=\"{artifact}\"")),
                "{artifact} was not rendered:\n{response}"
            );
        }
        assert!(
            response.contains("The system SHALL do a thing."),
            "the spec delta's content:\n{response}"
        );
    }

    #[tokio::test]
    async fn a_selection_submitted_from_a_session_page_becomes_an_anchored_comment() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let response = post(
            address,
            &format!("/sessions/{SESSION_ID}/comments"),
            &form(&[
                (
                    "artifact_path",
                    &format!(".openspec-doc/scratch/_session/{SESSION_ID}.md"),
                ),
                ("selected_text", SELECTED),
                ("body", "Which part is worth keeping?"),
            ]),
        )
        .await;

        assert!(
            response.starts_with("HTTP/1.1 303 See Other"),
            "the comment was not accepted:\n{response}"
        );
        let threads = comments::read(&fixture.path().canonicalize().unwrap(), &session_scope())
            .expect("read");
        let [thread] = threads.as_slice() else {
            panic!("expected one comment, got {}", threads.len());
        };
        assert_eq!(thread.comment.body, "Which part is worth keeping?");
        assert_eq!(
            thread.comment.anchor.selected_text, SELECTED,
            "the anchor is created from the selection"
        );
        assert_eq!(
            thread.comment.anchor.heading_path,
            ["Exploration"],
            "the anchor is derived from the raw markdown, not from the rendered page"
        );
    }

    #[tokio::test]
    async fn a_selection_submitted_from_a_change_page_becomes_an_anchored_comment() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        post(
            address,
            &format!("/changes/{CHANGE}/comments"),
            &form(&[
                (
                    "artifact_path",
                    &format!("openspec/changes/{CHANGE}/proposal.md"),
                ),
                ("selected_text", SELECTED),
                ("body", "Why is it worth keeping?"),
            ]),
        )
        .await;

        let threads =
            comments::read(&fixture.path().canonicalize().unwrap(), &change_scope()).expect("read");
        let [thread] = threads.as_slice() else {
            panic!("expected one comment, got {}", threads.len());
        };
        assert_eq!(thread.comment.body, "Why is it worth keeping?");
        assert_eq!(thread.comment.anchor.heading_path, ["Why"]);
    }

    /// The anchor is created against the file, not against what the browser
    /// believed was in it — so a selection the artifact has moved on from is
    /// refused outright rather than anchored to a guess.
    #[tokio::test]
    async fn a_selection_that_is_not_in_the_artifact_is_refused() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let response = post(
            address,
            &format!("/changes/{CHANGE}/comments"),
            &form(&[
                (
                    "artifact_path",
                    &format!("openspec/changes/{CHANGE}/proposal.md"),
                ),
                ("selected_text", "text that was never written"),
                ("body", "Body."),
            ]),
        )
        .await;

        assert!(
            response.starts_with("HTTP/1.1 400 Bad Request"),
            "a stale selection was accepted:\n{response}"
        );
        assert!(
            response.contains("was not found in"),
            "the reviewer is told what was wrong:\n{response}"
        );
        assert!(
            comments::read(&fixture.path().canonicalize().unwrap(), &change_scope())
                .expect("read")
                .is_empty(),
            "a refused selection records no comment"
        );
    }

    #[tokio::test]
    async fn a_keep_exploring_verdict_is_recorded_with_its_notes() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let response = post(
            address,
            &format!("/sessions/{SESSION_ID}/verdict"),
            &form(&[
                ("verdict", "keep-exploring"),
                ("notes", "Still unclear how promotion is detected."),
            ]),
        )
        .await;

        assert!(
            response.starts_with("HTTP/1.1 303 See Other"),
            "the verdict was not accepted:\n{response}"
        );
        let records =
            verdict::read(&fixture.path().canonicalize().unwrap(), &session_scope()).expect("read");
        let [record] = records.as_slice() else {
            panic!("expected one verdict, got {}", records.len());
        };
        assert_eq!(record.verdict, Verdict::KeepExploring);
        assert_eq!(record.notes, "Still unclear how promotion is detected.");
    }

    #[tokio::test]
    async fn a_move_to_proposal_verdict_is_recorded() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        post(
            address,
            &format!("/sessions/{SESSION_ID}/verdict"),
            &form(&[("verdict", "move-to-proposal"), ("notes", "")]),
        )
        .await;

        let records =
            verdict::read(&fixture.path().canonicalize().unwrap(), &session_scope()).expect("read");
        assert_eq!(
            records
                .iter()
                .map(|record| record.verdict)
                .collect::<Vec<_>>(),
            [Verdict::MoveToProposal]
        );
    }

    #[tokio::test]
    async fn a_send_to_agent_verdict_is_recorded_for_the_change() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        post(
            address,
            &format!("/changes/{CHANGE}/verdict"),
            &form(&[("verdict", "comment-resolution")]),
        )
        .await;

        let records =
            verdict::read(&fixture.path().canonicalize().unwrap(), &change_scope()).expect("read");
        assert_eq!(
            records
                .iter()
                .map(|record| record.verdict)
                .collect::<Vec<_>>(),
            [Verdict::CommentResolution]
        );
    }

    /// A verdict record is a session-level decision, so it must not be smuggled
    /// onto a scope it does not apply to by a hand-made request.
    #[tokio::test]
    async fn a_verdict_that_does_not_apply_to_the_scope_is_refused() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let response = post(
            address,
            &format!("/changes/{CHANGE}/verdict"),
            &form(&[("verdict", "keep-exploring"), ("notes", "Notes.")]),
        )
        .await;

        assert!(
            response.starts_with("HTTP/1.1 400 Bad Request"),
            "a misscoped verdict was accepted:\n{response}"
        );
        assert!(
            verdict::read(&fixture.path().canonicalize().unwrap(), &change_scope())
                .expect("read")
                .is_empty()
        );
    }

    /// What an already-open page refetches when its stream pushes: the comment
    /// list and verdict state on their own, without the page around them.
    #[tokio::test]
    async fn the_review_fragment_reflects_a_comment_added_after_the_page_was_served() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;
        let page = fetch(address, &format!("/changes/{CHANGE}")).await;
        assert!(page.contains("No comments yet."), "{page}");

        // Added the way the other tab's form would, while this page stays open.
        post(
            address,
            &format!("/changes/{CHANGE}/comments"),
            &form(&[
                (
                    "artifact_path",
                    &format!("openspec/changes/{CHANGE}/proposal.md"),
                ),
                ("selected_text", SELECTED),
                ("body", "Added from the other tab."),
            ]),
        )
        .await;
        let fragment = fetch(address, &format!("/changes/{CHANGE}/review")).await;

        assert!(
            fragment.contains("Added from the other tab."),
            "the fragment is stale:\n{fragment}"
        );
        assert!(
            !fragment.contains("<html"),
            "the fragment replaces one element, not the document:\n{fragment}"
        );
    }

    /// The other half of the live-update path: a comment written into the scope's
    /// sidecar has to push an event, or an open page never learns to refetch.
    #[tokio::test]
    async fn a_new_comment_is_pushed_to_the_scopes_event_stream() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;
        let mut stream = get(address, &format!("/changes/{CHANGE}/events")).await;

        // The handler subscribes while building the response, so keep posting: a
        // single comment could land before the watcher is registered.
        let comments = tokio::spawn(async move {
            for revision in 0.. {
                post(
                    address,
                    &format!("/changes/{CHANGE}/comments"),
                    &form(&[
                        (
                            "artifact_path",
                            &format!("openspec/changes/{CHANGE}/proposal.md"),
                        ),
                        ("selected_text", SELECTED),
                        ("body", &format!("comment {revision}")),
                    ]),
                )
                .await;
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        });

        let pushed = tokio::time::timeout(
            Duration::from_secs(10),
            read_until(&mut stream, "data: changed"),
        )
        .await;
        comments.abort();

        assert!(pushed.is_ok(), "no event was pushed for the new comment");
    }

    /// A verdict lands in its own sidecar, in a different directory from the
    /// comments, so it needs its own proof that the scope is watching it.
    #[tokio::test]
    async fn a_new_verdict_is_pushed_to_the_scopes_event_stream() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;
        let mut stream = get(address, &format!("/sessions/{SESSION_ID}/events")).await;

        let verdicts = tokio::spawn(async move {
            loop {
                post(
                    address,
                    &format!("/sessions/{SESSION_ID}/verdict"),
                    &form(&[("verdict", "keep-exploring"), ("notes", "Still open.")]),
                )
                .await;
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        });

        let pushed = tokio::time::timeout(
            Duration::from_secs(10),
            read_until(&mut stream, "data: changed"),
        )
        .await;
        verdicts.abort();

        assert!(pushed.is_ok(), "no event was pushed for the new verdict");
    }
}
