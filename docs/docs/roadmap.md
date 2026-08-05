# Roadmap

## Shipped

Nine capabilities, each with a spec under `openspec/specs/` and an archived change behind it.

| Capability | What it gives you |
|---|---|
| `project-scanner` | Locates the project root and enumerates changes and specs by directory structure alone |
| `cli-surface` | The `openspec-doc` binary and its subcommand tree |
| `dashboard-server` | The axum server, filesystem watcher, and SSE update channel |
| `dashboard-html-views` | The session and change review pages, select-to-comment, verdict controls |
| `anchored-comments` | Comments attached to spans of markdown, with honest drift reporting |
| `scratch-note-workflow` | The exploration note's lifecycle, and claim-based promotion onto a change |
| `agent-hook-bridge` | Turn-boundary hook for Claude Code and pi.dev; block and reinject |
| `directive-verdict-loop` | Verdict to directive translation, consumed exactly once |
| `explore-note-capture` | Detecting the explore command and telling the agent where to write |

The loop these compose is closed and verified on live sessions across both agents. See
[Vision & MVP scope](/vision.md) for what that does and does not establish.

## Open

Triaged against [the completion criterion](/vision.md#how-the-mvp-is-judged-complete): the reviewer
comments in the browser, says "move to proposal" in the terminal, and the agent produces a proposal that
accounts for those comments.

That criterion is what decides this table, and it reorders it. What matters is whether the comments reach
the agent **before** it writes the proposal; whether the reviewer used a terminal does not.

| Change | Tasks | In MVP? | Why |
|---|---|---|---|
| *(new)* point the move-to-proposal directive at the comments | ~3 | **Yes — blocking** | The template names the note and the verdict sidecar but never the comment sidecar, so an agent formalizing an exploration is not told the reviewer's comments exist. Directly defeats the criterion. Smallest fix on this list: one template, one test. No change written yet. |
| `add-prompt-time-directive-delivery` | 13 | **Yes — blocking** | A directive arrives when a turn *ends*. So "let's move to proposal" produces a turn that writes the proposal and only then hears about the comments. Delivering at `UserPromptSubmit` puts them in context at the start of that turn, which is exactly the interaction the criterion describes. Previously triaged as a nice-to-have; the reframing moved it onto the critical path. |
| `add-comment-thread-actions` | 20 | **No** — quality of life | Replying or closing a comment means leaving the browser for the CLI. Under the old "no terminal" framing that was MVP-blocking; it no longer is. The `addressed` status it adds is still how an agent records that it responded to a comment rather than marking its own work resolved, and `directive-verdict-loop` already refers to it as a dependency. |
| `add-dashboard-lifecycle` | 28 | **No** — quality of life | `serve` must already be running or the loop silently does nothing: hooks fire, the agent writes, and there is no reviewer at the other end. A silent no-op is worse than an error, but it is a trap for the operator rather than a break in the loop. |
| `add-change-approval-gate` | 28 | **No** — post-MVP | Nothing requires anyone to say a change is ready before implementation starts. A genuinely new concept rather than a missing piece of the review loop, and it would change what the tool *is*. |

## Known defects and debt

| Item | Severity | Notes |
|---|---|---|
| Three `watch.rs` filesystem-watcher tests fail | Low | Pre-existing; fail on a stashed tree too, so not caused by recent work. Timing or platform sensitivity in the test, not evidence about the watcher. |
| `cli-surface` spec omits the `scratch` subcommand | Low | Spec text still lists the MVP surface as `summary`, `serve`, `hook`, `comment`. Drifted when `scratch claim` landed. |
| Agents may formalize an exploration without claiming it | Moderate | Degrades to a visibly un-promoted note rather than a corrupted one — the trade claim-based promotion was made for — but has not been observed either way on a live session. The next real explore-to-proposal cycle is the check. |

## Suggested order

1. **Point the move-to-proposal directive at the comment sidecar.** Hours, not days, and nothing else on
   this list matters while an agent formalizing an exploration is unaware the comments exist.
2. **`add-prompt-time-directive-delivery`** — so the comments arrive before the proposal is written rather
   than after.
3. **Run the criterion end to end on a live session**: comment in the browser, say "move to proposal" in
   the terminal, and check the proposal actually accounts for the comments. That check is the MVP call.
   Nothing below the live run can make it — the same reason the injection risk needed a real session.
4. Then declare the MVP done, or don't.
5. `add-comment-thread-actions`, `add-dashboard-lifecycle`, `add-change-approval-gate` afterwards.

Both blocking items are small. The MVP is closer than the 89 open tasks suggest, because most of those
tasks are not on the criterion's path.
