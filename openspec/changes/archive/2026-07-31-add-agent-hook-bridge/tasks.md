## 1. Hook event model

- [x] 1.1 Define agent-agnostic `HookEvent` (agent kind, session id, transcript path, cwd) and `HookDecision` (allow | block-with-reason) types
- [x] 1.2 Implement the Claude Code adapter: parse Stop-hook stdin JSON into `HookEvent`, serialize `HookDecision` into Claude Code's expected stdout JSON
- [x] 1.3 Implement the pi.dev adapter and pi's half of the bridge. **This task's original premise was void:** it asked to "capture a real pi.dev Stop-hook payload/response fixture" first, but pi.dev has no external-process hook to capture — its turn boundary is only reachable from a TypeScript extension (`pi.on("agent_end")`), and continuation is `pi.sendUserMessage(..., { deliverAs: "followUp" })`. So the pi payload/response schema is one we define, and pi's half of the bridge is a thin extension at `.pi/extensions/openspec-doc-hook.ts`. A real payload from that extension was captured and is the fixture. See design.md.
- [x] 1.4 Add fixture-based tests for both adapters using representative real payloads

## 2. Directive file

- [x] 2.1 Define the session-scoped pending-directive JSON file format (`pending`, `reason`, timestamps)
- [x] 2.2 Implement directive lookup by session id
- [x] 2.3 Implement consume-on-read (set `pending:false`, preserve file as audit trail)
- [x] 2.4 Add tests: pending directive is consumed exactly once; absent directive yields allow

## 3. `hook stop` subcommand

- [x] 3.1 Implement `openspec-doc hook stop --agent <claude|pi>` reading stdin, resolving the directive, emitting the decision
- [x] 3.2 Resolve the fail-fast-vs-fail-safe open question from design.md and implement the chosen internal-error handling policy — resolved as fail-fast with no catch-and-downgrade layer; see design.md Decisions for why that is safe in both agents
- [x] 3.3 Add an end-to-end test per agent: fixture stdin in, fixture directive on disk, expected stdout decision out

## 4. Manual verification

- [x] 4.1 Wire `openspec-doc hook stop --agent claude` as a real Claude Code Stop hook against a hand-written pending directive, and confirm the session is genuinely kept alive with the injected reason visible. **Deviation:** wired in a throwaway project's `.claude/settings.json` rather than this repo's, so a broken hook could not wedge live work in this repo. This repo's `.claude/settings.json` is deliberately left unarmed — arming it belongs to whoever wants the loop running here, not to this change. Evidence recorded in design.md.
- [x] 4.2 Repeat for pi.dev. Proven in a real interactive pi session in this repo: the agent wrote a pending directive for its own session via `$PI_SESSION_ID`, ended its turn, and pi started a fresh turn on its own with the reason text delivered as a user message, which the agent then acted on. No stale-ctx error, no hang. The two pi quirks found earlier (session-replacement stale ctx; `agent_end` + `--session-id` hang) turned out to be **harness-only** and are recorded in design.md and in `.pi/extensions/openspec-doc-hook.ts` for whoever next tries to automate this. Reproduce with `OPENSPEC_DOC_BIN=$PWD/target/debug/openspec-doc pi` — no `--session-id`, no `--continue`.
