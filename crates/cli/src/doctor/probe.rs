//! Running a configured hook against a project that is not this one.
//!
//! `hook stop` is not read-only: it registers the session, runs the promotion
//! check, and *consumes any pending directive*. A probe against the real project
//! would silently eat the reviewer's outstanding feedback, making `doctor` a
//! cause of the failure it exists to diagnose. So the probes run with their
//! working directory set to a throwaway OpenSpec project built here, which is
//! what root discovery lands on.
//!
//! The recorded command string is executed verbatim through `sh -c`, because
//! that is how the agent runs it. Nothing is spliced into it — which is why an
//! entry that already names a `--root` is refused rather than rewritten.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use openspec_doc_core::comments::ScopeKey;
use openspec_doc_core::dashboard::STATE_DIR_ENV;
use openspec_doc_core::hook::directive_reason;
use openspec_doc_core::verdict::{self, Verdict};
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::error::Error;

use super::Outcome;

/// One session per hook, all in the same throwaway root.
///
/// `hook explore` writes a scratch note for the session it is given, and `hook
/// stop` starts a dashboard for a session that has one. Sharing an id between
/// the two would have the stop probe spawn a server for the note the explore
/// probe just left, in whichever order they ran.
const STOP_SESSION: &str = "openspec-doc-doctor-stop";
const PROMPT_SESSION: &str = "openspec-doc-doctor-prompt";
const EXPLORE_SESSION: &str = "openspec-doc-doctor-explore";

/// The notes on the seeded verdict, so a probe root that somehow outlives the
/// run says what put it there.
const SEED_NOTES: &str = "seeded by `openspec-doc doctor` to probe the prompt hook";

pub struct Probe {
    root: TempDir,
    /// The port registry the probes write to. Assigning a port is a side effect
    /// of the explore hook, and a temp root would otherwise take an assignment
    /// out of the operator's own registry on every run.
    state: TempDir,
    /// The text the seeded verdict makes `hook prompt` emit.
    directive: String,
}

impl Probe {
    /// A minimal OpenSpec project with a standing verdict for the prompt probe's
    /// session.
    ///
    /// The verdict is seeded through the same writers the dashboard uses rather
    /// than as literal JSON, so a directive-format change breaks against an
    /// internal API it already has to update.
    pub fn new() -> Result<Self, Error> {
        let root = TempDir::new().map_err(|source| Error::ProbeRoot { source })?;
        let state = TempDir::new().map_err(|source| Error::ProbeRoot { source })?;

        let openspec = root.path().join("openspec");
        std::fs::create_dir_all(&openspec).map_err(|source| Error::Write {
            path: openspec.clone(),
            source,
        })?;
        let config = openspec.join("config.yaml");
        std::fs::write(&config, "").map_err(|source| Error::Write {
            path: config,
            source,
        })?;

        let key = ScopeKey::Session(PROMPT_SESSION.to_owned());
        verdict::add(root.path(), &key, Verdict::KeepExploring, SEED_NOTES)?;

        Ok(Self {
            root,
            state,
            directive: directive_reason(&key, Verdict::KeepExploring)?,
        })
    }

    /// Run `command` as the hook registered under `event`, and judge what came
    /// back by what that hook has to answer with.
    pub fn check(&self, event: &str, command: &str) -> Outcome {
        if command.contains("--root") {
            return Outcome::Refused(format!(
                "`{command}` names a project root of its own, so probing it would run against \
                 that project and consume any directive pending for the probe session there. \
                 Drop the option — the hook resolves the root from the session's own directory — \
                 or check this entry by hand."
            ));
        }

        let session = match event {
            "Stop" => STOP_SESSION,
            "UserPromptSubmit" => PROMPT_SESSION,
            "UserPromptExpansion" => EXPLORE_SESSION,
            other => {
                return Outcome::Fail(format!(
                    "no probe is defined for the {other} hook, so `doctor` and the shared wiring \
                     definition have drifted"
                ));
            }
        };

        let output = match self.run(command, session) {
            Ok(output) => output,
            Err(error) => return Outcome::Fail(super::chain(&error)),
        };

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if !output.status.success() {
            return Outcome::Fail(format!(
                "`{command}` {}\n{}",
                super::status(&output.status),
                transcript(&stdout, &output)
            ));
        }

        self.judge(event, command, &stdout, &output)
    }

