//! The exploration scratch note: a plain markdown file that exists from the
//! start of a session, so explore-phase commenting has something to anchor
//! against before any `openspec/changes/<name>/` directory does.
//!
//! A note starts out keyed by session id and is promoted to a change-name key
//! ([`check`]) once the session's exploration is formalized into a change.

mod note;
mod promote;
mod snapshot;

pub use note::{
    change_path, change_relative, ensure_session, read_session, session_path, session_relative,
};
pub use promote::{Promotion, Validation, check};

/// The sidecar directory holding every scratch note.
const SCRATCH_DIR: &str = ".openspec-doc/scratch";

/// Session-keyed notes sit one level deeper, so a session id can never collide
/// with a change name.
const SESSION_DIR: &str = "_session";
