//! Which `openspec-doc` the agent will actually run.
//!
//! Every hook command in the settings file names the binary by bare name, so the
//! agent runs whichever one `PATH` resolves. An operator working in this
//! repository routinely has an installed copy on `PATH` and a newer one under
//! `target/`, and a check run from the newer binary that reports on the older
//! one's behalf is answering the wrong question.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::Outcome;

const BINARY: &str = "openspec-doc";

/// The version this executable was built as. Read from the build rather than by
/// running ourselves, which would be a subprocess to learn a compile-time fact.
const RUNNING_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn check() -> Outcome {
    let running = match std::env::current_exe().and_then(|path| path.canonicalize()) {
        Ok(path) => path,
        Err(source) => {
            return Outcome::Fail(format!(
                "could not locate the running executable, so the binary on PATH cannot be \
                 compared against it: {source}"
            ));
        }
    };

    let Some(on_path) = resolve(std::env::var_os("PATH").as_deref()) else {
        return Outcome::Fail(format!(
            "no `{BINARY}` on PATH. Every hook command names the binary by bare name, so with \
             nothing to resolve, none of them can run however the settings file reads. This \
             executable is {}",
            running.display()
        ));
    };

    if on_path == running {
        return Outcome::Pass(format!(
            "{} {RUNNING_VERSION}, which is this executable",
            on_path.display()
        ));
    }

    Outcome::Fail(format!(
        "the agent runs a different binary from this one, so this report would be about a \
         binary the hooks never invoke\n  on PATH:  {} {}\n  running:  {} {RUNNING_VERSION}",
        on_path.display(),
        version(&on_path),
        running.display()
    ))
}

/// The first executable `openspec-doc` in `path_var`, canonicalized.
///
/// Taken as an argument rather than read here so it can be tested: setting the
/// process environment is unsound with other tests running beside it.
fn resolve(path_var: Option<&OsStr>) -> Option<PathBuf> {
    std::env::split_paths(path_var?)
        .map(|dir| dir.join(BINARY))
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

/// What `binary --version` says, so the report names the two binaries by
/// something more useful than their paths.
fn version(binary: &Path) -> String {
    let Ok(output) = Command::new(binary).arg("--version").output() else {
        return "(would not run)".to_owned();
    };

    let reported = String::from_utf8_lossy(&output.stdout);
    match reported.split_whitespace().last() {
        Some(version) if output.status.success() => version.to_owned(),
        _ => "(no version)".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[cfg(unix)]
    fn executable(dir: &Path, name: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\n").expect("write");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
        path
    }

    #[cfg(unix)]
    #[test]
    fn the_first_executable_on_the_path_wins() {
        let first = TempDir::new().expect("temp dir");
        let second = TempDir::new().expect("temp dir");
        let expected = executable(first.path(), BINARY);
        executable(second.path(), BINARY);

        let resolved = resolve(Some(
            std::env::join_paths([first.path(), second.path()])
                .expect("join")
                .as_os_str(),
        ));

        assert_eq!(resolved, Some(expected.canonicalize().expect("canonical")));
    }

    #[cfg(unix)]
    #[test]
    fn a_directory_without_the_binary_resolves_nothing() {
        let empty = TempDir::new().expect("temp dir");
        fs::write(empty.path().join("unrelated"), "").expect("write");

        assert_eq!(
            resolve(Some(empty.path().as_os_str())),
            None,
            "only an executable named {BINARY} counts"
        );
    }

    #[test]
    fn no_path_at_all_resolves_nothing() {
        assert_eq!(resolve(None), None);
    }
}
