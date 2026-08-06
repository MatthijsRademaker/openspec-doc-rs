//! The built frontend, embedded in the binary.
//!
//! `web/dist/` is committed and compiled in, so the binary needs no asset
//! directory, no checked-out repository and no Node toolchain — neither to run
//! nor to `cargo install`. What makes that safe is the CI check that rebuilds
//! the frontend from a clean checkout and fails if the committed output differs;
//! without it a stale `dist/` would ship an interface that does not match its
//! source, silently.
//!
//! A missing or empty `web/dist/` is a compile error rather than a server that
//! starts and serves nothing.

use axum::http::header;
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../web/dist"]
struct Dist;

/// The application shell — the one document the frontend boots from.
pub fn shell() -> Response {
    serve("index.html").expect("web/dist/index.html is embedded at compile time")
}

/// The embedded asset at `path`, relative to `web/dist/`, or `None` when the
/// frontend does not own that path.
pub fn asset(path: &str) -> Option<Response> {
    serve(path)
}

fn serve(path: &str) -> Option<Response> {
    let file = Dist::get(path)?;
    let mimetype = file.metadata.mimetype().to_owned();

    Some(([(header::CONTENT_TYPE, mimetype)], file.data).into_response())
}