    /// Whether the hook's stdout is the answer that hook exists to give.
    fn judge(&self, event: &str, command: &str, stdout: &str, output: &Output) -> Outcome {
        match event {
            "Stop" => match serde_json::from_str::<Value>(stdout.trim()) {
                Ok(decision) if decision.is_object() => {
                    Outcome::Pass(format!("`{command}` answered {}", stdout.trim()))
                }
                _ => Outcome::Fail(format!(
                    "`{command}` exited zero without a decision payload on stdout, which is what \
                     the agent reads to decide whether the turn may end\n{}",
                    transcript(stdout, output)
                )),
            },
            "UserPromptExpansion" => {
                if stdout.trim().is_empty() {
                    Outcome::Fail(format!(
                        "`{command}` exited zero and printed nothing, so an exploration would be \
                         given no note to write in\n{}",
                        transcript(stdout, output)
                    ))
                } else {
                    Outcome::Pass(format!("`{command}` answered {}", summarize(stdout)))
                }
            }
            // Silence cannot be the pass condition: this hook exits zero and
            // prints nothing on failure by design, so an empty session would
            // probe clean with no binary installed at all. The probe root has a
            // standing verdict, and the directive it translates into is what the
            // hook must not stay silent about.
            "UserPromptSubmit" => {
                if stdout.contains(&self.directive) {
                    Outcome::Pass(format!(
                        "`{command}` delivered the directive seeded for the probe session"
                    ))
                } else {
                    Outcome::Fail(format!(
                        "`{command}` did not deliver the directive seeded for the probe session. \
                         This hook exits zero and prints nothing when it fails, so its silence is \
                         the failure\n{}",
                        transcript(stdout, output)
                    ))
                }
            }
            other => Outcome::Fail(format!("no probe is defined for the {other} hook")),
        }
    }

    /// Execute `command` the way the agent does: through a shell, with the
    /// event's payload on stdin.
    fn run(&self, command: &str, session: &str) -> Result<Output, Error> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(self.root.path())
            .env(STATE_DIR_ENV, self.state.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| Error::Probe {
                command: command.to_owned(),
                source,
            })?;

        // A command that does not exist exits before reading, so a failed write
        // is the child's exit rather than a problem of ours; the exit status and
        // its stderr are what the report is about. Taking stdin here also closes
        // it, which is what lets a hook that does read reach end of input.
        let _ = child
            .stdin
            .take()
            .expect("stdin was piped")
            .write_all(payload(session, self.root.path()).as_bytes());

        child.wait_with_output().map_err(|source| Error::Probe {
            command: command.to_owned(),
            source,
        })
    }
}

/// A synthetic Claude Code payload carrying the fields every one of its events
/// has. The entries in the shared definition are Claude's; an entry edited to
/// speak another agent's format fails here loudly, which is correct.
fn payload(session: &str, root: &Path) -> String {
    json!({
        "session_id": session,
        "transcript_path": root.join("doctor-probe.jsonl"),
        "cwd": root,
    })
    .to_string()
}

/// The head of a hook's answer, for a report line rather than a transcript. The
/// explore hook's instruction runs to several hundred characters, and printing
/// it whole buries every other check.
fn summarize(stdout: &str) -> String {
    const LIMIT: usize = 96;

    let line = stdout.lines().next().unwrap_or_default().trim();
    match line.char_indices().nth(LIMIT) {
        Some((cut, _)) => format!("{}…", &line[..cut]),
        None => line.to_owned(),
    }
}

