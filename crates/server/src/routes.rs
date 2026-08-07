//! Embedded frontend routes, scope JSON APIs, and scope update streams.

use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use openspec_doc_core::comments::{self, Comment, Reply, ReplyAuthor, Status, StatusUpdate};
use openspec_doc_core::verdict::{self, Record, Verdict};
use openspec_doc_core::{Error as CoreError, Project};
use serde::{Deserialize, Serialize};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::error::{self, Error};
use crate::scope::Resolved;
use crate::watch::{Hub, Target};
use crate::{api, assets, scope};

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
        .route("/", get(app))
        .route("/api/index", get(index_data))
        .route("/sessions/{session_id}", get(session_page))
        .route("/changes/{name}", get(change_page))
        .route("/api/sessions/{session_id}", get(session_detail))
        .route("/api/changes/{name}", get(change_detail))
        .route("/api/sessions/{session_id}/events", get(session_events))
        .route("/api/changes/{name}/events", get(change_events))
        .route("/api/sessions/{session_id}/comments", post(session_comment))
        .route("/api/changes/{name}/comments", post(change_comment))
        .route(
            "/api/sessions/{session_id}/comments/{comment_id}/replies",
            post(session_reply),
        )
        .route(
            "/api/changes/{name}/comments/{comment_id}/replies",
            post(change_reply),
        )
        .route(
            "/api/sessions/{session_id}/comments/{comment_id}/status",
            post(session_status),
        )
        .route(
            "/api/changes/{name}/comments/{comment_id}/status",
            post(change_status),
        )
        .route("/api/sessions/{session_id}/verdict", post(session_verdict))
        .route("/api/changes/{name}/verdict", post(change_verdict))
        .fallback(unknown)
        .with_state(state)
}

#[derive(Debug, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
enum NewComment {
    Anchored {
        artifact_path: String,
        selected_text: String,
        search_from: usize,
        body: String,
    },
    Unanchored {
        body: String,
    },
}

