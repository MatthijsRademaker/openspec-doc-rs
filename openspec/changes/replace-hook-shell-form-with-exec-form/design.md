# Design

## Delete the shell rather than emulate it

The problem arrived as "what shell does Claude Code use for hooks on Windows", which is a question with an answer, and the answer is bad:

```text
   shell form                          exec form
   ──────────                          ─────────
   "openspec-doc hook stop            command: "openspec-doc"
    --agent claude"                   args: ["hook","stop","--agent","claude"]

   macOS/Linux  →  sh -c              every platform → spawn directly
   Windows      →  Git Bash                            no shell
   Windows      →  PowerShell
                   (if no Git Bash)
   per-hook `shell` field overrides

   probe must reproduce all of it     probe is Command::new(bin).args(...)
```

The probe exists because *"a settings entry proves something was written"* and nothing else. Its value is entirely in running the hook the way the agent runs it, so a probe that approximates the branch on the left can pass on an install the agent cannot use — the precise failure `setup-diagnostics` was created to remove. Reproducing that branch faithfully means implementing Git Bash discovery, the PowerShell fallback condition, and the `shell` override, in a checker, for a project whose stated rule is that a dependency or a mechanism must be smaller than the correctness risk of doing without it.

The right side is not a smaller version of the left side. It is the absence of the problem: `std::process::Command` already spawns an executable with an argument vector on every platform, which is what exec form is.

Confidence: high that this is correct, and it was raised from moderate by the upstream defect below rather than by the code being tidier.

## The upstream defect that makes this more than a preference

Claude Code detects the full Git Bash path at startup and then executes hooks with a bare `bash`. Git for Windows' default installation adds `Git\cmd` to `PATH` and not `Git\bin`, so `bash` does not resolve and every shell-form hook fails. Reported as `anthropics/claude-code#22700`, **closed as not planned and labelled stale**. A related report has Git Bash interpreting Windows backslash paths as escape sequences inside hook commands.

So the shell path on Windows is not a harder path we would be choosing to walk carefully. It is a path with a known hole in it and a sign saying nobody is coming to fill it. A `doctor` that faithfully reproduced Claude Code's shell selection on Windows would faithfully reproduce this bug — correctly reporting the user's hooks as broken, for a reason the user cannot fix except by editing their system `PATH`.

Exec form spawns no `bash`. Neither defect can reach it.

## The version floor, and why `doctor` has to check it

`args` was added in Claude Code 2.1.139. The relevant question is not what a supporting version does — it is what a non-supporting one does, and the answer is the worst available.

An older Claude Code does not error on an unknown key. It ignores `args` and treats the entry as shell form, so it runs `openspec-doc` with no subcommand. That is a clap usage error. `main.rs` exits 2 on a usage error unless the raw arguments look like `hook prompt` — and here they do not, because there are no arguments at all. Claude Code treats a `UserPromptSubmit` hook exiting 2 as a block.

The result is the failure mode this repository already documented and fixed once: *"Clap exits 2 on a usage error and Claude Code treats that as a block, so one typo refused every prompt in the session."* Registering exec form against an old agent reintroduces it, by a different route, for every user on an old agent.

Three ways to handle it, and the first two are worse than they look:

- **Write shell form for old agents and exec form for new ones.** Two registration formats, a version detection in `init`, and a `doctor` that has to check both. This is the backwards-compatibility path the repository's rules reject, and it keeps the shell branch alive — the entire thing this change exists to delete.
- **Make bare `openspec-doc` exit zero.** Changes the CLI's contract for every caller, to accommodate a misconfiguration produced by a version we do not support. The CLI would stop reporting a real usage error as one.
- **Declare a minimum and check it.** `doctor` already resolves binaries and reports their versions; it gains one more. An agent below the floor is a failing check with a named fix.

The third. It is honest, it is one check in a command built for exactly this, and 2.1.139 is far enough back — releases are frequent and Claude Code updates itself — that the floor is unlikely to bite anyone. Note the asymmetry worth stating plainly: `doctor` catches this *after* the user has already been blocked, because a blocked prompt is how they will notice. The check makes the cause discoverable, not the failure preventable. Preventing it would require `init` to refuse to write exec form against an old agent, which is the first option in disguise.

## What is lost: the environment-prefixed hook command

`setup-diagnostics` requires the probe to run *the recorded string*, so a developer who points a hook at a local build is probed on that build:

> **WHEN** a registered hook's command names a binary by absolute path **or is prefixed by an environment assignment** — **THEN** the system SHALL execute that string as it stands

`OPENSPEC_DOC_BIN=/tmp/b openspec-doc hook stop --agent claude` is shell syntax. `VAR=value command` is a shell feature and there is no shell any more. The absolute-path half survives — exec form's `command` can be a path — and the environment-assignment half cannot.

This is a real loss, and the honest accounting is that it is small: setting an environment variable for the agent's whole session, or naming the local binary by path, both do the job, and the second is what the wiring tests already exercise. The requirement and its scenario are edited in this change rather than left to fail, because a scenario that describes a capability the tool no longer has is worse than no scenario.

## The recognition predicate is the risky part

`COMMAND_MARKER` is the substring `openspec-doc hook `, and `init` uses it to decide whether a settings entry is one of ours — *"including one prefixed by an environment assignment or an absolute path"*. It is deliberately loose, and looseness is right here: the predicate's job is to find our entry so it can be **replaced**, and a predicate that is too strict produces a duplicate hook entry rather than a replacement. Two `Stop` hooks both starting a dashboard is a bad failure and a confusing one.

Under exec form the predicate becomes structural: an entry is ours when its `command` ends in the binary name and its `args` begin with `hook`. That must keep matching an entry whose `command` is an absolute path to a local build, and it must keep matching the *old* shell-form entries — not for compatibility, but because `init` run against an already-wired project has to replace them rather than add beside them. That is the one place this change touches the old format, and it touches it in order to delete it.

Test this from the direction of the failure: an `init` over an existing shell-form settings file must produce exactly three of our entries, not six.

## Windows joins the covered platforms in this change

`add-continuous-verification` records Windows as an omission with a reason. This change removes the reason, so it removes the omission. Doing that anywhere else would leave a window in which the repository claims Windows support it has not run a test against, which is what the capability's *"coverage and support move together"* scenario exists to prevent.

Order within the change matters: add the Windows leg **first**, with the three tests still gated, and look at what fails. Roughly 150 of the suite's 153 test functions compile on Windows today and nobody has ever run them. Predicting their failures from a Mac is guessing; the lane is the instrument. Fix `binary.rs`, do the exec-form work, then un-gate the three and see the leg go green with them included.
