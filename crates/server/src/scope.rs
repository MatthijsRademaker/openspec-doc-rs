//! Resolving a URL key to the project scope it names, and to the artifacts and
//! sidecars that scope is made of. A key that no discovered session or active
//! change matches resolves to nothing, which the routes turn into a 404 rather
//! than an empty page.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use openspec_doc_core::comments::{self, ScopeKey, Status};
use openspec_doc_core::verdict::Verdict;
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
    /// What the scope is called, when it is called anything but its key.
    pub title: Option<String>,
    pub artifacts: Vec<Artifact>,
    pub target: Target,
}

/// One scope as the index lists it: enough to tell it apart from its neighbours
/// and to see where its review stands without opening it.
pub struct Summary {
    /// The session id or change name. It is the scope's identity and its
    /// address; the title is display text and never either.
    pub key: String,
    /// What the scope is called, when it is called anything but its key.
    pub title: Option<String>,
    /// When the scope's own artifacts were last written, or `None` while none of
    /// them exists — a session whose exploration has not started yet.
    pub modified: Option<SystemTime>,
    pub open_comments: usize,
    /// The scope's standing verdict: the last one recorded for it.
    pub verdict: Option<Verdict>,
    /// Whether this is the scope whose review sidecars changed most recently.
    pub most_recently_active: bool,
}

/// Every session with a directive record, and every active change, each
/// summarised for the index.
///
/// The most recently active session is marked here rather than in the rendering,
/// because which one it is is a fact about the whole set.
pub fn discovered(project: &Project) -> Result<(Vec<Summary>, Vec<Summary>), Error> {
    let mut sessions = Vec::new();
    let mut newest: Option<(usize, SystemTime)> = None;

    for session_id in hook::sessions(&project.root)? {
        if let Some(at) = session_activity(project, &session_id)?
            && newest.is_none_or(|(_, best)| at > best)
        {
            newest = Some((sessions.len(), at));
        }

        sessions.push(session_summary(project, session_id)?);
    }

    if let Some((index, _)) = newest {
        sessions[index].most_recently_active = true;
    }

    let mut changes = Vec::new();
    for dir in scan(project.clone())?.active_changes {
        changes.push(change_summary(project, dir.name)?);
    }

    Ok((sessions, changes))
}

fn session_summary(project: &Project, session_id: String) -> Result<Summary, Error> {
    let key = ScopeKey::Session(session_id.clone());

    Ok(Summary {
        title: session_name(project, &session_id)?,
        modified: last_modified(project, &[scratch::session_relative(&session_id)?])?,
        open_comments: open_comments(project, &key)?,
        verdict: standing_verdict(project, &key)?,
        key: session_id,
        most_recently_active: false,
    })
}

fn change_summary(project: &Project, name: String) -> Result<Summary, Error> {
    let key = ScopeKey::Change(name.clone());

    Ok(Summary {
        title: scratch::change_title(&project.root, &name)?,
        modified: last_modified(project, &change_artifact_paths(project, &name)?)?,
        open_comments: open_comments(project, &key)?,
        verdict: standing_verdict(project, &key)?,
        key: name,
        // A change is not a session and has nothing to be the live one of.
        most_recently_active: false,
    })
}

/// What to call a session: the title of its own note, or — once its exploration
/// has been promoted away — the change that exploration became.
///
/// Promotion leaves a redirect at the note's path, and a redirect has no heading
/// and so no title. Falling all the way back to the session id there leaves
/// exactly the unreadable row the index exists to fix, when the one thing worth
/// knowing about that session is already recorded in the redirect.
fn session_name(project: &Project, session_id: &str) -> Result<Option<String>, Error> {
    if let Some(title) = scratch::session_title(&project.root, session_id)? {
        return Ok(Some(title));
    }

    Ok(scratch::promoted_to(&project.root, session_id)?
        .map(|change| format!("Promoted to {change}")))
}

/// How many of the scope's comments still want attention.
fn open_comments(project: &Project, key: &ScopeKey) -> Result<usize, Error> {
    Ok(comments::read(&project.root, key)?
        .iter()
        .filter(|thread| thread.status == Status::Open)
        .count())
}

/// The scope's standing verdict — the last one recorded, the earlier ones being
/// history rather than the decision in force.
fn standing_verdict(project: &Project, key: &ScopeKey) -> Result<Option<Verdict>, Error> {
    Ok(verdict::read(&project.root, key)?
        .pop()
        .map(|record| record.verdict))
}

/// When a session's directive or verdict sidecar was last written, or `None`
/// while it has neither.
///
/// Those two files are where a session's traffic with the reviewer lands, so
/// their timestamps are the closest thing on disk to activity. This is not
/// liveness: nothing here says the session is still running, which is why the
/// index labels it as the most recently active one and no more.
fn session_activity(project: &Project, session_id: &str) -> Result<Option<SystemTime>, Error> {
    let key = ScopeKey::Session(session_id.to_owned());
    let paths = [
        hook::directive_path(&project.root, session_id)?,
        verdict::path(&project.root, &key)?,
    ];

    newest_mtime(&paths)
}

/// When any of `paths`, relative to the project root, was last written.
fn last_modified(project: &Project, paths: &[String]) -> Result<Option<SystemTime>, Error> {
    let absolute: Vec<PathBuf> = paths.iter().map(|path| project.root.join(path)).collect();

    newest_mtime(&absolute)
}

