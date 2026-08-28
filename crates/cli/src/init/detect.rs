//! Which harnesses a project uses, and whether Claude Code can reach the
//! exploration command the expansion hook is matched on.

use std::path::Path;

use openspec_doc_core::hook::Agent;

/// The harnesses this project shows evidence of using.
///
/// Directory presence, deliberately, rather than an attempt to read what
/// `openspec init` recorded: that marker format is upstream's to change, and a
/// detector coupled to it fails by silently configuring nothing. This is coarse
/// and over-detects, which is the direction to be wrong in — the plan names what
/// it found before anything is written, and `--agent` overrides it.
pub fn harnesses(root: &Path) -> Vec<Agent> {
    [(Agent::Claude, ".claude"), (Agent::Pi, ".pi")]
        .into_iter()
        .filter(|(_, dir)| root.join(dir).is_dir())
        .map(|(agent, _)| agent)
        .collect()
}

/// Where a project can provide the command `UserPromptExpansion` dispatches on.
/// Both `.claude/commands/` layouts, and the skill directory.
const EXPLORE_COMMANDS: [&str; 3] = [
    ".claude/commands/opsx/explore.md",
    ".claude/commands/openspec-explore.md",
    ".claude/skills/openspec-explore/SKILL.md",
];

/// Whether anything in the project answers to `opsx:explore` or
/// `openspec-explore`.
///
/// The matcher is written either way; a configuration that is complete and
/// warned beats one that is silently a third missing. But a matcher with no
/// command behind it never fires and reports nothing, which is exactly the
/// silence this tool exists to remove, so the hole gets named.
pub fn explore_command(root: &Path) -> bool {
    EXPLORE_COMMANDS
        .iter()
        .any(|command| root.join(command).is_file())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn project(dirs: &[&str]) -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        for dir in dirs {
            fs::create_dir_all(temp.path().join(dir)).expect("create dir");
        }
        temp
    }

    #[test]
    fn detects_each_harness_by_its_directory() {
        assert_eq!(harnesses(project(&[".claude"]).path()), [Agent::Claude]);
        assert_eq!(harnesses(project(&[".pi"]).path()), [Agent::Pi]);
        assert_eq!(
            harnesses(project(&[".claude", ".pi"]).path()),
            [Agent::Claude, Agent::Pi]
        );
        assert!(harnesses(project(&[]).path()).is_empty());
    }

    #[test]
    fn finds_the_explore_command_in_either_layout() {
        for command in EXPLORE_COMMANDS {
            let temp = project(&[]);
            let path = temp.path().join(command);
            fs::create_dir_all(path.parent().expect("parent")).expect("create dir");
            fs::write(&path, "").expect("write command");

            assert!(explore_command(temp.path()), "{command} not found");
        }

        assert!(!explore_command(project(&[".claude/commands"]).path()));
    }
}
