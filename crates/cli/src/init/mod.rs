//! `init`: take a project from nothing to wired.
//!
//! Dry run by default. Writing immediately surprises the operator, and prompting
//! when a terminal is attached puts the interactive path outside the reach of the
//! integration tests — so the default is a plan, and `--yes` performs it. The
//! cost is a second invocation, which is the right price for a command that
//! appends to someone's `AGENTS.md`.

mod detect;
mod instructions;
mod plan;

use std::fs;
use std::io;
use std::path::Path;

use openspec_doc_core::Project;
use openspec_doc_core::hook::Agent;
use serde_json::Value;

use crate::error::Error;
use crate::wiring;

use plan::{Plan, Write};

/// The one copy of pi's half of the bridge. Every project's `.pi/extensions/`
/// copy is this file's output, including this repository's own, so a divergence
/// shows up as a diff rather than as a defect in a project nobody is looking at.
const PI_EXTENSION: &str = include_str!("../../assets/openspec-doc-hook.ts");

pub fn run(
    project: Project,
    agents: Vec<Agent>,
    yes: bool,
    skip_instructions: bool,
) -> Result<(), Error> {
    let plan = build(
        &project.root,
        select(&project.root, agents)?,
        skip_instructions,
    )?;

    print!("{}", plan.render());

    if yes {
        print!("{}", plan.perform()?);
    } else {
        println!("\nnothing written. re-run with --yes to perform these writes.");
    }

    Ok(())
}

/// Which harnesses to configure: what was named, or failing that, what the
/// project shows evidence of.
///
/// Named replaces detected outright rather than adding to it. `openspec init
/// --tools` already asked "which coding agents?" in this same directory; asking
/// it a second time is a worse command than reading the answer off disk, and an
/// operator who overrides that answer means the override.
fn select(root: &Path, agents: Vec<Agent>) -> Result<Vec<Agent>, Error> {
    let mut selected = if agents.is_empty() {
        detect::harnesses(root)
    } else {
        agents
    };
    selected.dedup();

    if selected.is_empty() {
        return Err(Error::NoHarnessDetected);
    }

    Ok(selected)
}

fn build(root: &Path, agents: Vec<Agent>, skip_instructions: bool) -> Result<Plan, Error> {
    let mut writes = Vec::new();
    let mut warnings = Vec::new();

    for agent in &agents {
        match agent {
            Agent::Claude => {
                writes.push(claude_settings(root)?);

                if !detect::explore_command(root) {
                    warnings.push(
                        "no opsx:explore or openspec-explore command found, so the \
                         UserPromptExpansion hook will never fire and no exploration note will \
                         be started. Install the commands with `openspec init --tools claude`."
                            .to_owned(),
                    );
                }
            }
            Agent::Pi => {
                writes.push(Write::new(
                    root,
                    root.join(".pi/extensions/openspec-doc-hook.ts"),
                    "pi's half of the hook bridge, from the copy embedded in this binary",
                    PI_EXTENSION.to_owned(),
                ));
            }
        }
    }

    if !skip_instructions {
        let path = root.join("AGENTS.md");
        let existing = read(&path)?;
        let contents = instructions::install(&path, existing.as_deref())?;

        writes.push(Write::new(
            root,
            path,
            "the managed openspec-doc block, between its markers",
            contents,
        ));
    }

    Ok(Plan {
        root: root.to_owned(),
        agents,
        writes,
        warnings,
    })
}

/// The settings file with our hook entries registered in it.
///
/// A settings file that does not parse is a hard error naming the file. It is
/// someone's configuration, and the only two things to do with one we cannot
/// read are overwrite it and stop.
fn claude_settings(root: &Path) -> Result<Write, Error> {
    let path = root.join(".claude/settings.json");
    let existing = match read(&path)? {
        Some(text) => serde_json::from_str(&text).map_err(|source| Error::Settings {
            path: path.clone(),
            source,
        })?,
        None => Value::Null,
    };

    let contents = format!(
        "{}\n",
        serde_json::to_string_pretty(&wiring::register(&existing))
            .expect("a settings object serializes")
    );

    Ok(Write::new(
        root,
        path,
        "the Stop, UserPromptSubmit, and UserPromptExpansion hook entries",
        contents,
    ))
}

