//! What a wired project contains.
//!
//! The three Claude Code hook entries live here as data rather than as literals
//! inside the writer, because two commands need them: `init` writes them, and
//! `doctor` checks that what is written is what should be. Two independent
//! copies drift, and the drift presents as `doctor` passing a project `init`
//! would rewrite.

use serde_json::{Map, Value, json};
use std::path::Path;

/// A Claude Code hook registration: which event fires it, what it runs, and what
/// the session shows while it runs.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// The Claude Code event this is registered under.
    pub event: &'static str,
    /// The `command_name` pattern, for the events that dispatch on one.
    pub matcher: Option<&'static str>,
    /// The executable Claude Code spawns directly.
    pub command: &'static str,
    /// The arguments Claude Code passes to the executable.
    pub args: &'static [&'static str],
    pub timeout: u64,
    pub status_message: &'static str,
}

/// Every hook this tool needs registered in a Claude Code project.
pub const ENTRIES: [Entry; 3] = [
    Entry {
        event: "Stop",
        matcher: None,
        command: "openspec-doc",
        args: &["hook", "stop", "--agent", "claude"],
        timeout: 30,
        status_message: "Checking openspec-doc review feedback",
    },
    Entry {
        event: "UserPromptSubmit",
        matcher: None,
        command: "openspec-doc",
        args: &["hook", "prompt", "--agent", "claude"],
        timeout: 30,
        status_message: "Checking openspec-doc review feedback",
    },
    Entry {
        // `UserPromptExpansion` dispatches on the *bare* command name — no
        // leading slash, no namespace — and a matcher that does not match fails
        // silently.
        event: "UserPromptExpansion",
        matcher: Some("opsx:explore|openspec-explore"),
        command: "openspec-doc",
        args: &["hook", "explore", "--agent", "claude"],
        timeout: 30,
        status_message: "Opening the openspec-doc exploration note",
    },
];

/// Whether a settings entry invokes this tool's hook bridge.
///
/// Current entries are structural: the executable's file stem is ours and the
/// first argument is `hook`. An entry without `args` is the old shell form; it
/// is recognised only so `init` can replace it. The first shell word must be
/// the executable, so an environment assignment is deliberately not recognised.
pub fn is_ours(entry: &Value) -> bool {
    let Some(command) = entry.get("command").and_then(Value::as_str) else {
        return false;
    };
    match entry.get("args") {
        Some(args) => {
            our_executable(command)
                && args
                    .as_array()
                    .and_then(|args| args.first())
                    .and_then(Value::as_str)
                    .is_some_and(|argument| argument == "hook")
        }
        None => {
            let mut words = command.split_whitespace();
            words.next().is_some_and(our_executable) && words.next() == Some("hook")
        }
    }
}

fn our_executable(command: &str) -> bool {
    Path::new(command)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem == "openspec-doc")
}

/// `settings` with every one of our hook entries registered exactly once, and
/// everything else in the file left alone.
///
/// Ours are stripped from the whole `hooks` object first and then re-added under
/// the events they belong to, so an entry left under the wrong event by an older
/// version is corrected rather than kept beside its replacement.
pub fn register(settings: &Value) -> Value {
    let mut root = settings.as_object().cloned().unwrap_or_default();
    let mut hooks = root
        .get("hooks")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    for groups in hooks.values_mut() {
        strip_ours(groups);
    }

    for entry in ENTRIES {
        let event = hooks
            .entry(entry.event)
            .or_insert_with(|| Value::Array(Vec::new()));
        if let Some(groups) = event.as_array_mut() {
            groups.push(group(&entry));
        }
    }

    hooks.retain(|_, groups| !groups.as_array().is_some_and(Vec::is_empty));
    root.insert("hooks".to_owned(), Value::Object(hooks));

    Value::Object(root)
}

/// Drop our commands from one event's groups, and any group they emptied.
fn strip_ours(groups: &mut Value) {
    let Some(groups) = groups.as_array_mut() else {
        return;
    };

    for group in groups.iter_mut() {
        if let Some(commands) = group.get_mut("hooks").and_then(Value::as_array_mut) {
            commands.retain(|command| !is_ours(command));
        }
    }

    groups.retain(|group| {
        !group
            .get("hooks")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
    });
}

