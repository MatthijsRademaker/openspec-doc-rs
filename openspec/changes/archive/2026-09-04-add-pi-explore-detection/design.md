## Context

The hook bridge has three delivery points. Claude Code reaches all three through `.claude/settings.json`: `UserPromptSubmit` → `hook prompt`, `Stop` → `hook stop`, `UserPromptExpansion` → `hook explore`. pi has no external-process hook, so its half is an in-process TypeScript extension, and that extension implements two of the three.

The Rust side is already indifferent to which harness is calling. `hook explore --agent pi` is an accepted invocation, `parse_session_id` understands pi's payload shape, and the extension already sends `{"sessionId": …}` on stdin for both existing handlers. `start_explore` readies the note's directory, prints the resolved path and the instruction, and — through `discovery::preferred` — the session's review URL. None of it is Claude-specific.

So this change adds a third handler to one file. What needs deciding is which pi event it hangs off, and how it recognizes an exploration.

Claude Code's `UserPromptExpansion` matcher fires on `command_name` — the bare command, no leading slash, no namespace. pi has no equivalent event. Its options are the events the extension can already see and one it cannot yet.

## Goals / Non-Goals

**Goals:**

- A pi exploration produces the same scratch note, at the same path, with the same emitted instruction, as a Claude Code exploration.
- A pi session that explores becomes visible in the dashboard index and reviewable on its own page.
- The repository stops carrying prose about the gap once the gap is closed.

**Non-Goals:**

- The Claude Code `Skill`-tool gap. An exploration the agent starts itself fires no `UserPromptExpansion` and creates no note; that stays documented and accepted.
- Teaching `doctor` to probe pi's wiring. pi's delivery points are in-process with no command string to execute, which is why `doctor` reports them `not checked`; that remains true of the third handler and is a separate problem.
- Any change to `hook explore` itself, or to the note, claim or promotion mechanics.

## Decisions

### Match raw input, on `pi.on("input")`, not the expanded prompt

`pi.on("input")` fires before skill and template expansion and therefore sees the text the user typed.

The alternative is `before_agent_start`, which is cheaper on paper: the extension already handles that event and currently discards its payload unread, and that payload carries the submitted `prompt`. It is rejected because `before_agent_start` sees the prompt *after* expansion. A matcher there would be matching the expanded prose of `.pi/prompts/opsx-explore.md` — a file this repository ships but pi's template mechanism owns the rendering of — so an upstream rewrite of that prose silently stops explore detection, and does it in the exact fail-silent way this whole capability's spec calls out as indistinguishable from the original defect. Matching a command name is matching something this repository controls.

`agent_start` and `agent_settled` carry no payload at all, and `agent_end` fires after the turn, far too late to ready a note the agent was supposed to write during it.

### Do not use `pi.registerCommand`

Registering `opsx-explore` as an extension command would give clean dispatch, and it is wrong. pi checks extension commands *before* the input event, so a registered `opsx-explore` shadows `.pi/prompts/opsx-explore.md` — the prompt template that carries the actual explore instructions — and the exploration silently stops happening while the note starts being created. That is worse than the gap being fixed.

### The matcher covers both entry points

An exploration starts on pi two ways: `/opsx-explore`, the prompt template, and `/skill:openspec-explore`, the skill. Both are this repository's own names, both appear verbatim in raw input, and a matcher that covers one leaves half the gap open.

Matching is on the command token at the start of the input, not a substring search of the whole message. The Claude Code side's spec is explicit that prompt-text scanning is the wrong mechanism because it fires on any message that merely mentions the command — a question *about* `/opsx-explore` must not start an exploration. pi gives no command-name event, so raw input is what there is; restricting the match to the leading token is how that reasoning is honoured with the events available.

### Failures notify and continue, as the existing handlers do

`hook explore` is best-effort in exactly the sense `hook prompt` is: it sits in front of the human's own input, and refusing to run their prompt because a note could not be readied is worse than the missing note. A spawn failure or a non-zero exit surfaces through `ctx.ui.notify` and the input proceeds unchanged.

Where it differs from the other two handlers is stdout. `hook explore`'s output is an instruction for the model, delivered on Claude Code through that event's context contract. On pi it is injected the same way `before_agent_start` injects a directive — as an `openspec-doc-directive` message with `display: true` — so the agent is told the resolved path rather than left to construct one, which the capability's spec requires.

### Delete the analysis from `AGENTS.md` rather than update it

The three paragraphs in `AGENTS.md` are the reasoning above, written in the file the agent reads as standing instruction. Once the handler exists they describe nothing. Their durable content is these decisions, which live here and get archived with the change — which is where a design argument belongs.

## Risks / Trade-offs

- **`pi.on("input")` is asserted from pi 0.84.3's type declarations, which are not installed in this repository** → Verify the event name, payload field and return contract against the pi version actually installed before writing the handler. If `input` does not exist or cannot see raw text on that version, stop and re-decide rather than falling back to `before_agent_start`; the fail-silent objection above does not weaken because the preferred route was unavailable.
- **Raw-input matching is weaker than Claude Code's `command_name` dispatch** → Accepted; pi offers nothing stronger. Anchoring to the leading token keeps the failure mode to inputs that genuinely begin with the command.
- **A matcher that never fires looks exactly like the gap it fixes** → The verification tasks require a live pi session, not a unit test of the matcher. `explore-note-capture` already states that a non-firing matcher is a failure of the change rather than a silent no-op, and this change inherits that bar on a second harness.
- **The extension exists in two places, `crates/cli/assets/` and `.pi/extensions/`** → The asset is the one copy; the project's file is its output. Regenerate with `openspec-doc init --yes` and let `init` report it unchanged, rather than editing both by hand.
