## Context

Setting a project up means putting three unrelated artifacts in three places: JSON hook entries in a settings file Claude Code owns, a TypeScript extension file pi discovers by path, and prose in an instruction file the repository owner owns. They have nothing in common except that all three are currently copied by hand out of documentation, and that any one of them being absent produces silence rather than an error.

`openspec init --tools` already exists upstream, already covers `claude` and `pi`, and already writes into `.claude/`. This command sits beside it in the same directory, which constrains the design more than anything else does: it must not ask a question upstream has already answered, and it must not fight upstream for ownership of files upstream installs.

## Goals / Non-Goals

**Goals:**
- One command takes a project from nothing to wired, for the harnesses it actually uses.
- Re-running it is safe and produces no diff on an already-correct project.
- Every file it touches belongs to someone else, and everything it did not write survives untouched.
- A problem it cannot fix is reported with the command that fixes it.

**Non-Goals:**
- Not a replacement for `openspec init`. It configures this tool's hooks in a project OpenSpec already initialized.
- Not an installer for the `opsx` commands. Those are upstream's.
- Not a verifier. Proving the configuration *runs* is `doctor`'s job and needs subprocesses.
- Not the fix for pi's missing explore hook.
- No interactive form in this change. Ratatui is a wish-list item, and if it lands it fills the same struct the flags do.

## Decisions

- **Dry run by default; `--yes` performs.** The alternative — write immediately, or prompt when attached to a TTY — either surprises the operator or branches on TTY detection, which puts the interactive path outside the reach of the integration tests. Defaulting to a plan means one code path, one output, exercised identically by a person and by CI. The cost is a second invocation for the common case, which is the correct price for a command that appends to `AGENTS.md`.

- **Detection proposes, `--agent` disposes, and detection is evidence-based rather than inferential.** A harness counts as present when its directory exists: `.claude/` for Claude Code, `.pi/` for pi.dev. This deliberately does not try to read what `openspec init` recorded — its marker format is upstream's and can change without notice, and a detector coupled to it fails by silently configuring nothing. Directory presence is coarse, occasionally wrong, and wrong in a visible direction: it over-detects, the plan says which harnesses it found, and the operator reads that line before anything is written.

- **A hook entry is ours when its command contains `openspec-doc hook `.** Ours are replaced wholesale; everything else in the file is preserved. Substring rather than prefix, so an entry a developer has pointed at a local build — `OPENSPEC_DOC_BIN=… openspec-doc hook stop --agent claude` — is recognised as ours and updated rather than duplicated beside a new one. The upgrade case falls out of this for free: an entry written by an older version still matches and is replaced. Alternative considered: a marker key inside the JSON object, rejected because Claude Code owns that schema and unknown keys are its business, not ours.

- **`init` writes the `UserPromptExpansion` matcher even when no command matches it, and says so loudly.** The alternative is to omit the hook and produce a project that is quietly missing a third of its wiring, which is harder to diagnose than one that is complete and has a warning attached. The warning names the command that fixes it. This is the one place `init` knowingly writes something inert, and it is the least bad of the three options — the third, refusing to configure Claude at all, punishes the operator for upstream's file layout.

- **The `AGENTS.md` block is delimited by HTML comment markers and rewritten in place.** `<!-- openspec-doc:begin -->` … `<!-- openspec-doc:end -->`. Everything outside is byte-preserved. Absent both markers, the block is appended; present, the span between them is replaced. A file containing one marker and not the other is a hard error rather than a guess — a half-marked file means something already went wrong, and repairing it by appending would produce two overlapping blocks in the file that tells the agent how to behave.

- **The instructions are a block in `AGENTS.md`, not a skill.** Skills are pulled: loaded when the model judges their description relevant. This content has to be pushed. The agent needs to know that the scratch note is the only artifact the reviewer can read *before* it decides where to put its thinking; an agent that has already written its exploration into the conversation has no reason to consult a skill telling it not to. Ambient facts go where ambient facts go.

- **The pi extension is embedded and generated, including into this repository.** There is exactly one copy of that file in the world, in the binary. This repository's checked-in copy becomes its output, so the write path is exercised on a real project every time anyone runs `init` here, and a divergence shows up as a diff rather than as a bug in someone else's project. The fifty-line verification header in the current file is development history rather than user documentation; it moves to `docs/` so the shipped file is the same file.

- **This repository also takes the `AGENTS.md` block.** The reviewer ruled that this file is this repository's own and that overlap with the generic block is acceptable if the block is its own entry. That ruling is compatible with installing it, and installing it is what keeps the marker-editing code from being a path only fixtures ever traverse. The hand-written `openspec-doc review directives` section stays exactly as it is; it says more, and more repository-specific things, than the generic block does.

- **No `--settings local`.** Committed `.claude/settings.json` is the only target. The hook command strings contain nothing machine-specific, so the per-machine argument that put them in `settings.local.json` does not hold, and the only remaining use for the flag is to reproduce the exact state `doctor` exists to flag.

- **`doctor` is re-scoped inside this change, as its last task.** Not a follow-up: this change falsifies that proposal's "Why" and its design's claim about un-committed wiring, and the repository's rule is that the text a change invalidates moves with it. The division that survives is `init` writes the configuration, `doctor` runs it — and the definition of what a wired project contains lives in one module both read.

## Risks / Trade-offs

- [Risk] **Round-tripping `.claude/settings.json` through `serde_json` reformats the whole file.** A hand-formatted settings file comes back normalized, and the diff is noisier than the change. → Accepted deliberately. Claude Code's settings are strict JSON, so nothing is lost, only rearranged; a format-preserving editor is a large dependency for a cosmetic gain.
- [Risk] **Directory-presence detection over-detects.** A `.claude/` directory left by a colleague configures Claude Code for someone who does not use it. → Mitigated by the dry-run default: over-detection is visible in the plan, before anything is written, and `--agent` overrides it.
- [Risk] **`init` cannot prove anything it writes works.** It can write a perfect settings file for a binary that is not on the agent's `PATH`. → Accepted; that is exactly the boundary with `doctor`, and it is why re-scoping `doctor` to its execution checks rather than deleting it is the right outcome.
- [Risk] **Generating this repository's own pi extension makes a tracked file a build output.** Someone edits the tracked copy, `init` overwrites it, and the edit is lost. → Mitigated by there being one source: the embedded asset is the file to edit, and the checked-in copy carries no independent content. The failure is loud — the next `init` produces a diff — rather than silent.
- [Trade-off] **Writing an inert `UserPromptExpansion` matcher.** A configuration that is complete-and-warned is preferred over one that is silently partial, on the grounds that the second is the failure mode this entire project exists to eliminate.
- [Trade-off] **Configuring pi ships a known gap.** `init --agent pi` gives pi the directive loop and the dashboard and not the explore phase. → Reported in the plan output rather than hidden, and not fixed here: closing it is a pi feature with its own design, and folding it in would trade a clean scope for a small convenience.
