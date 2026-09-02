//! Filesystem discovery for OpenSpec projects: locate the project root and
//! enumerate active changes, archived changes, and current specs by directory
//! structure alone. No markdown content is parsed here.
//!
//! Also hosts the agent hook bridge ([`hook`]), which reads and writes the
//! project's directive sidecar files, the exploration scratch notes
//! ([`scratch`]) that pre-date a change directory, the anchored review
//! comments ([`comments`]) recorded against either, the phase verdicts
//! ([`verdict`]) a reviewer reaches about a whole scope, and the port a
//! project's dashboard belongs on ([`dashboard`]).

pub mod comments;
pub mod dashboard;
mod error;
pub mod hook;
mod root;
mod scan;
pub mod scratch;
mod session;
pub mod verdict;

#[cfg(test)]
mod test_fixture;

pub use error::Error;
pub use root::{Project, canonicalize, find_project, project_at};
pub use scan::{NamedDir, Scan, scan};
