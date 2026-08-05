# Vision & MVP scope

:::warning Draft — needs the owner's sign-off
This page was reconstructed from the archived change proposals, because the MVP had never been written
down anywhere. Every statement below is either evidenced by a shipped capability or marked as an
inference. Correct it rather than working around it: a scope nobody has stated is a scope nobody can be
finished with, and that is what this page exists to fix.
:::

## The problem

A coding agent doing exploratory work is opaque while it is being useful. Its reasoning lives in a
terminal transcript that scrolls past, cannot be annotated, and is gone once the session ends. By the
time there is an artifact worth reviewing — a proposal, a spec delta — the thinking that produced it has
already hardened, and the reviewer's leverage has mostly been spent.

Review tooling exists for the end of that process (pull requests) and nothing sits at the start of it.
The cost is not that review is impossible; it is that review arrives too late to change direction
cheaply.

## What openspec-doc is

A local review loop for [OpenSpec](https://github.com/Fission-AI/OpenSpec) projects:

> An agent's exploration is readable in a browser **while it is happening**. The repo owner selects
> text, leaves anchored comments, and submits a phase verdict. That verdict reaches the agent
> automatically at its next turn boundary, as a directive it acts on.

Everything is a plain file under `.openspec-doc/` at the project root. No database, no daemon, no cloud
service, nothing that outlives the machine it runs on.

## MVP scope

The MVP is the **closed loop**, and nothing more:

| Leg of the loop | Capability | State |
|---|---|---|
| An exploration becomes a readable artifact | `explore-note-capture`, `scratch-note-workflow` | shipped |
| The reviewer can read it and respond | `dashboard-server`, `dashboard-html-views`, `anchored-comments` | shipped |
| The response reaches the agent | `agent-hook-bridge`, `directive-verdict-loop` | shipped |
| The exploration becomes a formal change | `scratch-note-workflow` (claim-based promotion) | shipped |

Supporting: `project-scanner` (locating the project and its changes) and `cli-surface` (the same
operations from a terminal, which is also how the dashboard's behaviour is verified).

### Two agents, deliberately

Claude Code and pi.dev, both. This is scope, not a bonus. Building against one agent's hook API produces
a design shaped by that API's accidents; building against two forces the decision logic into a binary
both can call, which is why `core` holds every rule and the agent-specific parts are a thin adapter.

## What is deliberately excluded

- **No approval or gating.** The loop informs; it does not block work on a human decision. Tracked
  separately as `add-change-approval-gate`, and it is a genuinely new concept rather than a missing piece
  of this one.
- **No remote, multi-user, or hosted mode.** `127.0.0.1` only. Single reviewer, their own machine.
- **No git or provider integration.** No branches, PRs, or MRs.
- **No editing artifacts from the browser.** The dashboard reads and annotates; the agent writes.
- **No process supervision.** The dashboard is started by the owner. Whether it should start itself is
  tracked as `add-dashboard-lifecycle`.

## How the MVP is judged complete

One criterion, and it is not a test suite:

> The reviewer reads the exploration in the browser and leaves anchored comments where the thinking needs
> to change. When satisfied, they go to the terminal and tell the agent to move to proposal — and the
> agent produces a proposal **that accounts for those comments**.

The terminal is deliberately inside that loop. An earlier draft of this page aimed at "without touching a
terminal for the review itself", which is both very hard and the wrong target: the reviewer already lives
in a terminal, and asking them to drive the agent from a browser adds a control surface nobody needs. The
browser's job is *reading and annotating*, which is the part a terminal is bad at. Driving stays where it
already works.

What that criterion demands is narrower and sharper than the old one: **the comments have to reach the
agent before it writes the proposal, not after.**

### What is proven

The transport, and it was the hard part. Both verdict kinds were driven end to end on real Claude Code
sessions, one on pi.dev, and claim-based promotion under two concurrent sessions. The risk everything was
designed around — that an agent would refuse an injected directive as prompt injection, which had been
observed once — did not recur.

### What is not

Two gaps sit directly on the criterion above, and both were found by reading the code against it rather
than by any test:

**The move-to-proposal directive never points at the comments.** It names the exploration note and the
verdict sidecar, and mentions comments only as something the claim will relocate afterwards. The
keep-exploring template names the comment sidecar; this one does not. So an agent formalizing an
exploration is not told the reviewer's anchored comments exist.

**A directive arrives when a turn ends, not when it starts.** So telling the agent "let's move to
proposal" in the terminal produces a turn that writes the proposal, and only then receives the directive
about the comments. The proposal is written blind and has to be revised. Fixing this is
`add-prompt-time-directive-delivery`, which this criterion moves from a nice-to-have onto the critical
path.

Two further things need a terminal or insider knowledge, and under this framing they are **no longer
MVP-blocking** — they are quality of life:

- Closing out a comment means leaving the browser for the CLI (`add-comment-thread-actions`), though the
  `addressed` status it adds is how an agent records that it responded to a comment while proposing.
- If `serve` is not already running, the whole loop silently does nothing (`add-dashboard-lifecycle`).

See [Roadmap](/roadmap.md) for how the open changes triage against this.

## Principles that constrain the design

Each of these was arrived at by getting it wrong first, and each is enforced somewhere in the specs.

- **Plain files, hand-editable.** Append-only streams replayed in order; a later record never rewrites
  an earlier one. State you can read with `cat` is state you can debug.
- **Pointer, not embed.** Injected directives name files to read instead of quoting the reviewer. This
  is what makes them corroborable, and corroborable is what stops them being refused. See
  [Pointer, not embed](/concepts/pointer-not-embed.md).
- **Never guess at attribution.** Promotion acts on an explicit claim. The heuristic it replaced credited
  any new change directory to any session, which silently moved the wrong exploration onto the wrong
  change.
- **Fail loudly, except in front of the human.** Errors propagate with their source chain. The one
  deliberate exception is any hook standing between the reviewer and their own prompt, where refusing
  input is worse than the problem being reported.
- **The reviewer's judgement is not automated.** An agent reports what it did; it never marks its own
  work accepted. Resolving a comment is the reviewer's act.
