# openspec-doc Rust migration — ideation phase

Captured from an `/opsx:explore` session. This is a thinking-capture document, not a spec — the authoritative artifacts are the 8 OpenSpec changes listed at the bottom.

## Starting point

`openspec-doc-rs-example/` (TypeScript) already has a mature scanner → parser → analyzer → viewer → server pipeline, a review-companion browser mode with anchored HTML comments persisted to append-only JSONL sidecars, and a separate advisory Pi session for live analysis on review pages.

What it does **not** have: autonomy. `src/server/lifecycle-actions.ts` only prints handoff text (e.g. `"Ask Pi: continue proposal work for X"`) for a human to copy-paste into the agent. The dashboard never talks back into a live agent turn.

Goals for the rewrite:
- Rust instead of TypeScript.
- Real autonomy during a coding session: detect OpenSpec usage, surface a live dashboard, and let dashboard actions actually drive the agent forward — not just print instructions for a human to relay.
- Keep the in-browser anchored commenting from the TS example, extended to work during pure exploration (before any formal change exists), not just on post-proposal review pages.
- CLI first, prove the mechanism, plain HTML dashboard — Vue and full TS parity come later, once the mechanism is proven.

## The core mechanism

Claude Code's Stop hook fires when the agent's turn ends, receiving `session_id`, `transcript_path`, `cwd` on stdin. It can return `{"decision":"block","reason":"..."}` to veto the stop and feed `reason` back in as the next thing the agent acts on. pi.dev has an equivalent mechanism (confirmed by the user; exact schema not yet verified against docs).

This is the actual lever that makes "autonomous hooks" real: instead of a human copy-pasting a suggested prompt, the CLI itself can re-drive the agent from outside the conversation, triggered purely by turn boundaries.

## Key decisions made during exploration

1. **Dashboard UX**: one server, live-updating via SSE, separate tabs per OpenSpec phase — not a browser window popped open on every single turn.
2. **Session scoping**: no support for multiple concurrent agents/sessions against the same OpenSpec change. One change, one agent, at a time is an accepted constraint — simplifies directive/comment storage to a single key per scope, no locking.
3. **Comment → agent handoff is simple, not a digest**: the CLI never pre-summarizes review comments for the agent. The injected Stop-hook reason is a short **pointer + verdict** ("read the comments at path X, address them, mark ready for review"), and the agent reads and interprets the actual content itself via normal file tools. This avoids the CLI ever needing to serialize arbitrary review text into hook JSON, and avoids staleness if more comments land between a dashboard click and the next Stop event.
4. **Anchor target during explore phase**: before a formal `openspec/changes/<name>/` directory exists, there's nothing for the anchor engine to bind comments to. Resolved by having the agent maintain a **scratch note** (`.openspec-doc/scratch/_session/<session_id>.md`, outside `openspec/`) that the dashboard anchors comments against — reusing the exact same anchor-resolution machinery as post-proposal artifacts, rather than inventing transcript parsing.
5. **Two scoping regimes, not one**:
   - Pre-proposal: keyed by `session_id` (scratch note, comments, phase-verdict).
   - Post-proposal: keyed by change name (real artifacts, comments, phase-verdict).
6. **Promotion mechanic**: when a change directory appears for a session with a pending scratch note, the CLI **renames** the scratch note (and, per the comment engine's own responsibility, its comment sidecar) from the session-scoped key to the change-name-scoped key — one key per change forever, no permanent dual-namespace. Detection is via **snapshot-diff** of the active-change list between two `hook stop` invocations for the same session, not an explicit "agent writes back its chosen name" convention (rejected as an easily-forgotten, invisible contract). If more than one new active change appears between checks, auto-promotion is skipped and an ambiguity notice is recorded instead of guessing — an accepted edge case given decision #2.
7. **Promotion requires a hook round-trip**: promotion isn't just a rename — it also runs `openspec validate <name>` immediately after, and surfaces failure rather than discarding it. This check happens *at Stop-hook time*, not via dashboard polling, so validity is known the moment a proposal is formalized.
8. **Phase-verdict carries real content, but the hook payload doesn't embed it**: "keep exploring" verdicts do feed open questions/resolutions/free-text input back to the agent — but as content living in files (scratch note, comments sidecar) that the agent reads directly, never as text embedded in the injected Stop-hook reason itself. The reason is always "read file X, here's the verdict," never the literal review text.
9. **Rust scope for the MVP**: prove the hook mechanism end-to-end (Claude Code + pi.dev) with a minimal scanner and plain HTML — not full parity with the TS example. Full parity (and Vue) is an explicit later goal, not part of this slice.

## Open questions carried into the change proposals (not resolved here)

- pi.dev's exact Stop-hook payload/response schema — needs to be captured from real output before implementing its adapter (`add-agent-hook-bridge`).
- Fail-fast vs. fail-safe policy for *internal* hook errors (after stdin parses successfully): exit non-zero and let the failure surface however the host agent handles a broken hook, or catch-and-log-and-allow so a bug in `openspec-doc` never bricks a live coding session. This directly tensions with the project's stated "fail loudly, no silent fallback" rule and was deliberately left as an explicit decision point in `add-agent-hook-bridge`'s design.md rather than defaulted.

## Resulting MVP change sequence

Eight OpenSpec changes were created and validated under `openspec/changes/`, in dependency order:

```
1. add-rust-foundation        (workspace, root discovery, change/spec enumeration)
2. add-cli-surface            ──depends on 1
3. add-agent-hook-bridge      ──depends on 1   (the linchpin — proven against fixtures)
4. add-dashboard-server       ──depends on 2
5. add-scratch-note-workflow  ──depends on 1
6. add-anchored-comments      ──depends on 1
7. add-dashboard-html-views   ──depends on 4, 5, 6
8. add-directive-verdict-loop ──depends on 3, 7   (capstone — end-to-end proof required)
```

`add-agent-hook-bridge` (#3) and `add-directive-verdict-loop` (#8) are deliberately split: the hook mechanism is proven standalone against a hand-written directive first, de-risking the part that was most uncertain (pi.dev's mechanism), before any dashboard UI investment. #8 is the actual proof that the whole premise holds — it requires a real end-to-end run against both Claude Code and pi.dev before the MVP is considered done.

Next step: `/opsx:apply add-rust-foundation`.
