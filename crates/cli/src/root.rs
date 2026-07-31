use std::env;
use std::path::Path;

use openspec_doc_core::{Project, find_project, project_at};

use crate::error::Error;

/// Resolve the project root shared by every subcommand: an explicit `--root` is
/// used verbatim, otherwise discovery walks up from the current directory.
pub fn resolve(root: Option<&Path>) -> Result<Project, Error> {
    match root {
        Some(root) => Ok(project_at(root)?),
        None => {
            let cwd = env::current_dir().map_err(|source| Error::CurrentDir { source })?;
            Ok(find_project(&cwd)?)
        }
    }
}