#[derive(Debug, Deserialize)]
struct NewReply {
    body: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ReviewerStatus {
    Open,
    Resolved,
}

impl From<ReviewerStatus> for Status {
    fn from(status: ReviewerStatus) -> Self {
        match status {
            ReviewerStatus::Open => Self::Open,
            ReviewerStatus::Resolved => Self::Resolved,
        }
    }
}

#[derive(Debug, Deserialize)]
struct NewStatus {
    status: ReviewerStatus,
}

#[derive(Debug, Deserialize)]
struct NewVerdict {
    verdict: Verdict,
}

async fn app() -> Response {
    assets::shell()
}

async fn index_data(State(state): State<AppState>) -> Result<Json<api::Index>, RouteError> {
    let (sessions, changes) = scope::discovered(&state.project)?;
    Ok(Json(api::Index::new(&sessions, &changes)))
}

async fn session_page(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Response, RouteError> {
    require_session(&state, &session_id)?;
    Ok(assets::shell())
}

async fn change_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, RouteError> {
    require_change(&state, &name)?;
    Ok(assets::shell())
}

async fn session_detail(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<api::Detail>, RouteError> {
    let resolved = require_session(&state, &session_id)?;
    Ok(Json(api::Detail::new(&state.project, &resolved)?))
}

async fn change_detail(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<api::Detail>, RouteError> {
    let resolved = require_change(&state, &name)?;
    Ok(Json(api::Detail::new(&state.project, &resolved)?))
}

async fn session_events(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, RouteError> {
    let resolved = require_session(&state, &session_id)?;
    Ok(stream(&state.hub, &resolved.target))
}

async fn change_events(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, RouteError> {
    let resolved = require_change(&state, &name)?;
    Ok(stream(&state.hub, &resolved.target))
}

async fn session_comment(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<NewComment>,
) -> Result<(StatusCode, Json<Comment>), RouteError> {
    let resolved = require_session(&state, &session_id)?;
    let comment = add_comment(&state, &resolved, request)?;
    Ok((StatusCode::CREATED, Json(comment)))
}

async fn change_comment(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<NewComment>,
) -> Result<(StatusCode, Json<Comment>), RouteError> {
    let resolved = require_change(&state, &name)?;
    let comment = add_comment(&state, &resolved, request)?;
    Ok((StatusCode::CREATED, Json(comment)))
}

async fn session_reply(
    State(state): State<AppState>,
    Path((session_id, comment_id)): Path<(String, String)>,
    Json(request): Json<NewReply>,
) -> Result<(StatusCode, Json<Reply>), RouteError> {
    let resolved = require_session(&state, &session_id)?;
    let reply = comments::reply_as(
        &state.project.root,
        &resolved.key,
        &comment_id,
        &request.body,
        ReplyAuthor::Reviewer,
    )?;
    Ok((StatusCode::CREATED, Json(reply)))
}

async fn change_reply(
    State(state): State<AppState>,
    Path((name, comment_id)): Path<(String, String)>,
    Json(request): Json<NewReply>,
) -> Result<(StatusCode, Json<Reply>), RouteError> {
    let resolved = require_change(&state, &name)?;
    let reply = comments::reply_as(
        &state.project.root,
        &resolved.key,
        &comment_id,
        &request.body,
        ReplyAuthor::Reviewer,
    )?;
    Ok((StatusCode::CREATED, Json(reply)))
}

async fn session_status(
    State(state): State<AppState>,
    Path((session_id, comment_id)): Path<(String, String)>,
    Json(request): Json<NewStatus>,
) -> Result<Json<StatusUpdate>, RouteError> {
    let resolved = require_session(&state, &session_id)?;
    let update = comments::set_status(
        &state.project.root,
        &resolved.key,
        &comment_id,
        request.status.into(),
    )?;
    Ok(Json(update))
}

async fn change_status(
    State(state): State<AppState>,
    Path((name, comment_id)): Path<(String, String)>,
    Json(request): Json<NewStatus>,
) -> Result<Json<StatusUpdate>, RouteError> {
    let resolved = require_change(&state, &name)?;
    let update = comments::set_status(
        &state.project.root,
        &resolved.key,
        &comment_id,
        request.status.into(),
    )?;
    Ok(Json(update))
}

async fn session_verdict(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<NewVerdict>,
) -> Result<(StatusCode, Json<Record>), RouteError> {
    let resolved = require_session(&state, &session_id)?;
    let record = verdict::add(&state.project.root, &resolved.key, request.verdict, "")?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn change_verdict(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<NewVerdict>,
) -> Result<(StatusCode, Json<Record>), RouteError> {
    let resolved = require_change(&state, &name)?;
    let record = verdict::add(&state.project.root, &resolved.key, request.verdict, "")?;
    Ok((StatusCode::CREATED, Json(record)))
}

fn require_session(state: &AppState, session_id: &str) -> Result<Resolved, RouteError> {
    scope::session(&state.project, session_id)?.ok_or(RouteError::NotFound)
}

fn require_change(state: &AppState, name: &str) -> Result<Resolved, RouteError> {
    scope::change(&state.project, name)?.ok_or(RouteError::NotFound)
}

fn add_comment(
    state: &AppState,
    resolved: &Resolved,
    request: NewComment,
) -> Result<Comment, RouteError> {
    match request {
        NewComment::Anchored {
            artifact_path,
            selected_text,
            search_from,
            body,
        } => Ok(comments::add(
            &state.project.root,
            &resolved.key,
            &artifact_path,
            &selected_text,
            search_from,
            &body,
        )?),
        NewComment::Unanchored { body } => Ok(comments::add_unanchored(
            &state.project.root,
            &resolved.key,
            &body,
        )?),
    }
}

async fn unknown(uri: Uri) -> Response {
    match assets::asset(uri.path().trim_start_matches('/')) {
        Some(response) => response,
        None => RouteError::NotFound.into_response(),
    }
}

fn stream(hub: &Hub, target: &Target) -> impl IntoResponse + use<> {
    let updates = hub.subscribe(target);
    let events = BroadcastStream::new(updates)
        .map(|_| Ok::<_, Infallible>(Event::default().data("changed")));

    Sse::new(events).keep_alive(KeepAlive::default())
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

enum RouteError {
    NotFound,
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
        match error {
            CoreError::EmptySelection
            | CoreError::SelectionNotFound { .. }
            | CoreError::MissingArtifact { .. }
            | CoreError::InvalidArtifactPath { .. }
            | CoreError::MisscopedVerdict { .. }
            | CoreError::UnknownComment { .. } => Self::Rejected(error),
            other => Self::Internal(Error::Core(other)),
        }
    }
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "scope or asset not found".to_owned(),
                }),
            )
                .into_response(),
            Self::Rejected(error) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    error: error.to_string(),
                }),
            )
                .into_response(),
            Self::Internal(error) => {
                eprintln!("error: {}", error::chain(&error));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "internal server error".to_owned(),
                    }),
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
    use std::path::Path as FsPath;
    use std::time::Duration;

