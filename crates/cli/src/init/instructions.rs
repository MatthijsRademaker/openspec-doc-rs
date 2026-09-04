//! The managed block in `AGENTS.md`.
//!
//! These instructions are ambient rather than on demand. An agent has to know
//! what the scratch note is *for* before it decides where to put its thinking,
//! and one that has already written its exploration into the conversation has no
//! reason to load a skill telling it not to. So the content is pushed into the
//! instruction file the owner already owns, delimited so everything they wrote
//! survives byte for byte.

use std::path::Path;

use crate::error::Error;

pub const BEGIN: &str = "<!-- openspec-doc:begin -->";
pub const END: &str = "<!-- openspec-doc:end -->";

const BLOCK: &str = include_str!("../../assets/agents-block.md");

/// The delimited block, from the opening marker through the closing one. No
/// trailing newline: the byte after the closing marker belongs to whoever owns
/// the rest of the file.
fn block() -> String {
    format!("{BEGIN}\n{BLOCK}{END}")
}

/// `existing` with the managed block installed: appended when neither marker is
/// present, and otherwise replacing exactly the span between them.
///
/// A file carrying one marker and not the other is an error rather than a guess.
/// Half-delimited means something already went wrong, and repairing it by
/// appending would leave two overlapping blocks in the file that tells the agent
/// how to behave.
pub fn install(path: &Path, existing: Option<&str>) -> Result<String, Error> {
    let Some(existing) = existing else {
        return Ok(format!("{}\n", block()));
    };

    match (existing.find(BEGIN), existing.find(END)) {
        (Some(begin), Some(end)) if begin < end => Ok(format!(
            "{}{}{}",
            &existing[..begin],
            block(),
            &existing[end + END.len()..]
        )),
        (None, None) => {
            let separator = if existing.is_empty() || existing.ends_with("\n\n") {
                ""
            } else if existing.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };

            Ok(format!("{existing}{separator}{}\n", block()))
        }
        _ => Err(Error::HalfDelimitedInstructions {
            path: path.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path() -> &'static Path {
        Path::new("AGENTS.md")
    }

    #[test]
    fn appends_when_neither_marker_is_present() {
        let existing = "# Rules\n\nBe precise.\n";

        let installed = install(path(), Some(existing)).expect("install");

        assert!(installed.starts_with(existing), "{installed}");
        assert!(
            installed.ends_with(&format!("{}\n", block())),
            "{installed}"
        );
    }

    #[test]
    fn creates_the_file_content_when_absent() {
        assert_eq!(
            install(path(), None).expect("install"),
            format!("{}\n", block())
        );
    }

    #[test]
    fn replaces_only_the_span_between_the_markers() {
        // A sentinel no prose could contain: the block is ordinary English, so a
        // word like "stale" starts out absent from it and stops being absent the
        // day someone writes about staleness.
        const SUPERSEDED: &str = "xyzzy-superseded-block-content";
        let existing = format!("before\n\n{BEGIN}\n{SUPERSEDED}\n{END}\n\nafter\n");

        let installed = install(path(), Some(&existing)).expect("install");

        assert!(installed.starts_with("before\n\n"), "{installed}");
        assert!(installed.ends_with("\n\nafter\n"), "{installed}");
        assert!(!installed.contains(SUPERSEDED), "{installed}");
    }

    #[test]
    fn installing_twice_is_the_same_file() {
        let once = install(path(), Some("# Rules\n")).expect("install");

        assert_eq!(install(path(), Some(&once)).expect("install"), once);
    }

    #[test]
    fn rejects_a_half_delimited_file() {
        for existing in [format!("{BEGIN}\nno end\n"), format!("no begin\n{END}\n")] {
            assert!(matches!(
                install(path(), Some(&existing)),
                Err(Error::HalfDelimitedInstructions { .. })
            ));
        }
    }
}
