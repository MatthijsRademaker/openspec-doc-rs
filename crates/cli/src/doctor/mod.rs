//! `doctor`: run this project's wiring and report what happened.
//!
//! `init` writes the configuration and reports whether it is written. It cannot
//! report whether it *runs*: a perfect settings file for a binary that is not on
//! the agent's `PATH` is exactly as silent as a healthy one, because everything
//! here fails by the absence of an event. So every hook check here executes the
//! command the agent would execute and requires the answer that hook exists to
//! give.
//!
//! Anything this does not examine gets a line saying so. A green line for
//! something never looked at is the failure mode this command exists to remove.

mod agent;
mod binary;
mod probe;
mod settings;

use std::fmt::Write as _;
use std::path::Path;
use std::process::ExitStatus;

use openspec_doc_core::Project;

use crate::error::Error;
use crate::wiring;

/// What one check found.
#[derive(Debug)]
pub enum Outcome {
    Pass(String),
    Fail(String),
    /// Deliberately outside this command's reach, and not a problem.
    NotChecked(String),
    /// Within reach, but running it would do harm. Reported as unchecked and
    /// counted as a failure: the operator asked whether their wiring works and
    /// the honest answer is that this part cannot be settled as it stands.
    Refused(String),
}

impl Outcome {
    fn failed(&self) -> bool {
        matches!(self, Self::Fail(_) | Self::Refused(_))
    }

    fn tag(&self) -> &'static str {
        match self {
            Self::Pass(_) => "pass",
            Self::Fail(_) => "FAIL",
            Self::NotChecked(_) | Self::Refused(_) => "not checked",
        }
    }

    fn detail(&self) -> &str {
        match self {
            Self::Pass(detail)
            | Self::Fail(detail)
            | Self::NotChecked(detail)
            | Self::Refused(detail) => detail,
        }
    }
}

#[derive(Debug)]
struct Check {
    label: String,
    outcome: Outcome,
}

impl Check {
    fn new(label: impl Into<String>, outcome: Outcome) -> Self {
        Self {
            label: label.into(),
            outcome,
        }
    }
}

pub fn run(project: Project) -> Result<(), Error> {
    let checks = collect(&project);

    print!("{}", render(&project, &checks));

    let failed = checks.iter().filter(|check| check.outcome.failed()).count();
    if failed > 0 {
        return Err(Error::DoctorFailed {
            failed,
            total: checks.len(),
        });
    }

    Ok(())
}

/// Every check, run in full before anything is printed, so one failure does not
/// truncate the report.
fn collect(project: &Project) -> Vec<Check> {
    let mut checks = vec![
        Check::new("binary on PATH", binary::check()),
        Check::new("Claude Code version", agent::check()),
    ];
    checks.extend(hooks(&project.root));
    checks.extend(unchecked());

    checks
}

/// One check per hook in the shared definition, whether or not the project
/// registered it.
fn hooks(root: &Path) -> Vec<Check> {
    let settings = match settings::read(root) {
        Ok(settings) => settings,
        Err(error) => {
            return vec![Check::new(
                settings::RELATIVE,
                Outcome::Fail(format!(
                    "{}\nWith the file unreadable there is nothing to say about the hooks; \
                     repair it, or let `openspec-doc init` report the diff.",
                    chain(&error)
                )),
            )];
        }
    };

    let probe = match probe::Probe::new() {
        Ok(probe) => probe,
        Err(error) => {
            return vec![Check::new(
                "probe project",
                Outcome::Fail(format!(
                    "{}\nThe hooks are only checked by running them, and running them against \
                     this project would consume its pending review feedback.",
                    chain(&error)
                )),
            )];
        }
    };

    wiring::ENTRIES
        .iter()
        .map(|entry| {
            let outcome = match settings::command(&settings, entry.event) {
                Some(command) => probe.check(entry.event, command),
                None => Outcome::Fail(format!(
                    "no entry in {}; `openspec-doc init` registers it",
                    settings::RELATIVE
                )),
            };

            Check::new(format!("{} hook", entry.event), outcome)
        })
        .collect()
}

