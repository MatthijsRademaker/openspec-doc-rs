use openspec_doc_core::Project;
use openspec_doc_core::scratch;

use crate::error::Error;

/// Record that `session_id`'s exploration became `change`, printing the note the
/// claim was written into.
pub fn claim(project: Project, session_id: &str, change: &str) -> Result<(), Error> {
    let note = scratch::claim(&project.root, session_id, change)?;

    println!("session {session_id} claims change {change}");
    println!("  note: {}", note.display());

    Ok(())
}
