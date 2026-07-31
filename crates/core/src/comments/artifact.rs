//! Reading the markdown a comment is anchored to. Any markdown file under the
//! project root qualifies — an explore-phase scratch note and a change's
//! `proposal.md` are handled identically, by path.

use std::fs;
use std::io;
use std::path::{Component, Path};

use crate::error::Error;

use super::anchor::{self, Anchor, Resolution};

/// The markdown of the artifact at `artifact_path`, relative to `root`, or
/// `None` when no such file exists.
pub fn read(root: &Path, artifact_path: &str) -> Result<Option<String>, Error> {
    let path = root.join(check_path(artifact_path)?);

    match fs::read_to_string(&path) {
        Ok(markdown) => Ok(Some(markdown)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::io(path, source)),
    }
}

/// Where `anchor` lands in its artifact as that artifact stands on disk right
/// now, which is the only state a reader of a stored comment cares about.
pub fn resolve(root: &Path, anchor: &Anchor) -> Result<Resolution, Error> {
    let markdown = read(root, &anchor.artifact_path)?;

    Ok(anchor::resolve(anchor, markdown.as_deref()))
}

/// Reject an artifact path that does not name a file inside the project.
pub(super) fn check_path(artifact_path: &str) -> Result<&Path, Error> {
    let path = Path::new(artifact_path);

    // The path is recorded in a comment and later joined onto the project root,
    // so an absolute or upward component would read outside the project.
    let contained = !artifact_path.is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));

    if !contained {
        return Err(Error::InvalidArtifactPath {
            artifact_path: artifact_path.to_owned(),
        });
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::anchor::AnchorState;
    use tempfile::TempDir;

    #[test]
    fn an_artifact_is_read_relative_to_the_project_root() {
        let temp = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp.path().join("openspec/changes/add-thing")).expect("create dirs");
        fs::write(
            temp.path().join("openspec/changes/add-thing/proposal.md"),
            "# Why\n",
        )
        .expect("write artifact");

        assert_eq!(
            read(temp.path(), "openspec/changes/add-thing/proposal.md").expect("read"),
            Some("# Why\n".to_owned())
        );
    }

    #[test]
    fn a_missing_artifact_reads_as_none() {
        let temp = TempDir::new().expect("temp dir");

        assert_eq!(read(temp.path(), "openspec/gone.md").expect("read"), None);
    }

    #[test]
    fn a_missing_artifact_resolves_an_anchor_as_missing() {
        let temp = TempDir::new().expect("temp dir");
        let anchor = anchor::create("openspec/gone.md", "Selected text.", "Selected text.")
            .expect("create anchor");

        let resolution = resolve(temp.path(), &anchor).expect("resolve");

        assert_eq!(resolution.state, AnchorState::Missing);
    }

    #[test]
    fn artifact_paths_cannot_escape_the_project_root() {
        let temp = TempDir::new().expect("temp dir");

        for artifact_path in [
            "",
            "../escape.md",
            "openspec/../../escape.md",
            "/etc/passwd",
        ] {
            assert!(
                matches!(
                    read(temp.path(), artifact_path),
                    Err(Error::InvalidArtifactPath { .. })
                ),
                "accepted {artifact_path:?}"
            );
        }
    }
}