/// The parts of the setup this command does not examine, each with the reason.
fn unchecked() -> Vec<Check> {
    vec![
        Check::new(
            "pi.dev extension",
            Outcome::NotChecked(
                "pi's delivery points are an in-process TypeScript extension rather than a \
                 settings entry, so there is no command string to read and nothing to execute \
                 the way the three above are."
                    .to_owned(),
            ),
        ),
        Check::new(
            "dashboard server",
            Outcome::NotChecked(
                "whether a dashboard is serving this project is a different question from \
                 whether the wiring runs; `openspec-doc serve url` answers it."
                    .to_owned(),
            ),
        ),
        Check::new(
            "UserPromptExpansion matcher",
            Outcome::NotChecked(
                "executing the command bypasses matcher dispatch, so a matcher that never fires \
                 still probes clean. `openspec-doc init` is what keeps its value correct."
                    .to_owned(),
            ),
        ),
    ]
}

/// The report as the operator reads it.
fn render(project: &Project, checks: &[Check]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "project: {}", project.root.display());
    let _ = writeln!(out, "\nchecks ({}):", checks.len());

    for check in checks {
        let _ = writeln!(
            out,
            "  {:<11}  {}: {}",
            check.outcome.tag(),
            check.label,
            indent(check.outcome.detail())
        );
    }

    let failed = checks.iter().filter(|check| check.outcome.failed()).count();
    let _ = writeln!(
        out,
        "\n{} passed, {failed} failed",
        checks.len() - failed - not_checked(checks),
    );

    out
}

fn not_checked(checks: &[Check]) -> usize {
    checks
        .iter()
        .filter(|check| matches!(check.outcome, Outcome::NotChecked(_)))
        .count()
}

/// Continuation lines line up under the detail rather than under the tag.
fn indent(detail: &str) -> String {
    detail.replace('\n', "\n    ")
}

/// An error and everything that caused it, on one line each.
fn chain(error: &dyn std::error::Error) -> String {
    let mut out = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        let _ = write!(out, "\ncaused by: {cause}");
        source = cause.source();
    }

    out
}

/// How a probe's exit is reported, since a signalled process has no code.
fn status(status: &ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exited {code}"),
        None => format!("exited without a status ({status})"),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use openspec_doc_core::project_at;
    use tempfile::TempDir;

    use super::*;

    fn fixture() -> (TempDir, Project) {
        let temp = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp.path().join("openspec")).expect("create openspec dir");
        fs::write(temp.path().join("openspec/config.yaml"), "").expect("write config");
        let project = project_at(temp.path()).expect("project");
        (temp, project)
    }

    #[test]
    fn an_unwired_project_fails_every_hook_naming_init() {
        let (_temp, project) = fixture();

        let checks = hooks(&project.root);

        assert_eq!(checks.len(), wiring::ENTRIES.len());
        for check in &checks {
            let Outcome::Fail(detail) = &check.outcome else {
                panic!("{check:?}");
            };
            assert!(detail.contains("openspec-doc init"), "{detail}");
        }
    }

    #[test]
    fn a_malformed_settings_file_is_reported_rather_than_read_as_unregistered() {
        let (temp, project) = fixture();
        let path = settings::path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("create .claude");
        fs::write(path, "{ not json").expect("write settings");

        let checks = hooks(&project.root);

        assert_eq!(checks.len(), 1);
        assert!(checks[0].outcome.failed(), "{:?}", checks[0]);
        assert!(
            checks[0].outcome.detail().contains("not valid JSON"),
            "{:?}",
            checks[0]
        );
    }

    /// Every check is reported, including the ones nothing looked at.
    #[test]
    fn the_report_covers_every_hook_and_every_unchecked_part() {
        let (_temp, project) = fixture();

        let rendered = render(&project, &collect(&project));

        for entry in wiring::ENTRIES {
            assert!(rendered.contains(entry.event), "{rendered}");
        }
        for unchecked in ["pi.dev extension", "dashboard server", "matcher"] {
            assert!(rendered.contains(unchecked), "{rendered}");
        }
        assert!(rendered.contains("Claude Code version"), "{rendered}");
    }
}
