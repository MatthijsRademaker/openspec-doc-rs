//! The dashboard's server-rendered pages: the shared document shell, the
//! artifact and comment rendering both scoped pages are built from, and the
//! pages themselves.
//!
//! Rendering is plain Rust string building against the same `escape` helper
//! throughout, and the review-state fragment the pages share is one function
//! rather than a template partial — there is no template engine and no build
//! step, per this change's MVP scope.

mod artifact;
mod index;
mod review;
mod scoped;
mod script;

pub use index::index;
pub use review::{Anchored, Review, fragment as review_fragment};
pub use scoped::{change, session};

pub fn not_found() -> String {
    document(
        "Not found",
        "<h1>Not found</h1>\n<p>No discovered session or change matches this URL.</p>",
        None,
    )
}

/// A submission the review primitives refused, reported with the reason they
/// gave rather than as a generic failure — the reviewer is the one who can act
/// on it, most often by reselecting text the artifact has moved on from.
pub fn rejected(reason: &str) -> String {
    document(
        "Not recorded",
        &format!(
            "<h1>Not recorded</h1>\n<p>{}</p>\n<p>Go back and try again.</p>",
            escape(reason)
        ),
        None,
    )
}

pub fn internal_error() -> String {
    document(
        "Error",
        "<h1>Error</h1>\n<p>The request failed; see the <code>openspec-doc serve</code> output.</p>",
        None,
    )
}

/// Where a scoped page's live updates come from, and where it refetches its
/// review state when one arrives.
pub(super) struct Endpoints {
    pub events: String,
    pub review: String,
    pub comments: String,
    pub verdict: String,
}

/// The document shell. Endpoint paths travel in `data-` attributes rather than
/// being interpolated into the script: HTML escaping applies to attributes, but
/// not to script content, so a session id or change name is never able to break
/// out of the source.
pub(super) fn document(title: &str, body: &str, endpoints: Option<&Endpoints>) -> String {
    let attributes = match endpoints {
        Some(endpoints) => format!(
            " data-events=\"{}\" data-review=\"{}\" data-review-container=\"{}\"",
            escape(&endpoints.events),
            escape(&endpoints.review),
            review::CONTAINER_ID
        ),
        None => String::new(),
    };

    format!(
        "<!doctype html>\n\
         <html lang=\"en\">\n\
         <head><meta charset=\"utf-8\"><title>{title}</title>\n<style>{style}</style></head>\n\
         <body{attributes}>\n{body}\n<script>{script}</script>\n</body>\n\
         </html>\n",
        title = escape(title),
        style = script::STYLE,
        script = script::REVIEW
    )
}

/// Session ids, change names, artifact markdown and comment bodies all come from
/// disk or from the reviewer, so none of them are trusted to be free of markup.
pub(super) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
