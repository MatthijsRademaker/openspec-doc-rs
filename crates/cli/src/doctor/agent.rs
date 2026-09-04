//! Checking whether Claude Code can execute hook argument vectors.
//!
//! Claude Code 2.1.139 added the `args` hook field. An older agent ignores it
//! and runs the executable with no arguments, turning every configured hook into
//! a usage error, so its version is part of the setup report.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::Outcome;

const AGENT: &str = "claude";
const MINIMUM: Version = Version {
    major: 2,
    minor: 1,
    patch: 139,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl std::fmt::Display for Version {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Check the resolved Claude Code version without treating an absent agent as a
/// failure: projects using only another harness have no Claude executable to
/// check.
pub fn check() -> Outcome {
    check_path(std::env::var_os("PATH").as_deref())
}

fn check_path(path_var: Option<&OsStr>) -> Outcome {
    let Some(agent) = resolve(path_var) else {
        return Outcome::NotChecked(
            "`claude` was not found on PATH, so Claude Code's hook argument-vector support was not \
             checked."
                .to_owned(),
        );
    };

    let output = match Command::new(&agent).arg("--version").output() {
        Ok(output) => output,
        Err(error) => {
            return Outcome::Fail(format!(
                "Claude Code at {} could not report its version: {error}",
                agent.display()
            ));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let reported = format!("{stdout} {stderr}");
    let Some(version) = parse_version(&reported) else {
        return Outcome::Fail(format!(
            "Claude Code at {} returned no parseable version from `claude --version`\n  stdout: {}\n  \
             stderr: {}",
            agent.display(),
            quote(stdout.trim()),
            quote(stderr.trim())
        ));
    };

    if !output.status.success() {
        return Outcome::Fail(format!(
            "Claude Code at {} reported version {version} but `claude --version` failed\n  \
             stderr: {}",
            agent.display(),
            quote(stderr.trim())
        ));
    }

    if version < MINIMUM {
        return Outcome::Fail(format!(
            "Claude Code {version} at {} is too old for hook argument vectors; requires {MINIMUM} \
             or newer. Run `claude update` to upgrade.",
            agent.display()
        ));
    }

    Outcome::Pass(format!(
        "Claude Code {version} at {} supports hook argument vectors (minimum {MINIMUM})",
        agent.display()
    ))
}

fn resolve(path_var: Option<&OsStr>) -> Option<PathBuf> {
    std::env::split_paths(path_var?)
        .map(|dir| dir.join(format!("{AGENT}{}", std::env::consts::EXE_SUFFIX)))
        .filter(|candidate| is_executable(candidate))
        .find_map(|candidate| candidate.canonicalize().ok())
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn parse_version(output: &str) -> Option<Version> {
    output.split_whitespace().find_map(|token| {
        let token =
            token.trim_matches(|character: char| !character.is_ascii_digit() && character != '.');
        let parts = token
            .split('.')
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        match parts.as_slice() {
            [major, minor, patch] => Some(Version {
                major: *major,
                minor: *minor,
                patch: *patch,
            }),
            _ => None,
        }
    })
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
    use std::fs;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

    fn openspec_doc() -> PathBuf {
        if let Some(path) = std::env::var_os("CARGO_BIN_EXE_openspec-doc") {
            return path.into();
        }

        std::env::current_exe()
            .expect("test executable")
            .parent()
            .expect("deps directory")
            .parent()
            .expect("target directory")
            .join(format!("openspec-doc{}", std::env::consts::EXE_SUFFIX))
    }

    fn copied_claude() -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        fs::copy(
            openspec_doc(),
            temp.path()
                .join(format!("claude{}", std::env::consts::EXE_SUFFIX)),
        )
        .expect("copy openspec-doc as claude");
        temp
    }

    #[test]
    fn an_old_agent_fails_with_the_upgrade_details() {
        let agent = copied_claude();

        let outcome = check_path(Some(agent.path().as_os_str()));
        let Outcome::Fail(detail) = outcome else {
            panic!("{outcome:?}");
        };

        assert!(detail.contains("Claude Code 0.1.0"), "{detail}");
        assert!(detail.contains("requires 2.1.139"), "{detail}");
        assert!(detail.contains("claude update"), "{detail}");
    }

    #[test]
    fn an_absent_agent_is_not_checked() {
        let outcome = check_path(None);
        let Outcome::NotChecked(detail) = outcome else {
            panic!("{outcome:?}");
        };

        assert!(
            detail.contains("`claude` was not found on PATH"),
            "{detail}"
        );
    }

    #[test]
    fn parses_claude_code_version_output() {
        assert_eq!(parse_version("2.1.139 (Claude Code)"), Some(MINIMUM));
        assert_eq!(
            parse_version("Claude Code version 2.1.260"),
            Some(Version {
                major: 2,
                minor: 1,
                patch: 260,
            })
        );
    }

    #[test]
    fn compares_versions_against_the_exec_form_floor() {
        assert!(
            Version {
                major: 2,
                minor: 1,
                patch: 138,
            } < MINIMUM
        );
        assert!(
            MINIMUM
                <= Version {
                    major: 2,
                    minor: 1,
                    patch: 140,
                }
        );
    }

    #[test]
    fn malformed_version_output_is_not_accepted() {
        assert_eq!(parse_version("Claude Code"), None);
        assert_eq!(parse_version("2.1"), None);
    }
}