/// What the probe saw, for a report the operator can act on.
fn transcript(stdout: &str, output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "  stdout: {}\n  stderr: {}",
        quote(stdout.trim()),
        quote(stderr.trim())
    )
}

fn quote(text: &str) -> String {
    if text.is_empty() {
        "(empty)".to_owned()
    } else {
        text.replace('\n', "\n          ")
    }
}

#[cfg(test)]
mod tests {
    use openspec_doc_core::hook::load_pending;

    use super::*;

    /// Each hook is probed under its own session, so the explore probe's note
    /// cannot make the stop probe start a dashboard.
    #[test]
    fn each_hook_probes_a_session_of_its_own() {
        let sessions = [STOP_SESSION, PROMPT_SESSION, EXPLORE_SESSION];

        for (index, session) in sessions.iter().enumerate() {
            assert!(
                !sessions[index + 1..].contains(session),
                "{session} is used by two probes"
            );
        }
    }

    #[test]
    fn the_probe_root_is_a_project_with_a_standing_verdict_and_nothing_pending() {
        let probe = Probe::new().expect("probe root");

        assert!(probe.root.path().join("openspec/config.yaml").is_file());
        assert!(
            !probe.directive.is_empty(),
            "the prompt probe needs text to require"
        );
        assert_eq!(
            load_pending(probe.root.path(), PROMPT_SESSION).expect("load"),
            None,
            "the verdict is translated by the hook under test, not seeded as a directive"
        );
    }

    #[test]
    fn a_command_naming_its_own_root_is_refused_rather_than_run() {
        let probe = Probe::new().expect("probe root");

        let outcome = probe.check(
            "Stop",
            "openspec-doc --root /some/project hook stop --agent claude",
        );

        assert!(
            matches!(&outcome, Outcome::Refused(reason) if reason.contains("consume")),
            "{outcome:?}"
        );
    }

    #[test]
    fn a_command_that_does_not_exist_fails_with_its_status_and_stderr() {
        let probe = Probe::new().expect("probe root");

        let outcome = probe.check("Stop", "openspec-doc-that-is-not-installed hook stop");

        let Outcome::Fail(detail) = &outcome else {
            panic!("{outcome:?}");
        };
        assert!(detail.contains("exited"), "{detail}");
        assert!(detail.contains("not found"), "{detail}");
    }

    #[test]
    fn an_empty_answer_fails_every_hook() {
        let probe = Probe::new().expect("probe root");

        for event in ["Stop", "UserPromptSubmit", "UserPromptExpansion"] {
            let outcome = probe.check(event, "true");

            assert!(
                matches!(outcome, Outcome::Fail(_)),
                "{event} passed on silence: {outcome:?}"
            );
        }
    }

    /// The exact shape of the failure the prompt hook's fail-soft creates: a
    /// command that exits zero saying nothing is indistinguishable from a
    /// healthy one unless the probe demands the seeded directive back.
    #[test]
    fn the_prompt_hook_fails_on_a_decision_payload_that_is_not_its_directive() {
        let probe = Probe::new().expect("probe root");

        let outcome = probe.check("UserPromptSubmit", "echo '{\"continue\":true}'");

        assert!(matches!(outcome, Outcome::Fail(_)), "{outcome:?}");
    }

    #[test]
    fn a_hook_answering_what_it_should_passes() {
        let probe = Probe::new().expect("probe root");
        let directive = probe.directive.clone();

        for (event, command) in [
            ("Stop", "echo '{\"continue\":true}'".to_owned()),
            (
                "UserPromptExpansion",
                "echo .openspec-doc/scratch/_session/x.md".to_owned(),
            ),
            ("UserPromptSubmit", format!("cat <<'EOF'\n{directive}\nEOF")),
        ] {
            let outcome = probe.check(event, &command);

            assert!(matches!(outcome, Outcome::Pass(_)), "{event}: {outcome:?}");
        }
    }
}
