//! Embedded frontend routes, scope JSON APIs, and scope update streams.

use std::convert::Infallible;
use std::path::Path as FsPath;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use openspec_doc_core::approval;
use openspec_doc_core::comments::{
    self, Comment, Reply, ReplyAuthor, ScopeKey, Status, StatusUpdate,
};
use openspec_doc_core::dashboard::Identity;
use openspec_doc_core::verdict::{self, Record, Verdict};
use openspec_doc_core::{Error as CoreError, Project};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::{BroadcastStream, WatchStream};

use crate::error::{self, Error};
use crate::idle::Activity;
use crate::scope::Resolved;
use crate::shutdown::Stop;
use crate::watch::{Hub, Target, Update};
use crate::{api, assets, scope};

#[derive(Clone)]
struct AppState {
    project: Project,
    hub: Arc<Hub>,
    activity: Arc<Activity>,
    stop: Arc<Stop>,
}

pub fn router(project: Project, hub: Arc<Hub>, activity: Arc<Activity>, stop: Arc<Stop>) -> Router {
    let state = AppState {
        project,
        hub,
        activity,
        stop,
    };

    Router::new()
        .route("/", get(app))
        .route(openspec_doc_core::dashboard::IDENTITY_PATH, get(identity))
        // `post` only, so a link, a prefetch, or an address bar cannot reach it.
        .route(openspec_doc_core::dashboard::SHUTDOWN_PATH, post(shutdown))
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
        .route("/api/changes/{name}/approval", post(change_approval))
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

/// What the reviewer asked to do about a change's approval.
///
/// One route rather than the browser firing N status posts and then an approve:
/// a partial sequence from the client leaves the reviewer looking at a change
/// that is neither settled nor approved, and emits one live update per comment
/// for a single intent.
#[derive(Debug, Deserialize)]
#[serde(tag = "act", rename_all = "kebab-case")]
enum ApprovalAct {
    Approve,
    ResolveAllAndApprove,
    Withdraw,
}

/// What an approval submission did, beside the state it left behind.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApprovalOutcome {
    /// How many comments a resolve-all-and-approve moved to `resolved`. Zero for
    /// the acts that resolve nothing.
    resolved: usize,
    approval: api::Approval,
}

async fn app() -> Response {
    assets::shell()
}

/// Report which project this dashboard serves and the process serving it, and
/// count the request as activity.
///
/// Discovery compares canonical roots, so the comparison is exact rather than
/// heuristic. The process id is reported because it is the only thing that makes
/// an unwanted dashboard killable once discovery has found it.
///
/// The turn-end hook probes this route to decide whether to start a dashboard, so
/// a heartbeat and a start probe are the same request — which is what lets the
/// idle deadline know a session is still working with every tab closed.
async fn identity(State(state): State<AppState>) -> Json<Identity> {
    state.activity.register();

    Json(Identity {
        root: state.project.root.clone(),
        pid: std::process::id(),
    })
}

/// Begin a graceful shutdown, if the caller named the root this dashboard serves.
///
/// The port is the dashboard's identity in a way a pid is not, and the dashboard
/// is the only participant that knows for certain which project it serves — so
/// the caller says which one it believes it is stopping and this refuses when
/// that is somebody else. That closes the case the signal design could not: the
/// target port having changed hands between the enumeration that found it and
/// the request, because the dashboard that was there exited and another project's
/// fell forward onto its port.
///
/// A missing header is refused for a second reason. No cross-origin HTML form
/// can set one, so requiring it is what stops a page in the operator's own
/// browser from posting a dashboard down on a guessable local port.
///
/// The answer goes out before the process exits, not after: axum's graceful
/// shutdown lets an in-flight response finish, and a caller handed a dropped
/// connection cannot tell a shutdown that started from a request that never
/// arrived.
async fn shutdown(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let header = openspec_doc_core::dashboard::SHUTDOWN_ROOT_HEADER;
    let Some(named) = headers.get(header) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: format!("{header} is required: name the project root to stop"),
            }),
        )
            .into_response();
    };

    // Lossy on purpose: a root that does not survive the trip is not this
    // dashboard's root either, and the refusal below says exactly that.
    let named = String::from_utf8_lossy(named.as_bytes());
    if FsPath::new(named.as_ref()) != state.project.root {
        return (
            StatusCode::CONFLICT,
            Json(ErrorBody {
                error: format!(
                    "this dashboard serves {}, not {named}: nothing was stopped",
                    state.project.root.display()
                ),
            }),
        )
            .into_response();
    }

    state.stop.request();

    StatusCode::ACCEPTED.into_response()
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
    Ok(stream(&state.hub, &resolved.target, &state.stop))
}

