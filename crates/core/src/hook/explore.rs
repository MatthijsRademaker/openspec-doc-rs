//! Entering explore mode: creating the session's scratch note, and telling the
//! agent where that note is.
//!
//! What this readies is the note's *location*, not the note. Writing the
//! exploration down is the agent's job, and an empty file left in its way turns
//! its first write into a failure — Claude Code refuses to write over a file it
//! has not read. So a note comes into existence when an exploration is written,
//! which is also the invariant `promote::check` leans on: a session's claim is
//! recorded inside its note, so no note means no claim and nothing to promote.

use std::path::Path;

use crate::error::Error;
use crate::scratch;

/// Start an exploration for `session_id`: ready the directory its scratch note
/// belongs in, and return the text telling the agent to write the exploration
/// there.
///
/// The note path is returned already resolved rather than as something for the
/// agent to build out of its session id — an agent that interpolates the path
/// itself can silently write to the wrong file, and a note nobody is reading
/// looks exactly like an exploration nobody wrote.
pub fn start(root: &Path, session_id: &str) -> Result<String, Error> {
    scratch::prepare_session(root, session_id)?;

    let note = scratch::session_relative(session_id)?;

    Ok(format!(
        "openspec-doc: this exploration is being reviewed from the openspec-doc dashboard in \
         this project. Write the exploration down in `{note}` as you go, and keep it current \
         as your understanding changes. That note is the only thing the reviewer can read and \
         anchor comments against while you are still exploring — an exploration that stays in \
         the conversation is invisible to them, so prose they can quote beats a list of \
         headings. The note stays with the work: once you have created the change this becomes, \
         run `openspec-doc scratch claim --session {session_id} --change <the change you \
         created>` and the note, with its comments, is promoted onto that change."
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    const SESSION: &str = "session-a";

    fn note_path(root: &Path) -> std::path::PathBuf {
        scratch::session_path(root, SESSION).expect("path")
    }

    /// The agent writes the note. Leaving an empty one in its way would make
    /// that write fail, since Claude Code will not write over an unread file.
    #[test]
    fn the_notes_location_is_readied_and_the_note_is_left_to_the_exploration() {
        let temp = TempDir::new().expect("temp dir");

        start(temp.path(), SESSION).expect("start");

        let path = note_path(temp.path());
        assert!(
            path.parent().expect("parent").is_dir(),
            "the note has nowhere to be written"
        );
        assert!(
            !path.exists(),
            "an empty note here breaks the agent's first write"
        );
    }

    #[test]
    fn an_exploration_already_under_way_keeps_what_it_has_written() {
        let temp = TempDir::new().expect("temp dir");
        start(temp.path(), SESSION).expect("first start");
        fs::write(
            note_path(temp.path()),
            "# Exploration\n\nA thought worth keeping.\n",
        )
        .expect("write note");

        start(temp.path(), SESSION).expect("second start");

        assert_eq!(
            fs::read_to_string(note_path(temp.path())).expect("read back"),
            "# Exploration\n\nA thought worth keeping.\n",
            "re-entering explore mode must not clear the note"
        );
    }

    /// The agent is told a path, not a formula: a session id it has to
    /// interpolate itself is one it can interpolate wrongly.
    #[test]
    fn the_instruction_names_the_resolved_note_path() {
        let temp = TempDir::new().expect("temp dir");

        let instruction = start(temp.path(), SESSION).expect("start");

        assert!(
            instruction.contains(".openspec-doc/scratch/_session/session-a.md"),
            "{instruction}"
        );
        assert!(
            !instruction.contains("SESSION_ID"),
            "the path must arrive resolved, not as a template: {instruction}"
        );
    }

    #[test]
    fn the_instruction_says_what_the_note_is_for() {
        let temp = TempDir::new().expect("temp dir");

        let instruction = start(temp.path(), SESSION).expect("start");

        assert!(instruction.contains("reviewer"), "{instruction}");
        assert!(instruction.contains("anchor comments"), "{instruction}");
    }

    /// Nothing promotes the note but the claim, so the instruction that sets an
    /// exploration going cannot describe promotion as automatic.
    #[test]
    fn the_instruction_names_the_claim_command_for_this_session() {
        let temp = TempDir::new().expect("temp dir");

        let instruction = start(temp.path(), SESSION).expect("start");

        assert!(
            instruction.contains("openspec-doc scratch claim --session session-a --change"),
            "{instruction}"
        );
    }

    #[test]
    fn session_ids_cannot_escape_the_scratch_directory() {
        let temp = TempDir::new().expect("temp dir");

        for session_id in ["", ".", "..", "../escape", "nested/id"] {
            assert!(
                matches!(
                    start(temp.path(), session_id),
                    Err(Error::InvalidSessionId { .. })
                ),
                "accepted {session_id:?}"
            );
        }
    }
}
