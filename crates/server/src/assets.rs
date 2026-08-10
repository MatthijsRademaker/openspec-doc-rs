//! The built frontend, embedded in the binary.
//!
//! `web/dist/` is a gitignored build artifact compiled in from whatever the
//! local build produced, so the binary needs no asset directory, no checked-out
//! repository and no Node toolchain — neither to run nor to `cargo install`.
//! The cost is that Cargo depends on a build step outside Cargo: build the
//! frontend first (`make build`), because a fresh clone has no `dist/`.
//!
//! An absent `web/dist/` is a compile error rather than a server that starts
//! and serves nothing. A `web/dist/` that exists but is empty is *not* caught
//! here: rust-embed compiles zero files happily and `shell` then panics on the
//! first request. Catching that would take a compile-time assertion that
//! `index.html` is embedded.
//!
//! Nothing compares the embedded bytes against a committed baseline, because
//! there is no committed baseline. What CI proves instead is that a clean
//! checkout can install, check, build, embed and serve the frontend in a real
//! browser; see `.github/workflows/frontend-assets.yml`.

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
