//! The approval precheck: what an apply workflow runs before it starts writing
//! code, and what its exit code means.
//!
//! Only one of the three states exits zero. A precheck that exited zero for a
//! stale approval would be read as clearance by every caller that checks a
//! status code, which is the only way this is used.

use openspec_doc_core::Project;
use openspec_doc_core::approval;

use crate::error::Error;

/// Print `change`'s approval state and its reason, and report whether the change
/// is cleared for implementation.
pub fn state(project: Project, change: &str) -> Result<(), Error> {
    let approval = approval::state(&project, change)?;

    println!("{change} is {}: {}", approval.state, approval.reason);

    if approval.state.is_approved() {
        return Ok(());
    }

    Err(Error::NotApproved {
        change: change.to_owned(),
    })
}
