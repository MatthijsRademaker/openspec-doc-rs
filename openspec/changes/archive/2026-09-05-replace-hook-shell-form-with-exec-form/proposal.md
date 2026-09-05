## Why

`doctor` is wrong on Windows in two ways, and the obvious fix for the second one is worse than the defect.

`crates/cli/src/doctor/binary.rs:15` declares `const BINARY: &str = "openspec-doc"` and joins that name onto each `PATH` entry to find the binary an agent would run. On Windows the file is `openspec-doc.exe`, so the lookup misses and `doctor` reports a correctly installed binary as absent from `PATH` — which it then treats as a failing check, *"since every hook command depends on it"*. The one command whose purpose is telling a new user whether their setup works answers wrongly, on their first run, on a working install.

`crates/cli/src/doctor/probe.rs:176` spawns `sh -c <command>` unconditionally, to run each hook the way the agent does. Stock Windows has no `sh`. The naive fix is to make the probe pick a shell per platform, and Claude Code's documentation says what that would have to be: *"The `command` string is passed to a shell: `sh -c` on macOS and Linux, Git Bash on Windows, or PowerShell when Git Bash isn't installed."* That is a three-way branch with a fallback condition, plus a per-hook `shell` field that can override all of it — more code in the checker than in the thing checked, and the capability's own standard is that a probe must run the hook *as the agent runs it*, so an approximation of that branch is a probe that can pass on an install the agent cannot use.

The same documentation offers a way out. Hooks have a second form: *"Exec form (when `args` is set): Claude Code resolves `command` as an executable on `PATH` and spawns it directly with `args` as the argument vector. There is no shell."* Anthropic recommends preferring it. Written that way, `wiring.rs` emits a command and an argument vector, and the probe's faithful reproduction is `Command::new(binary).args(...)` — **less** code than the `sh -c` it replaces, with no platform branch anywhere. The Windows shell problem is not solved. It stops existing.

There is a second reason, and it is the stronger one. Claude Code has an **open, unfixed defect** in shell-form hook execution on Windows: it detects the full Git Bash path at startup and then invokes hooks with a bare `bash`, which fails whenever `Git\bin` is not on `PATH` — and Git for Windows' default install puts only `Git\cmd` there. Reported as `anthropics/claude-code#22700` and **closed as not planned, labelled stale**; a sibling report has Git Bash misreading Windows backslash paths as escape sequences. Shell-form hooks on Windows are not merely harder to check, they are upstream-broken with no fix coming. Exec form spawns no shell and is exposed to neither.

## What Changes

- Register Claude Code hooks in **exec form**: a `command` naming the executable and an `args` vector, instead of one shell string. This is the change; everything else follows from it.
- Replace the predicate that recognises this tool's entries. `wiring.rs:58` matches the substring `openspec-doc hook ` against the command string, and under exec form no such string exists. Recognition becomes a check on the executable plus the leading arguments.
- Drop the shell from `doctor`'s hook probe. The probe spawns the recorded executable with the recorded arguments, on every platform, with the payload on stdin as now.
- Resolve the binary on `PATH` with the platform's executable suffix, fixing the Windows lookup.
- **Require a Claude Code new enough to have exec form, and check it.** `args` arrived in Claude Code 2.1.139. An older one ignores an unrecognised key and runs `command` as a shell string — bare `openspec-doc`, which is a clap usage error, which exits 2, which Claude Code treats as a block. That is the failure this repository already records as *"a typo in `hook prompt`'s command string is unrecoverable"*: one bad entry refusing every prompt in the session. `doctor` gains a check for the agent's version, because a requirement nothing verifies is a requirement that will be discovered by a user.
- Un-gate the three `#[cfg(unix)]` tests in `crates/cli/tests/cli.rs`. The gating is test-harness mechanics, not a product limit: `path_with_binary` uses `std::os::unix::fs::symlink`, and `doctor()` builds a `PATH` with a hardcoded `:`. A file copy and `std::env::join_paths` remove both.
- Add the Windows leg to the automated gate run, and add Windows to the recorded set of covered platforms. `add-continuous-verification` deliberately left it out because a Windows run today would be green while omitting exactly these three tests; this change is what makes the run honest, so it is the change that adds it.

**A capability is lost, deliberately.** `setup-diagnostics` currently requires the probe to execute *the command string as recorded*, so that a developer pointing a hook at a local build with `OPENSPEC_DOC_BIN=… openspec-doc hook stop` is probed on the binary they are actually running. An environment-variable prefix is shell syntax and exec form has no shell. The replacement is an absolute path in `command`, which is arguably clearer and is strictly less expressive. The requirement and its scenario change accordingly rather than being quietly left to fail.

Deliberately **not** in scope: the pi.dev extension, which invokes the CLI directly and never went through a shell; migrating an already-wired project's settings file, since `init` rewrites its own entries and this repository does not preserve old formats; and the `UserPromptExpansion` matcher, which is unaffected.

## Capabilities

### Modified Capabilities

- `project-init`: hook registration is specified today as three named shell command strings. It becomes registration in the form the agent executes without a shell, and the predicate for recognising an existing entry changes with it.
- `setup-diagnostics`: the probe stops going through a shell, the binary-on-`PATH` check becomes correct on platforms where executables carry a suffix, and a new check covers the agent being new enough for the form the hooks are registered in.
- `repository-verification-gates`: Windows joins the covered platforms, and the omission `add-continuous-verification` recorded is removed rather than left standing.

## Impact

- `crates/cli/src/wiring.rs` — the three command definitions, `COMMAND_MARKER`, and the entry-recognition predicate. This is the file that decides whether a project counts as already wired, so a wrong predicate here duplicates hook entries or silently fails to replace them.
- `crates/cli/src/doctor/probe.rs` — `sh -c` and everything that depends on the command being one string.
- `crates/cli/src/doctor/binary.rs` — `BINARY`, and `is_executable`, whose non-Unix arm currently accepts any file.
- `crates/cli/src/doctor/` — a new check for the agent's version.
- `crates/cli/tests/cli.rs` — three tests and three helpers.
- `.claude/settings.json` in this repository, which `init` manages and which will be rewritten.
- `README.md`, `AGENTS.md`, `docs/docs/reference/hooks.md`, `docs/docs/quickstart.md` — every place a hook is quoted as `openspec-doc hook stop --agent claude` describes a form that no longer exists.
- Claude Code 2.1.139 becomes a stated minimum. That is a new external requirement on users and belongs in the quickstart, not only in a spec.

**Depends on `add-continuous-verification`**, which establishes the lane this change adds a platform to. **Overlaps `add-pi-explore-detection`**, which also edits `openspec/specs/project-init/spec.md`; the two touch different requirements, but whichever lands second rebases.
