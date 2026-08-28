//! What is registered in `.claude/settings.json`, as recorded.
//!
//! The events to look under and the predicate that recognises our entries both
//! come from [`wiring`], the module `init` writes the file from, so the two
//! commands cannot disagree about what a wired project contains.
//!
//! What comes back is the command string exactly as it stands. A developer
//! pointing an entry at a local build — `OPENSPEC_DOC_BIN=… openspec-doc hook
//! stop --agent claude`, or an absolute path into `target/debug` — is running
//! that string, and probing the canonical one would check a binary the agent
//! never invokes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::Error;
use crate::wiring;

/// The one settings file `init` writes and `doctor` reads back.
pub const RELATIVE: &str = ".claude/settings.json";

pub fn path(root: &Path) -> PathBuf {
    root.join(RELATIVE)
}

/// The settings file's contents, or `Value::Null` when the project has none.
///
/// A file that does not parse is an error rather than an empty result: an
/// unreadable settings file is a reason to report nothing about the hooks, and
/// reporting them as unregistered would name the wrong fix.
pub fn read(root: &Path) -> Result<Value, Error> {
    let path = path(root);

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Value::Null),
        Err(source) => return Err(Error::Read { path, source }),
    };

    serde_json::from_str(&contents).map_err(|source| Error::Settings { path, source })
}

/// The command registered under `event` that is ours, exactly as recorded.
pub fn command<'a>(settings: &'a Value, event: &str) -> Option<&'a str> {
    settings
        .get("hooks")?
        .get(event)?
        .as_array()?
        .iter()
        .filter_map(|group| group.get("hooks")?.as_array())
        .flatten()
        .find(|command| wiring::is_ours(command))?
        .get("command")?
        .as_str()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn root_with_settings(contents: &str) -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        let path = path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("create .claude");
        fs::write(path, contents).expect("write settings");
        temp
    }

    /// The recorded string is what the agent runs, so it is what comes back —
    /// however the operator spelled the binary.
    #[test]
    fn a_locally_built_binary_is_returned_verbatim() {
        for recorded in [
            "OPENSPEC_DOC_BIN=/tmp/b openspec-doc hook stop --agent claude",
            "/tmp/target/debug/openspec-doc hook stop --agent claude",
        ] {
            let temp = root_with_settings(&format!(
                r#"{{"hooks": {{"Stop": [{{"hooks": [{{"type": "command", "command": "{recorded}"}}]}}]}}}}"#
            ));

            let settings = read(temp.path()).expect("read");

            assert_eq!(command(&settings, "Stop"), Some(recorded));
        }
    }

    #[test]
    fn every_entry_init_writes_is_found_under_its_own_event() {
        let temp = root_with_settings(
            &serde_json::to_string(&wiring::register(&Value::Null)).expect("serialize"),
        );

        let settings = read(temp.path()).expect("read");

        for entry in wiring::ENTRIES {
            assert_eq!(command(&settings, entry.event), Some(entry.command));
        }
    }

    #[test]
    fn a_missing_settings_file_registers_nothing() {
        let temp = TempDir::new().expect("temp dir");

        let settings = read(temp.path()).expect("a missing file is not an error");

        for entry in wiring::ENTRIES {
            assert_eq!(command(&settings, entry.event), None);
        }
    }

    #[test]
    fn an_unrelated_hook_under_our_event_is_not_ours() {
        let temp = root_with_settings(
            r#"{"hooks": {"Stop": [{"hooks": [{"type": "command", "command": "make lint"}]}]}}"#,
        );

        let settings = read(temp.path()).expect("read");

        assert_eq!(command(&settings, "Stop"), None);
    }

    #[test]
    fn a_malformed_settings_file_is_an_error_rather_than_an_empty_result() {
        let temp = root_with_settings("{ not json");

        assert!(matches!(read(temp.path()), Err(Error::Settings { .. })));
    }
}