/// The latest modification time among the `paths` that exist. A path with no
/// file behind it contributes nothing rather than failing.
fn newest_mtime(paths: &[PathBuf]) -> Result<Option<SystemTime>, Error> {
    let mut newest = None;

    for path in paths {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(Error::Metadata {
                    path: path.clone(),
                    source,
                });
            }
        };

        let modified = metadata.modified().map_err(|source| Error::Metadata {
            path: path.clone(),
            source,
        })?;
        if newest.is_none_or(|current| modified > current) {
            newest = Some(modified);
        }
    }

    Ok(newest)
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
        title: session_name(project, session_id)?,
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
        title: scratch::change_title(&project.root, name)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    use openspec_doc_core::project_at;
    use tempfile::TempDir;

    const SESSION_A: &str = "11111111-2222-3333-4444-555555555555";
    const SESSION_B: &str = "66666666-7777-8888-9999-aaaaaaaaaaaa";

    /// A project with `sessions` discovered and one active change, and nothing
    /// else: no note, no comments, no verdicts. This is what every scope looks
    /// like the first time the index renders it.
    fn bare_project(sessions: &[&str]) -> (TempDir, Project) {
        let temp = TempDir::new().expect("temp dir");
        let change_dir = temp.path().join("openspec/changes/add-a");
        let directives = temp.path().join(".openspec-doc/directives/_session");

        fs::create_dir_all(&change_dir).expect("create change dir");
        fs::create_dir_all(&directives).expect("create directives dir");
        fs::write(temp.path().join("openspec/config.yaml"), "").expect("write config");
        fs::write(change_dir.join("proposal.md"), "## Why\n\nBecause.\n").expect("write proposal");

        for (index, session_id) in sessions.iter().enumerate() {
            if index > 0 {
                // Distinct mtimes, so "most recent" is decidable rather than a
                // tie broken by enumeration order.
                sleep(Duration::from_millis(20));
            }
            fs::write(
                directives.join(format!("{session_id}.json")),
                r#"{"pending":false,"reason":"x","createdAt":"2026-08-06T08:00:00Z","consumedAt":null}"#,
            )
            .expect("write directive");
        }

        let project = project_at(temp.path()).expect("project root");
        (temp, project)
    }

    #[test]
    fn a_scope_with_no_comments_and_no_verdict_summarises_without_error() {
        let (_temp, project) = bare_project(&[SESSION_A]);

        let (sessions, changes) = discovered(&project).expect("summarise");

        let [session] = sessions.as_slice() else {
            panic!("expected one session, got {}", sessions.len());
        };
        assert_eq!(session.key, SESSION_A);
        assert_eq!(session.title, None, "nothing has been explored yet");
        assert_eq!(session.open_comments, 0);
        assert_eq!(session.verdict, None);
        assert_eq!(session.modified, None, "the session has written no note");

        let [change] = changes.as_slice() else {
            panic!("expected one change, got {}", changes.len());
        };
        assert_eq!(change.key, "add-a");
        assert_eq!(change.open_comments, 0);
        assert_eq!(change.verdict, None);
        assert!(
            change.modified.is_some(),
            "the change's proposal is on disk and has an mtime"
        );
    }

    #[test]
    fn a_summary_carries_the_notes_title_and_the_scopes_review_state() {
        let (temp, project) = bare_project(&[SESSION_A]);
        let key = ScopeKey::Session(SESSION_A.to_owned());
        fs::write(
            scratch::prepare_session(temp.path(), SESSION_A).expect("prepare"),
            "# Exploring: session titles\n\nAn idea worth keeping.\n",
        )
        .expect("write note");
        comments::add(
            temp.path(),
            &key,
            &scratch::session_relative(SESSION_A).expect("relative"),
            "An idea worth keeping.",
            "Which part?",
        )
        .expect("add comment");
        verdict::add(temp.path(), &key, Verdict::KeepExploring, "Still open.")
            .expect("add verdict");

        let (sessions, _) = discovered(&project).expect("summarise");

        let [session] = sessions.as_slice() else {
            panic!("expected one session, got {}", sessions.len());
        };
        assert_eq!(session.title.as_deref(), Some("Exploring: session titles"));
        assert_eq!(session.open_comments, 1);
        assert_eq!(session.verdict, Some(Verdict::KeepExploring));
        assert!(session.modified.is_some(), "the note is on disk");
    }

    /// A promoted session has no note of its own left to be titled from, and its
    /// id alone is the row the index existed to fix. The redirect already records
    /// the one thing worth knowing about it.
    #[test]
    fn a_promoted_session_is_named_by_the_change_it_became() {
        let (temp, project) = bare_project(&[SESSION_A]);
        fs::write(
            scratch::prepare_session(temp.path(), SESSION_A).expect("prepare"),
            "<!-- openspec-doc:moved-to .openspec-doc/scratch/add-a.md -->\n\n\
             This scratch note moved to `.openspec-doc/scratch/add-a.md`.\n",
        )
        .expect("write redirect");

        let (sessions, _) = discovered(&project).expect("summarise");

        assert_eq!(sessions[0].title.as_deref(), Some("Promoted to add-a"));
        assert_eq!(
            sessions[0].key, SESSION_A,
            "the session is still addressed by its id"
        );
    }

    /// Which session a reviewer most likely wants is the one that has been
    /// talking to them most recently, and only one of them is marked.
    #[test]
    fn the_session_with_the_latest_sidecar_activity_is_marked() {
        let (_temp, project) = bare_project(&[SESSION_A, SESSION_B]);

        let (sessions, _) = discovered(&project).expect("summarise");

        let marked: Vec<&str> = sessions
            .iter()
            .filter(|session| session.most_recently_active)
            .map(|session| session.key.as_str())
            .collect();
        assert_eq!(
            marked,
            [SESSION_B],
            "{:?}",
            sessions
                .iter()
                .map(|s| (&s.key, s.modified))
                .collect::<Vec<_>>()
        );
    }
}