fn read(path: &Path) -> Result<Option<String>, Error> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::Read {
            path: path.to_owned(),
            source,
        }),
    }
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

    fn plan(root: &Path, agents: Vec<Agent>) -> Plan {
        build(root, select(root, agents).expect("select"), false).expect("build")
    }

    #[test]
    fn named_agents_replace_what_was_detected() {
        let temp = project(&[".claude"]);

        let plan = plan(temp.path(), vec![Agent::Pi]);

        assert_eq!(plan.agents, [Agent::Pi]);
        assert!(
            !plan.render().contains("settings.json"),
            "{}",
            plan.render()
        );
    }

    #[test]
    fn repeated_agents_are_configured_once() {
        let temp = project(&[]);

        let plan = plan(temp.path(), vec![Agent::Pi, Agent::Pi]);

        assert_eq!(plan.agents, [Agent::Pi]);
    }

    #[test]
    fn nothing_detected_and_nothing_named_is_an_error() {
        let temp = project(&[]);

        assert!(matches!(
            select(temp.path(), Vec::new()),
            Err(Error::NoHarnessDetected)
        ));
    }

    #[test]
    fn a_dry_run_leaves_every_file_alone() {
        let temp = project(&[".claude", ".pi"]);
        fs::write(temp.path().join("AGENTS.md"), "# Rules\n").expect("write");

        let rendered = plan(temp.path(), Vec::new()).render();

        assert!(rendered.contains("harnesses: claude, pi"), "{rendered}");
        assert!(!temp.path().join(".claude/settings.json").exists());
        assert_eq!(
            fs::read_to_string(temp.path().join("AGENTS.md")).expect("read"),
            "# Rules\n"
        );
    }

    #[test]
    fn performing_twice_writes_the_same_bytes() {
        let temp = project(&[".claude", ".pi"]);

        plan(temp.path(), Vec::new()).perform().expect("first");
        let after_first: Vec<String> = managed(temp.path());

        let second = plan(temp.path(), Vec::new());
        let report = second.perform().expect("second");

        assert_eq!(managed(temp.path()), after_first);
        assert_eq!(report.matches("unchanged").count(), 3, "{report}");
    }

    fn managed(root: &Path) -> Vec<String> {
        [
            ".claude/settings.json",
            ".pi/extensions/openspec-doc-hook.ts",
            "AGENTS.md",
        ]
        .iter()
        .map(|path| fs::read_to_string(root.join(path)).expect("read a managed file"))
        .collect()
    }

    #[test]
    fn a_malformed_settings_file_fails_without_a_write() {
        let temp = project(&[".claude"]);
        let path = temp.path().join(".claude/settings.json");
        fs::write(&path, "{ not json").expect("write");

        assert!(matches!(
            build(temp.path(), vec![Agent::Claude], false),
            Err(Error::Settings { .. })
        ));
        assert_eq!(fs::read_to_string(&path).expect("read"), "{ not json");
    }

    #[test]
    fn a_project_without_the_explore_command_is_warned_and_still_gets_the_matcher() {
        let temp = project(&[".claude"]);

        let plan = plan(temp.path(), Vec::new());
        let rendered = plan.render();

        assert!(
            rendered.contains("openspec init --tools claude"),
            "{rendered}"
        );
        plan.perform().expect("perform");
        assert!(
            fs::read_to_string(temp.path().join(".claude/settings.json"))
                .expect("read")
                .contains("opsx:explore|openspec-explore")
        );
    }

    #[test]
    fn a_project_with_the_explore_command_is_not_warned_about_it() {
        let temp = project(&[".claude/commands/opsx"]);
        fs::write(temp.path().join(".claude/commands/opsx/explore.md"), "").expect("write");

        let rendered = plan(temp.path(), Vec::new()).render();

        assert!(
            !rendered.contains("openspec init --tools claude"),
            "{rendered}"
        );
    }

    /// The extension captures pi explorations, so warning about them would be a
    /// warning about a capability the tool has. A reappearing warning fails here.
    #[test]
    fn configuring_pi_reports_no_exploration_gap() {
        let temp = project(&[".pi"]);

        let rendered = plan(temp.path(), Vec::new()).render();

        assert!(!rendered.contains("exploration note"), "{rendered}");
    }

    #[test]
    fn a_differing_pi_extension_is_replaced_and_an_identical_one_is_not() {
        let temp = project(&[".pi/extensions"]);
        let path = temp.path().join(".pi/extensions/openspec-doc-hook.ts");
        fs::write(&path, "// stale\n").expect("write");

        let rendered = plan(temp.path(), Vec::new()).render();
        assert!(
            rendered.contains("modify    .pi/extensions/openspec-doc-hook.ts"),
            "{rendered}"
        );

        plan(temp.path(), Vec::new()).perform().expect("perform");
        assert_eq!(fs::read_to_string(&path).expect("read"), PI_EXTENSION);
        assert!(
            plan(temp.path(), Vec::new())
                .render()
                .contains("unchanged .pi/extensions/openspec-doc-hook.ts")
        );
    }

    #[test]
    fn skip_instructions_plans_no_write_to_the_instruction_file() {
        let temp = project(&[".claude"]);

        let plan = build(temp.path(), vec![Agent::Claude], true).expect("build");

        assert!(!plan.render().contains("AGENTS.md"), "{}", plan.render());
        plan.perform().expect("perform");
        assert!(!temp.path().join("AGENTS.md").exists());
    }

    #[test]
    fn a_half_delimited_instruction_file_fails_without_a_write() {
        let temp = project(&[".claude"]);
        let path = temp.path().join("AGENTS.md");
        fs::write(&path, format!("{}\nno end\n", instructions::BEGIN)).expect("write");

        assert!(matches!(
            build(temp.path(), vec![Agent::Claude], false),
            Err(Error::HalfDelimitedInstructions { .. })
        ));
        assert!(!temp.path().join(".claude/settings.json").exists());
    }
}
