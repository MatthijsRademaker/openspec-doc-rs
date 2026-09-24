# LLM quickstart

Let the agent install and wire openspec-doc for you. Open a **Claude Code** or **pi** session at the root
of the project you want reviewed, paste the prompt below, and answer the one or two questions it asks.

It does what [Quickstart](/quickstart.md) sections 1 and 2 describe: runs the release installer, wires the
hooks for the harness it is running in, and proves the wiring with `doctor`. It stops and tells you instead
of guessing when something is missing — it does not install Node tooling or edit your shell startup files
without asking.

It needs macOS or Linux. On Windows it stops and points at the source build, because there are no Windows
binaries yet.

## The prompt

````text
Install and set up openspec-doc in this project so its review dashboard is ready to use.
openspec-doc is a local review dashboard for OpenSpec projects plus hooks that deliver the
reviewer's feedback back into this agent session. Docs: https://matthijsrademaker.github.io/openspec-doc-rs/

Work through these steps in order. Run every command yourself and read its output. If a step
fails, stop and show me the exact output rather than working around it.

1. Platform. Run `uname -sm`. Published binaries exist for macOS (arm64, x86_64) and Linux
   (x86_64, arm64). On anything else, stop and tell me to build from source per
   https://github.com/matthijsrademaker/openspec-doc-rs#install.

2. Install. Run:
     curl -fsSL https://github.com/matthijsrademaker/openspec-doc-rs/releases/latest/download/install.sh | sh
   It installs to ~/.local/bin. If it reports that directory is not on PATH, prefix the
   remaining commands with `PATH="$HOME/.local/bin:$PATH"`, and ask me before adding the line
   it prints to my shell startup file. Either way, remember it for step 7: the hooks run with
   the PATH this agent was started with.

3. The openspec CLI. If the installer said `openspec` is not on PATH, it is required:
   promotion runs `openspec validate`. Ask me whether to install it with
   `npm install -g @fission-ai/openspec@latest`. Do not install it without my answer.

4. OpenSpec project. openspec-doc finds the project by `openspec/config.yaml`. If that file
   is missing, ask me before running `openspec init --tools claude` (in Claude Code) or
   `openspec init --tools pi` (in pi). Without a terminal it skips the config file, so if
   `openspec/config.yaml` still does not exist afterwards, create it containing the single
   line `schema: spec-driven`, OpenSpec's default workflow.

5. Wire the hooks. Use `--agent claude` if you are Claude Code, `--agent pi` if you are pi.
   First run `openspec-doc init --agent <agent>` — a dry run that writes nothing — and show me
   the plan in a few lines. Then run `openspec-doc init --agent <agent> --yes`. If it warns
   that no explore command exists, run the `openspec init --tools <agent>` it names, then
   re-run `openspec-doc init --agent <agent> --yes`.

6. Verify. Run `openspec-doc doctor`. It executes each registered Claude Code hook the way
   the agent would and exits non-zero if any fails.
   - In Claude Code: if it fails, show me its output and stop.
   - In pi: doctor does not examine pi's extension, and it fails the three Claude Code hook
     checks because this project has no Claude Code hooks. Those three failures are
     expected. Confirm instead that its `binary on PATH` line passes and that
     `.pi/extensions/openspec-doc-hook.ts` exists; stop on anything else.

7. Report. Run `openspec-doc serve url` and tell me, briefly:
   - the dashboard URL it printed (the dashboard starts itself at the first turn boundary
     that has something to review; I do not need to run a server);
   - that I must restart this session before the hooks take effect — and, if
     ~/.local/bin was not on PATH in step 2, restart it from a shell where it is;
   - how to start: `/opsx:explore <topic>` in Claude Code, `/opsx-explore <topic>` in pi;
   - anything from steps 2–6 that is not fully working.
````

## What "ready" means afterwards

- `openspec-doc` is on `PATH`. In Claude Code, `openspec-doc doctor` passes. In pi it cannot: `doctor`
  only probes Claude Code hooks, so a pi-only project fails those checks and its extension is reported
  as not checked. The first real proof in pi is a directive arriving after your first verdict.
- The project has the hook entries (`.claude/settings.json`) or the extension
  (`.pi/extensions/openspec-doc-hook.ts`), plus a block of standing instructions in `AGENTS.md`. Review them
  with `git diff` and commit them — the command strings hold nothing machine-specific.
- After restarting the session, `/opsx:explore` (Claude Code) or `/opsx-explore` (pi) starts an exploration
  the dashboard can show. From there, follow [Quickstart](/quickstart.md) from section 4.
