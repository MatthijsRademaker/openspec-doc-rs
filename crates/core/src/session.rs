//! The session-id guard shared by every sidecar directory keyed on session id.

use crate::error::Error;

/// Reject a session id that would escape the sidecar directory it keys.
pub(crate) fn check_session_id(session_id: &str) -> Result<(), Error> {
    // The session id comes from the agent and lands in a path, so a separator
    // or traversal component would escape the sidecar directory.
    if session_id.is_empty() || matches!(session_id, "." | "..") || session_id.contains(['/', '\\'])
    {
        return Err(Error::InvalidSessionId {
            session_id: session_id.to_owned(),
        });
    }

    Ok(())
}
