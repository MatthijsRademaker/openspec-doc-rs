//! The exploration scratch note: a plain markdown file that exists from the
//! start of a session, so explore-phase commenting has something to anchor
//! against before any `openspec/changes/<name>/` directory does.
//!
//! A note starts out keyed by session id and is promoted to a change-name key
//! ([`check`]) once the session has claimed ([`claim`]) the change its
//! exploration became.

mod note;
mod promote;

pub use note::{
    change_path, change_relative, claim, claimed_change, prepare_session, promoted_to,
    read_session, session_path, session_relative,
};
pub use promote::{Promotion, Validation, check};

/// The sidecar directory holding every scratch note.
const SCRATCH_DIR: &str = ".openspec-doc/scratch";

/// Session-keyed notes sit one level deeper, so a session id can never collide
/// with a change name.
const SESSION_DIR: &str = "_session";