    use openspec_doc_core::comments::{AnchorState, ScopeKey};
    use openspec_doc_core::{hook, project_at};
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    const SESSION_ID: &str = "0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74";
    const CHANGE: &str = "add-a";
    const REPEATED: &str = "Repeated review block.";

    fn project_fixture() -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        let change_dir = temp.path().join("openspec/changes").join(CHANGE);
        let directives = temp.path().join(".openspec-doc/directives/_session");
        let notes = temp.path().join(".openspec-doc/scratch/_session");

        fs::create_dir_all(change_dir.join("specs/some-cap")).expect("create change dir");
        fs::create_dir_all(&directives).expect("create directives dir");
        fs::create_dir_all(&notes).expect("create scratch dir");
        fs::write(temp.path().join("openspec/config.yaml"), "").expect("write config");
        fs::write(
            directives.join(format!("{SESSION_ID}.json")),
            r#"{"pending":false,"reason":"","createdAt":"2026-07-31T08:00:00Z","consumedAt":null}"#,
        )
        .expect("write directive");
        fs::write(
            notes.join(format!("{SESSION_ID}.md")),
            format!("# Exploration\n\n{REPEATED}\n\n{REPEATED}\n"),
        )
        .expect("write scratch note");
        fs::write(
            change_dir.join("proposal.md"),
            format!("## Why\n\n{REPEATED}\n\n{REPEATED}\n"),
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

    async fn serve(root: &FsPath) -> SocketAddr {
        let project = project_at(root).expect("project root");
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let address = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            axum::serve(listener, router(project)).await.expect("serve");
        });
        address
    }

    async fn exchange(address: SocketAddr, request: &str) -> String {
        let mut stream = TcpStream::connect(address).await.expect("connect");
        stream.write_all(request.as_bytes()).await.expect("write");
        let mut response = Vec::new();
        tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut response))
            .await
            .expect("response before timeout")
            .expect("read");
        String::from_utf8_lossy(&response).into_owned()
    }

    async fn fetch(address: SocketAddr, path: &str) -> String {
        exchange(
            address,
            &format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
        )
        .await
    }

    async fn fetch_json(address: SocketAddr, path: &str) -> serde_json::Value {
        response_json(&fetch(address, path).await, StatusCode::OK)
    }

    async fn post_json(
        address: SocketAddr,
        path: &str,
        body: serde_json::Value,
        status: StatusCode,
    ) -> serde_json::Value {
        let body = serde_json::to_string(&body).expect("encode body");
        let response = exchange(
            address,
            &format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\
                 Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            ),
        )
        .await;
        response_json(&response, status)
    }

    fn response_json(response: &str, status: StatusCode) -> serde_json::Value {
        assert!(
            response.starts_with(&format!("HTTP/1.1 {}", status.as_str())),
            "unexpected response:\n{response}"
        );
        let body = response.split_once("\r\n\r\n").expect("body").1;
        serde_json::from_str(body).unwrap_or_else(|error| panic!("{error} in {body}"))
    }

    #[tokio::test]
    async fn discovered_scopes_serve_the_shell_and_unknown_scopes_are_404() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        for path in [
            &format!("/sessions/{SESSION_ID}"),
            &format!("/changes/{CHANGE}"),
        ] {
            let response = fetch(address, path).await;
            assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
            assert!(response.contains("<div id=\"app\">"), "{response}");
        }
        for path in ["/api/sessions/no-such", "/api/changes/no-such"] {
            let response = fetch(address, path).await;
            assert!(response.starts_with("HTTP/1.1 404 Not Found"), "{response}");
        }
    }

    #[tokio::test]
    async fn scope_detail_contains_blocks_comments_counts_and_verdict_delivery() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let second = fs::read_to_string(root.join("openspec/changes/add-a/proposal.md"))
            .expect("read")
            .rfind(REPEATED)
            .expect("second");
        comments::add(
            &root,
            &change_scope(),
            "openspec/changes/add-a/proposal.md",
            REPEATED,
            second,
            "Second occurrence only.",
        )
        .expect("comment");
        let record =
            verdict::add(&root, &change_scope(), Verdict::CommentResolution, "").expect("verdict");
        verdict::mark_translated(&root, &change_scope(), &record.id).expect("translated");
        fs::write(
            root.join(".openspec-doc/scratch/_session")
                .join(format!("{SESSION_ID}.md")),
            "<!-- openspec-doc:moved-to .openspec-doc/scratch/add-a.md -->\n",
        )
        .expect("record promotion");
        let pending = hook::write_pending(&root, SESSION_ID, "Review change.").expect("pending");
        hook::mark_consumed(&root, SESSION_ID, &pending).expect("consume");

        let address = serve(&root).await;
        let detail = fetch_json(address, &format!("/api/changes/{CHANGE}")).await;

        assert_eq!(detail["kind"], "change");
        assert!(detail["artifacts"][0]["blocks"].as_array().is_some());
        assert_eq!(detail["comments"][0]["anchorState"], "exact");
        assert!(detail["comments"][0]["blockId"].is_string());
        assert_eq!(detail["commentCounts"]["open"], 1);
        assert_eq!(detail["standingVerdict"]["directiveDelivered"], true);
    }

    #[tokio::test]
    async fn scopes_with_no_artifacts_or_comments_return_empty_arrays() {
        let fixture = project_fixture();
        fs::remove_file(
            fixture
                .path()
                .join(".openspec-doc/scratch/_session")
                .join(format!("{SESSION_ID}.md")),
        )
        .expect("remove note");
        let address = serve(fixture.path()).await;

        let session = fetch_json(address, &format!("/api/sessions/{SESSION_ID}")).await;
        let change = fetch_json(address, &format!("/api/changes/{CHANGE}")).await;
        assert_eq!(session["artifacts"].as_array().map(Vec::len), Some(0));
        assert_eq!(session["comments"].as_array().map(Vec::len), Some(0));
        assert_eq!(change["comments"].as_array().map(Vec::len), Some(0));
    }

    #[tokio::test]
    async fn anchored_and_unanchored_comment_handlers_use_the_core_writers() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;
        let artifact = format!(".openspec-doc/scratch/_session/{SESSION_ID}.md");
        let markdown = fs::read_to_string(fixture.path().join(&artifact)).expect("read");
        let second = markdown.rfind(REPEATED).expect("second");

        post_json(
            address,
            &format!("/api/sessions/{SESSION_ID}/comments"),
            serde_json::json!({
                "kind": "anchored",
                "artifactPath": artifact,
                "selectedText": REPEATED,
                "searchFrom": second,
                "body": "Second block."
            }),
            StatusCode::CREATED,
        )
        .await;
        post_json(
            address,
            &format!("/api/sessions/{SESSION_ID}/comments"),
            serde_json::json!({
                "kind": "anchored",
                "artifactPath": artifact,
                "selectedText": "review",
                "searchFrom": second,
                "body": "Selected word."
            }),
            StatusCode::CREATED,
        )
        .await;
        post_json(
            address,
            &format!("/api/sessions/{SESSION_ID}/comments"),
            serde_json::json!({"kind": "unanchored", "body": "Whole scope."}),
            StatusCode::CREATED,
        )
        .await;

        let root = fixture.path().canonicalize().unwrap();
        let threads = comments::read(&root, &session_scope()).expect("read");
        assert_eq!(threads.len(), 3);
        assert_eq!(
            threads[0]
                .comment
                .anchor
                .as_ref()
                .expect("anchor")
                .start_offset,
            second
        );
        assert_eq!(
            comments::resolve_anchor(&root, threads[1].comment.anchor.as_ref())
                .expect("resolve selection")
                .state,
            AnchorState::Exact
        );
        assert_eq!(threads[2].comment.anchor, None);
    }

    #[tokio::test]
    async fn reply_and_reviewer_status_handlers_update_one_thread() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let comment =
            comments::add_unanchored(&root, &change_scope(), "Question.").expect("comment");
        let address = serve(&root).await;

        post_json(
            address,
            &format!("/api/changes/{CHANGE}/comments/{}/replies", comment.id),
            serde_json::json!({"body": "Response."}),
            StatusCode::CREATED,
        )
        .await;
        post_json(
            address,
            &format!("/api/changes/{CHANGE}/comments/{}/status", comment.id),
            serde_json::json!({"status": "resolved"}),
            StatusCode::OK,
        )
        .await;

        let thread = comments::read(&root, &change_scope())
            .expect("read")
            .remove(0);
        assert_eq!(thread.replies[0].body, "Response.");
        assert_eq!(thread.replies[0].author, ReplyAuthor::Reviewer);
        assert_eq!(thread.status, Status::Resolved);
    }

    #[tokio::test]
    async fn dashboard_status_handler_cannot_set_addressed() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let comment =
            comments::add_unanchored(&root, &change_scope(), "Question.").expect("comment");
        let address = serve(&root).await;
        let body =
            serde_json::to_string(&serde_json::json!({"status": "addressed"})).expect("body");
        let response = exchange(
            address,
            &format!(
                "POST /api/changes/{CHANGE}/comments/{}/status HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                comment.id,
                body.len()
            ),
        )
        .await;

        assert!(response.starts_with("HTTP/1.1 422"), "{response}");
        assert_eq!(
            comments::read(&root, &change_scope()).expect("read")[0].status,
            Status::Open
        );
    }

    #[tokio::test]
    async fn verdict_handlers_record_scope_appropriate_verdicts_without_notes() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        post_json(
            address,
            &format!("/api/sessions/{SESSION_ID}/verdict"),
            serde_json::json!({"verdict": "move-to-proposal"}),
            StatusCode::CREATED,
        )
        .await;
        post_json(
            address,
            &format!("/api/changes/{CHANGE}/verdict"),
            serde_json::json!({"verdict": "comment-resolution"}),
            StatusCode::CREATED,
        )
        .await;

        let root = fixture.path().canonicalize().expect("root");
        assert_eq!(
            verdict::read(&root, &session_scope()).expect("read")[0].notes,
            ""
        );
        assert_eq!(
            verdict::read(&root, &change_scope()).expect("read")[0].notes,
            ""
        );
    }

    #[tokio::test]
    async fn stale_selection_is_refused_with_its_reason_as_json() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let error = post_json(
            address,
            &format!("/api/changes/{CHANGE}/comments"),
            serde_json::json!({
                "kind": "anchored",
                "artifactPath": "openspec/changes/add-a/proposal.md",
                "selectedText": "not in source",
                "searchFrom": 0,
                "body": "Why?"
            }),
            StatusCode::BAD_REQUEST,
        )
        .await;

        assert!(error["error"].as_str().unwrap().contains("was not found"));
    }

    #[tokio::test]
    async fn comment_status_changes_are_pushed_to_the_event_stream() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let comment =
            comments::add_unanchored(&root, &change_scope(), "Question.").expect("comment");
        let address = serve(&root).await;
        let mut stream = TcpStream::connect(address).await.expect("connect");
        stream
            .write_all(
                format!("GET /api/changes/{CHANGE}/events HTTP/1.1\r\nHost: localhost\r\n\r\n")
                    .as_bytes(),
            )
            .await
            .expect("write");

        let updates = tokio::spawn(async move {
            for _ in 0..20 {
                let _ = post_json(
                    address,
                    &format!("/api/changes/{CHANGE}/comments/{}/status", comment.id),
                    serde_json::json!({"status": "resolved"}),
                    StatusCode::OK,
                )
                .await;
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        });
        let mut response = String::new();
        let mut buffer = [0_u8; 1024];
        let pushed = tokio::time::timeout(Duration::from_secs(10), async {
            while !response.contains("data: changed") {
                let read = stream.read(&mut buffer).await.expect("read");
                assert_ne!(read, 0, "stream closed: {response}");
                response.push_str(&String::from_utf8_lossy(&buffer[..read]));
            }
        })
        .await;
        updates.abort();

        assert!(pushed.is_ok(), "no event pushed: {response}");
    }

    #[test]
    fn exact_resolution_is_exposed_for_created_block_comment() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let artifact = "openspec/changes/add-a/proposal.md";
        let markdown = fs::read_to_string(root.join(artifact)).expect("read");
        let second = markdown.rfind(REPEATED).expect("second");
        let comment = comments::add(&root, &change_scope(), artifact, REPEATED, second, "Here.")
            .expect("comment");
        let resolution = comments::resolve_anchor(&root, comment.anchor.as_ref()).expect("resolve");
        assert_eq!(resolution.state, AnchorState::Exact);
        assert_eq!(resolution.offset, Some(second));
    }
}