fn group(entry: &Entry) -> Value {
    let mut group = Map::new();
    if let Some(matcher) = entry.matcher {
        group.insert("matcher".to_owned(), json!(matcher));
    }
    group.insert(
        "hooks".to_owned(),
        json!([{
            "type": "command",
            "command": entry.command,
            "args": entry.args,
            "timeout": entry.timeout,
            "statusMessage": entry.status_message,
        }]),
    );

    Value::Object(group)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registered() -> Value {
        register(&Value::Object(Map::new()))
    }

    fn commands(settings: &Value, event: &str) -> Vec<String> {
        settings["hooks"][event]
            .as_array()
            .expect("event array")
            .iter()
            .flat_map(|group| group["hooks"].as_array().expect("hooks array"))
            .map(|command| command["command"].as_str().expect("command").to_owned())
            .collect()
    }

    #[test]
    fn registers_every_entry_once_in_an_empty_file() {
        let settings = registered();

        for entry in ENTRIES {
            assert_eq!(commands(&settings, entry.event), vec![entry.command]);
        }
    }

    #[test]
    fn preserves_unrelated_keys_and_unrelated_hooks() {
        let existing = json!({
            "permissions": {"allow": ["Bash(ls:*)"]},
            "hooks": {
                "Stop": [{"hooks": [{"type": "command", "command": "make lint"}]}],
                "PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "audit"}]}],
            },
        });

        let settings = register(&existing);

        assert_eq!(settings["permissions"], existing["permissions"]);
        assert_eq!(
            settings["hooks"]["PreToolUse"],
            existing["hooks"]["PreToolUse"]
        );
        assert_eq!(
            commands(&settings, "Stop"),
            vec!["make lint", "openspec-doc"]
        );
        assert_eq!(
            settings["hooks"]["Stop"][1]["hooks"][0]["args"],
            json!(["hook", "stop", "--agent", "claude"])
        );
    }

    #[test]
    fn replaces_legacy_entries_that_are_ours() {
        for command in [
            "openspec-doc hook stop --agent claude",
            "/tmp/target/debug/openspec-doc hook stop --agent claude",
        ] {
            let existing = json!({
                "hooks": {"Stop": [{"hooks": [{"type": "command", "command": command}]}]},
            });

            let settings = register(&existing);

            assert_eq!(
                commands(&settings, "Stop"),
                vec!["openspec-doc"],
                "{command} was not recognised as ours"
            );
        }
    }

    #[test]
    fn drops_a_group_our_entry_emptied() {
        let existing = json!({
            "hooks": {"Stop": [{"hooks": [{"type": "command", "command": "openspec-doc hook stop --agent claude"}]}]},
        });

        let settings = register(&existing);

        assert_eq!(
            settings["hooks"]["Stop"].as_array().expect("array").len(),
            1
        );
    }

    #[test]
    fn registering_twice_is_the_same_file() {
        let once = registered();

        assert_eq!(register(&once), once);
    }

    /// The block in the docs is what someone wires by hand when they do not run
    /// `init`. Two hand-maintained copies of the same JSON drift, and the drift
    /// is invisible until someone follows the stale one.
    #[test]
    fn matches_the_json_documented_for_manual_setup() {
        for doc in [
            "../../docs/docs/reference/hooks.md",
            "../../docs/docs/quickstart.md",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(doc);
            let markdown = std::fs::read_to_string(&path).expect("read the doc");
            let documented: Value = serde_json::from_str(&first_json_block(&markdown))
                .expect("parse the documented JSON");

            assert_eq!(documented, registered(), "{doc} has drifted");
        }
    }

    fn first_json_block(markdown: &str) -> String {
        markdown
            .split("```json\n")
            .nth(1)
            .and_then(|rest| rest.split("\n```").next())
            .expect("a fenced json block")
            .to_owned()
    }
}
