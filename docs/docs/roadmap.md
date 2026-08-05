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

Four changes, all with complete artifacts and no implementation. The MVP column is **my reading, not a
decision** — the scope call is the owner's.

| Change | Tasks | In MVP? | Why it exists |
|---|---|---|---|
| `add-comment-thread-actions` | 20 | **Probably yes** | The dashboard can only *create* comments. Replying or closing one means leaving the browser, running `comment list` to recover a UUID, and pasting it into `comment resolve` — in a tool whose whole premise is browser-based review. Also adds the `addressed` status the loop needs for honest closure: today the agent either marks its own work resolved, or nothing records that it responded. `directive-verdict-loop` already refers to this status as a dependency. |
| `add-dashboard-lifecycle` | 28 | **Probably yes** | The loop has a precondition nobody remembers: `serve` must already be running. Forget it and nothing fails — the explore hook still readies the note, the agent still writes, the turn boundary still checks for a verdict, and there is simply no reviewer at the other end. A silent no-op is worse than an error. |
| `add-prompt-time-directive-delivery` | 13 | **No** | A directive only arrives when a turn *ends*, so the reviewer's own poke runs blind: you submit a verdict, prompt the agent, and it spends that whole turn unaware before the feedback lands at the end. Real annoyance, observed; the loop works regardless. |
| `add-change-approval-gate` | 28 | **No** | Nothing requires anyone to say a change is ready before implementation starts. That is a new concept rather than a missing piece of the review loop, and it would change what the tool *is*. |

## Known defects and debt

| Item | Severity | Notes |
|---|---|---|
| Three `watch.rs` filesystem-watcher tests fail | Low | Pre-existing; fail on a stashed tree too, so not caused by recent work. Timing or platform sensitivity in the test, not evidence about the watcher. |
| `cli-surface` spec omits the `scratch` subcommand | Low | Spec text still lists the MVP surface as `summary`, `serve`, `hook`, `comment`. Drifted when `scratch claim` landed. |
| Agents may formalize an exploration without claiming it | Moderate | Degrades to a visibly un-promoted note rather than a corrupted one — the trade claim-based promotion was made for — but has not been observed either way on a live session. The next real explore-to-proposal cycle is the check. |

## Suggested order

1. `add-comment-thread-actions` — closes the loop inside the browser, and unblocks the `addressed`
   status that `directive-verdict-loop` already references.
2. `add-dashboard-lifecycle` — removes the silent-no-op trap.
3. Declare the MVP done, or don't, against [the completion criterion](/vision.md).
4. `add-prompt-time-directive-delivery` and `add-change-approval-gate` as post-MVP work.