async fn change_events(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, RouteError> {
    let resolved = require_change(&state, &name)?;
    Ok(stream(&state.hub, &resolved.target, &state.stop))
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

/// Approve a change, withdraw its approval, or resolve its outstanding feedback
/// and approve it in one submission.
///
/// Every refusal comes back as a rejection carrying its reason, because an
/// approval that was not recorded and said nothing about why is the failure this
/// gate exists to avoid: the reviewer would read the unchanged state as the
/// button not having worked.
async fn change_approval(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<ApprovalAct>,
) -> Result<Json<ApprovalOutcome>, RouteError> {
    let resolved = require_change(&state, &name)?;
    let ScopeKey::Change(change) = &resolved.key else {
        unreachable!("a change scope is keyed by its name");
    };

    let resolved_count = match request {
        ApprovalAct::Approve => {
            approval::submit(&state.project, change)?;
            0
        }
        ApprovalAct::ResolveAllAndApprove => {
            approval::resolve_all_and_approve(&state.project, change)?.resolved
        }
        ApprovalAct::Withdraw => {
            approval::withdraw(&state.project, change)?;
            0
        }
    };

    Ok(Json(ApprovalOutcome {
        resolved: resolved_count,
        approval: api::Approval::read(&state.project, change)?,
    }))
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

/// A scope's live updates, until the scope stops changing or the server stops.
///
/// The stop is not decoration. axum's graceful shutdown waits for in-flight
/// responses, and an event stream is a response that never finishes on its own —
/// so without ending it here, a shutdown with a review page open would answer,
/// begin shutting down, and then sit there until the browser gave up. Ending the
/// stream is also what makes the page report the connection going away instead
/// of quietly waiting for the next event forever.
fn stream(hub: &Hub, target: &Target, stop: &Stop) -> impl IntoResponse + use<> {
    let updates = BroadcastStream::new(hub.subscribe(target)).map(Some);
    // A watch stream yields the value it currently holds first, so the `false` a
    // running server sits on is filtered out and only the flip reaches the merge.
    let stopping = WatchStream::new(stop.receiver())
        .filter(|requested| *requested)
        .map(|_| None);

    let events = updates.merge(stopping).map_while(|update| {
        let update = update?.unwrap_or_else(|_| Update::all());
        Some(Ok::<_, Infallible>(
            Event::default()
                .json_data(update)
                .expect("live update payload is serializable"),
        ))
    });

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
            | CoreError::UnfingerprintedApproval
            | CoreError::UnknownChange { .. }
            | CoreError::ApprovalBlocked { .. }
            | CoreError::NothingToWithdraw { .. }
            | CoreError::PartialApproval { .. }
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
    use std::time::Duration;

    use openspec_doc_core::comments::AnchorState;
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
            axum::serve(
                listener,
                router(
                    project,
                    Arc::new(Hub::default()),
                    Arc::default(),
                    Arc::default(),
                ),
            )
            .await
            .expect("serve");
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

    /// A change page reports its approval state, and a change nobody approved
    /// reports as not approved rather than as approved-by-default.
    #[tokio::test]
    async fn a_change_detail_carries_its_approval_state() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let detail = fetch_json(address, &format!("/api/changes/{CHANGE}")).await;

        assert_eq!(detail["approval"]["state"], "not-approved");
        assert!(
            detail["approval"]["reason"]
                .as_str()
                .expect("a reason")
                .contains("no approval"),
            "{detail}"
        );
    }

    /// A session is an exploration rather than a change, and there is nothing
    /// there to approve.
    #[tokio::test]
    async fn a_session_detail_has_no_approval_state() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let detail = fetch_json(address, &format!("/api/sessions/{SESSION_ID}")).await;

        assert_eq!(detail["approval"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn approving_a_settled_change_records_it_and_reports_the_new_state() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let outcome = post_json(
            address,
            &format!("/api/changes/{CHANGE}/approval"),
            serde_json::json!({ "act": "approve" }),
            StatusCode::OK,
        )
        .await;

        assert_eq!(outcome["resolved"], 0);
        assert_eq!(outcome["approval"]["state"], "approved");
        assert_eq!(
            fetch_json(address, &format!("/api/changes/{CHANGE}")).await["approval"]["state"],
            "approved"
        );
    }

    /// An approval that was not recorded and said nothing about why is the
    /// failure this gate exists to avoid: the reviewer reads the unchanged state
    /// as the button not having worked.
    #[tokio::test]
    async fn approving_over_outstanding_feedback_is_refused_with_its_reason() {
        let fixture = project_fixture();
        comments::add_unanchored(fixture.path(), &change_scope(), "Look at this.")
            .expect("add comment");
        let address = serve(fixture.path()).await;

        let refusal = post_json(
            address,
            &format!("/api/changes/{CHANGE}/approval"),
            serde_json::json!({ "act": "approve" }),
            StatusCode::BAD_REQUEST,
        )
        .await;

        let error = refusal["error"].as_str().expect("a reason");
        assert!(error.contains("1 open"), "{error}");
        assert_eq!(
            fetch_json(address, &format!("/api/changes/{CHANGE}")).await["approval"]["state"],
            "not-approved",
            "a refused approval records nothing"
        );
    }

    #[tokio::test]
    async fn resolve_all_and_approve_settles_every_thread_and_approves() {
        let fixture = project_fixture();
        let open = comments::add_unanchored(fixture.path(), &change_scope(), "Never answered.")
            .expect("add comment");
        let claimed = comments::add_unanchored(fixture.path(), &change_scope(), "Claimed done.")
            .expect("add comment");
        comments::set_status(
            fixture.path(),
            &change_scope(),
            &claimed.id,
            Status::Addressed,
        )
        .expect("address");
        let address = serve(fixture.path()).await;

        let outcome = post_json(
            address,
            &format!("/api/changes/{CHANGE}/approval"),
            serde_json::json!({ "act": "resolve-all-and-approve" }),
            StatusCode::OK,
        )
        .await;

        assert_eq!(outcome["resolved"], 2);
        assert_eq!(outcome["approval"]["state"], "approved");
        let detail = fetch_json(address, &format!("/api/changes/{CHANGE}")).await;
        assert_eq!(detail["commentCounts"]["open"], 0);
        assert_eq!(detail["commentCounts"]["addressed"], 0);
        assert_eq!(detail["commentCounts"]["resolved"], 2);
        assert_eq!(detail["comments"][0]["comment"]["id"], open.id);
    }

    #[tokio::test]
    async fn withdrawing_returns_the_change_to_not_approved() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;
        post_json(
            address,
            &format!("/api/changes/{CHANGE}/approval"),
            serde_json::json!({ "act": "approve" }),
            StatusCode::OK,
        )
        .await;

        let outcome = post_json(
            address,
            &format!("/api/changes/{CHANGE}/approval"),
            serde_json::json!({ "act": "withdraw" }),
            StatusCode::OK,
        )
        .await;

        assert_eq!(outcome["approval"]["state"], "not-approved");
    }

    /// A reviewed artifact edited after approval leaves the page saying stale
    /// and naming what moved, rather than saying approved about content nobody
    /// approved.
    #[tokio::test]
    async fn an_edited_artifact_makes_the_rendered_state_stale() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;
        post_json(
            address,
            &format!("/api/changes/{CHANGE}/approval"),
            serde_json::json!({ "act": "approve" }),
            StatusCode::OK,
        )
        .await;

        fs::write(
            fixture
                .path()
                .join("openspec/changes")
                .join(CHANGE)
                .join("proposal.md"),
            "## Why\n\nA different reason.\n",
        )
        .expect("edit proposal");

        let approval =
            fetch_json(address, &format!("/api/changes/{CHANGE}")).await["approval"].clone();
        assert_eq!(approval["state"], "stale");
        assert_eq!(approval["changedArtifacts"][0], "proposal.md");
    }

    /// The verdict route takes every other kind. An approval submitted through
    /// it would carry no fingerprint, so nothing could tell it from one whose
    /// artifacts were rewritten afterwards.
    #[tokio::test]
    async fn the_verdict_route_refuses_an_approval() {
        let fixture = project_fixture();
        let address = serve(fixture.path()).await;

        let refusal = post_json(
            address,
            &format!("/api/changes/{CHANGE}/verdict"),
            serde_json::json!({ "verdict": "approved" }),
            StatusCode::BAD_REQUEST,
        )
        .await;

        assert!(
            refusal["error"]
                .as_str()
                .expect("a reason")
                .contains("fingerprint"),
            "{refusal}"
        );
    }

    /// Discovery reuses a dashboard when this route names its own root, so the
    /// form the root arrives in is what makes the comparison exact.
    #[tokio::test]
    async fn the_identity_route_names_the_canonical_root_and_the_process() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let address = serve(&root).await;

        let identity = fetch_json(address, "/api/identity").await;

        assert_eq!(identity["root"], root.display().to_string());
        assert_eq!(identity["pid"], std::process::id());
    }

    /// A server wired the way `run` wires one: its shutdown route resolves the
    /// graceful shutdown, so a stop is observable here rather than only asserted
    /// about.
    async fn serve_stoppable(root: &FsPath) -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let project = project_at(root).expect("project root");
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let address = listener.local_addr().expect("local addr");
        let stop = Arc::new(Stop::default());
        let app = router(
            project,
            Arc::new(Hub::default()),
            Arc::default(),
            stop.clone(),
        );
        let served = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move { stop.requested().await })
                .await
                .expect("serve");
        });

        (address, served)
    }

    /// A shutdown request naming `root`, spelled the way the CLI spells it.
    fn shutdown_naming(root: &str) -> String {
        format!(
            "POST {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\
             {}: {root}\r\nContent-Length: 0\r\n\r\n",
            openspec_doc_core::dashboard::SHUTDOWN_PATH,
            openspec_doc_core::dashboard::SHUTDOWN_ROOT_HEADER,
        )
    }

    /// That the dashboard is still there, which is the assertion every refusal
    /// needs beside the status code.
    async fn assert_still_serving(address: SocketAddr, root: &FsPath) {
        let identity = fetch_json(address, openspec_doc_core::dashboard::IDENTITY_PATH).await;
        assert_eq!(identity["root"], root.display().to_string());
    }

    /// The route's whole job, and what lets `serve kill` report off a probe
    /// rather than off having sent a request: the port stops answering.
    #[tokio::test]
    async fn a_shutdown_naming_this_root_answers_and_then_stops_the_server() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let (address, served) = serve_stoppable(&root).await;

        let response = exchange(address, &shutdown_naming(&root.display().to_string())).await;

        assert!(
            response.starts_with("HTTP/1.1 202"),
            "the answer has to come before the exit: {response}"
        );
        tokio::time::timeout(Duration::from_secs(5), served)
            .await
            .expect("the server never stopped")
            .expect("the server panicked");
        assert!(
            TcpStream::connect(address).await.is_err(),
            "the port still answers after the shutdown"
        );
    }

    /// The fall-forward race: this port held another project's dashboard by the
    /// time the request arrived, and only the server could have noticed.
    #[tokio::test]
    async fn a_shutdown_naming_another_root_is_refused_and_the_server_keeps_serving() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let (address, served) = serve_stoppable(&root).await;

        let response = exchange(address, &shutdown_naming("/repos/somebody-else")).await;

        assert!(response.starts_with("HTTP/1.1 409"), "{response}");
        assert_still_serving(address, &root).await;
        served.abort();
    }

    /// A link, a prefetch, or an address bar must not reach it.
    #[tokio::test]
    async fn a_get_to_the_shutdown_route_is_rejected() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let (address, served) = serve_stoppable(&root).await;

        let response = fetch(address, openspec_doc_core::dashboard::SHUTDOWN_PATH).await;

        assert!(
            response.starts_with("HTTP/1.1 405"),
            "a GET stopped the dashboard: {response}"
        );
        assert_still_serving(address, &root).await;
        served.abort();
    }

    /// The request a cross-origin HTML form could actually have sent: a `POST`
    /// with no header on it. Refusing it is what keeps a page in the operator's
    /// browser from stopping a dashboard on a guessable local port.
    #[tokio::test]
    async fn a_shutdown_without_the_root_header_is_rejected() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let (address, served) = serve_stoppable(&root).await;

        let response = exchange(
            address,
            &format!(
                "POST {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\
                 Content-Type: text/plain\r\nContent-Length: 0\r\n\r\n",
                openspec_doc_core::dashboard::SHUTDOWN_PATH
            ),
        )
        .await;

        assert!(
            response.starts_with("HTTP/1.1 400"),
            "a form-shaped POST stopped the dashboard: {response}"
        );
        assert!(
            response.contains(openspec_doc_core::dashboard::SHUTDOWN_ROOT_HEADER),
            "the refusal must say what was missing: {response}"
        );
        assert_still_serving(address, &root).await;
        served.abort();
    }

    /// The failure this would otherwise have: axum's graceful shutdown waits for
    /// in-flight responses, and an event stream is a response that never
    /// finishes on its own — so a stop with a review page open would answer and
    /// then sit there until the browser gave up.
    #[tokio::test]
    async fn an_open_event_stream_ends_rather_than_holding_the_shutdown_open() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        let (address, served) = serve_stoppable(&root).await;
        let mut page = TcpStream::connect(address).await.expect("connect");
        page.write_all(
            format!("GET /api/changes/{CHANGE}/events HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .as_bytes(),
        )
        .await
        .expect("write");
        let mut head = [0_u8; 1024];
        let read = tokio::time::timeout(Duration::from_secs(5), page.read(&mut head))
            .await
            .expect("the event stream never started")
            .expect("read");
        assert!(
            String::from_utf8_lossy(&head[..read]).starts_with("HTTP/1.1 200"),
            "the page is not subscribed, so this test proves nothing"
        );

        let response = exchange(address, &shutdown_naming(&root.display().to_string())).await;

        assert!(response.starts_with("HTTP/1.1 202"), "{response}");
        tokio::time::timeout(Duration::from_secs(5), served)
            .await
            .expect("an open event stream held the shutdown open")
            .expect("the server panicked");
        // And the page is told, rather than left waiting for an event that is
        // never coming.
        let mut rest = Vec::new();
        tokio::time::timeout(Duration::from_secs(5), page.read_to_end(&mut rest))
            .await
            .expect("the page was never told the connection ended")
            .expect("read");
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
            while !response
                .contains("data: {\"artifactsChanged\":false,\"reviewStateChanged\":true}")
            {
                let read = stream.read(&mut buffer).await.expect("read");
                assert_ne!(read, 0, "stream closed: {response}");
                response.push_str(&String::from_utf8_lossy(&buffer[..read]));
            }
        })
        .await;
        updates.abort();

        assert!(pushed.is_ok(), "no event pushed: {response}");
        assert!(
            response.contains("\"artifactsChanged\":false"),
            "unexpected event: {response}"
        );
        assert!(
            response.contains("\"reviewStateChanged\":true"),
            "unexpected event: {response}"
        );
    }

    /// One submission, one event. The bulk act writes every resolution in a
    /// single append precisely so a tab watching a change with twenty threads
    /// does not receive twenty updates for what the reviewer did once.
    #[tokio::test]
    async fn resolve_all_and_approve_pushes_one_event_rather_than_one_per_comment() {
        let fixture = project_fixture();
        let root = fixture.path().canonicalize().expect("root");
        for body in ["One.", "Two.", "Three."] {
            comments::add_unanchored(&root, &change_scope(), body).expect("comment");
        }
        let address = serve(&root).await;
        let mut stream = TcpStream::connect(address).await.expect("connect");
        stream
            .write_all(
                format!("GET /api/changes/{CHANGE}/events HTTP/1.1\r\nHost: localhost\r\n\r\n")
                    .as_bytes(),
            )
            .await
            .expect("write");
        // The subscription is taken when the handler runs, and an update written
        // before it exists reaches nobody.
        tokio::time::sleep(Duration::from_millis(500)).await;

        let outcome = post_json(
            address,
            &format!("/api/changes/{CHANGE}/approval"),
            serde_json::json!({ "act": "resolve-all-and-approve" }),
            StatusCode::OK,
        )
        .await;
        assert_eq!(outcome["resolved"], 3);

        let mut response = String::new();
        let mut buffer = [0_u8; 1024];
        let arrived = tokio::time::timeout(Duration::from_secs(10), async {
            while !response.contains("data: ") {
                let read = stream.read(&mut buffer).await.expect("read");
                assert_ne!(read, 0, "stream closed: {response}");
                response.push_str(&String::from_utf8_lossy(&buffer[..read]));
            }
        })
        .await;
        assert!(arrived.is_ok(), "no event pushed: {response}");

        // Long enough for a per-comment burst to have shown itself, both under
        // the watcher's settle window and under the polling fallback's interval.
        let _ = tokio::time::timeout(Duration::from_millis(2500), async {
            loop {
                let read = stream.read(&mut buffer).await.expect("read");
                if read == 0 {
                    break;
                }
                response.push_str(&String::from_utf8_lossy(&buffer[..read]));
            }
        })
        .await;

        assert_eq!(
            response.matches("data: ").count(),
            1,
            "one submission announced more than once: {response}"
        );
        assert!(
            response.contains("\"reviewStateChanged\":true"),
            "the approval state has to reach the open tab: {response}"
        );
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
