//! Resolving a URL key to the project scope it names, and to the artifacts and
//! sidecars that scope is made of. A key that no discovered session or active
//! change matches resolves to nothing, which the routes turn into a 404 rather
//! than an empty page.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use openspec_doc_core::comments::{self, ScopeKey};
use openspec_doc_core::{Project, hook, scan, scratch, verdict};

use crate::error::Error;
use crate::watch::Target;

/// The change artifacts a page renders, in reading order, before its spec
/// deltas.
const CHANGE_ARTIFACTS: [&str; 3] = ["proposal.md", "design.md", "tasks.md"];

/// One rendered artifact: where it lives relative to the project root, and the
/// markdown currently in it.
pub struct Artifact {
    pub path: String,
    pub markdown: String,
}

/// Everything a scope's page renders and watches.
pub struct Resolved {
    pub key: ScopeKey,
    pub artifacts: Vec<Artifact>,
    pub target: Target,
}

/// Every session with a directive record, and every active change name.
pub fn discovered(project: &Project) -> Result<(Vec<String>, Vec<String>), Error> {
    let sessions = hook::sessions(&project.root)?;
    let changes = scan(project.clone())?
        .active_changes
        .into_iter()
        .map(|dir| dir.name)
        .collect();

    Ok((sessions, changes))
}

/// What `/sessions/<session_id>` renders and watches, or `None` when no session
/// by that id has a directive record.
///
/// The session's one artifact is its scratch note, which does not exist until
/// the exploration writes something down — an absent note is a page with
/// nothing to comment on, not a missing session.
pub fn session(project: &Project, session_id: &str) -> Result<Option<Resolved>, Error> {
    let known = hook::sessions(&project.root)?;
    if !known.iter().any(|session| session == session_id) {
        return Ok(None);
    }

    let key = ScopeKey::Session(session_id.to_owned());
    let note_path = scratch::session_relative(session_id)?;
    let artifacts = read_artifacts(project, &[note_path])?;

    Ok(Some(Resolved {
        target: target(project, &key, &[])?,
        key,
        artifacts,
    }))
}

/// What `/changes/<name>` renders and watches, or `None` when no active change
/// is named `name`.
pub fn change(project: &Project, name: &str) -> Result<Option<Resolved>, Error> {
    let found = scan(project.clone())?
        .active_changes
        .into_iter()
        .find(|dir| dir.name == name);
    let Some(dir) = found else {
        return Ok(None);
    };

    let key = ScopeKey::Change(name.to_owned());
    let artifacts = read_artifacts(project, &change_artifact_paths(project, name)?)?;

    Ok(Some(Resolved {
        target: target(project, &key, &[dir.path])?,
        key,
        artifacts,
    }))
}

/// A change's markdown artifacts relative to the project root: the fixed
/// proposal/design/tasks trio, then one spec delta per capability directory.
///
/// The scratch note the change was promoted from is included, so an exploration's
/// notes stay readable and commentable after it has been formalized.
fn change_artifact_paths(project: &Project, name: &str) -> Result<Vec<String>, Error> {
    let dir = format!("openspec/changes/{name}");
    let mut paths: Vec<String> = CHANGE_ARTIFACTS
        .iter()
        .map(|artifact| format!("{dir}/{artifact}"))
        .collect();

    let specs_dir = project.root.join(&dir).join("specs");
    for capability in read_dir_names(&specs_dir)? {
        paths.push(format!("{dir}/specs/{capability}/spec.md"));
    }

    paths.push(scratch::change_relative(name));

    Ok(paths)
}

/// The markdown of each of `paths` that exists, in the order given. A path with
/// no file behind it is left out rather than rendered as an empty artifact.
fn read_artifacts(project: &Project, paths: &[String]) -> Result<Vec<Artifact>, Error> {
    let mut artifacts = Vec::new();

    for path in paths {
        if let Some(markdown) = comments::read_artifact(&project.root, path)? {
            artifacts.push(Artifact {
                path: path.clone(),
                markdown,
            });
        }
    }

    Ok(artifacts)
}

/// Subdirectory names of `dir`, sorted. A missing `dir` has none.
fn read_dir_names(dir: &Path) -> Result<Vec<String>, Error> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(Error::ReadDir {
                path: dir.to_owned(),
                source,
            });
        }
    };

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::ReadDir {
            path: dir.to_owned(),
            source,
        })?;
        if !entry.path().is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_owned());
        }
    }

    names.sort();
    Ok(names)
}

/// What a scope watches: the trees `extra` names, plus the sidecars its
/// comments, verdicts, scratch note and directives live in.
///
/// The whole of `.openspec-doc` is handed to the watcher because a sidecar
/// directory is only created when its first record is written; the relevant
/// paths then narrow the events back down to this scope's own files.
fn target(project: &Project, key: &ScopeKey, extra: &[PathBuf]) -> Result<Target, Error> {
    let mut relevant = vec![key.path(&project.root)?, verdict::path(&project.root, key)?];
    relevant.extend(extra.iter().cloned());

    match key {
        ScopeKey::Session(session_id) => {
            relevant.push(scratch::session_path(&project.root, session_id)?);
            relevant.push(hook::directive_path(&project.root, session_id)?);
        }
        ScopeKey::Change(name) => relevant.push(scratch::change_path(&project.root, name)),
    }

    let mut watched_dirs = vec![project.root.join(".openspec-doc")];
    watched_dirs.extend(extra.iter().cloned());

    Ok(Target {
        watched_dirs,
        relevant,
    })
}
