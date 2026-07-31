## Context

Claude Code supports a Stop hook that receives a JSON payload on stdin (including `session_id`, `transcript_path`, `cwd`) and can return `{"decision":"block","reason":"..."}` to prevent the session from stopping, feeding `reason` back in as the next thing the agent acts on. This change is the first place either mechanism is exercised in this codebase.

pi.dev's mechanism turned out to be shaped differently, verified against the shipped `@earendil-works/pi-coding-agent` docs and examples: pi has **no external-process hook at all**. Its turn boundary is only reachable from a TypeScript extension, via `pi.on("agent_end", ...)`, and continuation is triggered by `pi.sendUserMessage(text, { deliverAs: "followUp" })`. `examples/extensions/git-merge-and-resolve.ts` is that exact pattern — `agent_end`, `pi.exec` out to a binary, `sendUserMessage` to re-inject. `ctx.sessionManager` exposes `getSessionId()` and `getSessionFile()`, and `ctx.cwd` the working directory, so the extension can supply every field the normalized event needs. The pi side of this bridge is therefore a thin TS extension that shells out to `openspec-doc hook stop --agent pi`, and the pi payload/response schema is one **we** define rather than one pi imposes.

## Goals / Non-Goals

**Goals:**
- Prove the block-and-reinject mechanism works end to end for Claude Code, against a hand-authored directive.
- Prove (or determine what's missing to prove) the equivalent for pi.dev.
- Establish an agent-agnostic internal representation so later changes (which write real directives from the dashboard) don't need to know per-agent hook schemas.

**Non-Goals:**
- No dashboard, comment engine, or scratch-note logic — directives are hand-written fixtures in this change.
- No change-name-scoped directives or the session→change-name promotion mechanic (later changes).

## Decisions

- **Internal agent-agnostic `HookEvent`/`HookDecision` types**, with a per-agent adapter module translating stdin/stdout shapes into/out of them. Alternative considered: duplicate the directive-lookup/consumption logic per agent — rejected, that logic is the actually risky/shared part and duplicating it doubles the surface for bugs.
- **Directives are scoped by session id only in this change** — a JSON file at a fixed sidecar location keyed by `session_id`. Change-name-scoped directives and the rename-on-promotion mechanic are deliberately deferred to `add-scratch-note-workflow` and `add-directive-verdict-loop`; this change proves the mechanism with the narrowest possible key.
- **Directive files are consumed by read-then-overwrite** (set `pending: false`), not deleted — preserves an audit trail of what was injected and when, useful for debugging why an agent got redirected.
- **Claude Code is the reference adapter, implemented and verified first**; the pi.dev adapter follows the same normalized event/decision types.
- **The pi wire format is deliberately not identical to Claude Code's** — pi's payload is camelCase (`sessionId`, `transcriptPath`, `cwd`) and its decision is `{"action":"continue","message":"..."}` / `{"action":"stop"}`, versus Claude Code's snake_case payload and `{"decision":"block","reason":"..."}` / `{"continue":true}`. Since we own the pi schema we could have reused Claude Code's verbatim and made the pi adapter zero code; rejected because then pointing `--agent pi` at a Claude payload (or the reverse) half-works silently instead of failing loudly. Distinct shapes make a misconfigured hook a parse error.
- **`--agent` dispatch is a `match` on an enum, not a trait.** Two adapters with one call site do not justify a trait object.
- **Fail-fast on internal errors, with no catch-and-downgrade layer** (resolves the open question below). Any error after stdin is read propagates: the process exits non-zero having emitted no decision. This is safe *because of* how both agents treat a broken hook — Claude Code reserves exit code 2 for "block" and treats other non-zero exits as a non-blocking error (stderr surfaced to the user, turn ends normally), and the pi extension injects nothing unless it gets `action: "continue"`. So a fail-safe "catch the error and emit allow" layer would buy nothing over propagating, while costing a divergence from the project's fail-loudly rule and an extra code path. The only real constraint this imposes is that the binary must never exit with code 2 on an error path.
- **A session id is validated before it reaches a path.** It arrives from the agent and is interpolated into the directive filename, so an empty id, `.`, `..`, or one containing a path separator is rejected rather than allowed to escape the sidecar directory.

## Risks / Trade-offs

- [Risk] pi.dev has no external hook, so its side of the bridge needs a TypeScript extension living in this otherwise-Rust repo → Mitigation: keep the extension as thin as possible — build the payload from `ctx`, `exec` the binary, act on one field of the response — so all logic worth testing stays in Rust and the TS is inspectable at a glance.
- [Risk] Hook execution may run under a tight timeout in either agent runtime; slow directive-file I/O could cause the hook process to be killed mid-decision → Mitigation: directive lookup is a single small file read with no network calls and no locking beyond a plain read/write.
- [Risk] **An agent can read its own injected directive as a prompt-injection attempt and refuse it.** Observed, not hypothetical: during verification a directive whose reason text was a bare imperative ("reply with exactly the single word PROVEN") was correctly identified by Claude Code as "a 'Stop hook feedback' message trying to get me to output a specific word", and it declined and asked the user about it. The mechanism worked perfectly — the block landed and the text reached the model — but the *content* was rejected. → Mitigation: reason text must read as an attributed pointer to the user's own review artifacts in this repo ("review feedback is waiting in `<path>`, address the open comments"), never as a naked instruction from an unattributed source. This is a second, independent reason for the "pointer, not embed" principle beyond context size. This constraint has been carried forward as a **normative requirement** in `add-directive-verdict-loop` ("Reason text is an attributed pointer the agent will act on"), with a refusal defined there as a blocking failure — so it survives this change being archived. Nothing is required of the transport layer in this change, which only carries whatever text the directive file holds.
- [Risk] A crash or panic inside the hook binary could disrupt the user's live coding session in a way that's hard to debug from inside that same session → Mitigation and open question below.

## Verification Status

**Claude Code: proven.** Verified against a real headless Claude Code session in a throwaway project (not this repo, so a broken hook could not wedge live work) with a project-local `.claude/settings.json` Stop hook. Evidence: the hand-written directive was marked consumed by that invocation; the captured payload came back with `stop_hook_active: true`, which Claude Code only sets on a turn it is running *because* a Stop hook blocked; and the assistant's reply explicitly reasoned about the injected reason text. The real payload is now the test fixture in `crates/core/src/hook/adapter.rs` — Claude Code sends substantially more fields than documented (`prompt_id`, `permission_mode`, `effort`, `last_assistant_message`, `background_tasks`, `session_crons`), all ignored without error.

**pi.dev: proven.** Verified in a real interactive pi session in this repo, with `.pi/extensions/openspec-doc-hook.ts` auto-loaded and `OPENSPEC_DOC_BIN` pointed at the local build. The agent wrote a pending directive for its own session (via `$PI_SESSION_ID` from the bash tool, sidestepping the need to know the session id up front), ended its turn, and pi then started a fresh turn on its own with the directive's reason text delivered as a user message, which the agent acted on. `agent_end` fires, `ctx.sessionManager.getSessionId()`/`getSessionFile()`/`ctx.cwd` supply every payload field, the binary is invoked with that payload on stdin, and the directive is found, consumed, and re-injected. No stale-ctx error and no hang.

Two pi quirks make this impossible to reproduce from a scripted `pi -p` harness. Both turned out to be **harness-only** — interactive use is unaffected — but they are recorded here and in the extension because they will cost anyone trying to automate this a lot of time:

1. `--session-id` on a not-yet-existing session, and `--continue`, both perform a session *replacement*; a handler touching the captured `pi`/`ctx` afterwards throws "extension ctx is stale after session replacement". This also hit an unrelated third-party extension in the same run, so it is not specific to our code. It is why the handler sits on `agent_end` rather than the semantically-truer `agent_settled`.
2. An `agent_end` handler combined with `--session-id` made pi hang before any output, handler never firing. Without `--session-id` the identical extension works. Cause unknown.

The lesson generalizes: verify this mechanism interactively. A scripted harness exercises session lifecycle paths the real thing never takes.

**Not resolved by either verification: whether an agent complies with a directive.** Claude Code refused a bare-imperative directive as prompt injection; pi complied with a similarly-shaped one. That is not evidence the concern is moot — different agent, different model, and the pi directive was equally injection-shaped. Both runs prove *delivery*; neither proves *compliance* with realistic templates. That remains a live risk owned by `add-directive-verdict-loop`.

## Open Questions

Both questions this change opened are now resolved and recorded under Decisions: fail-fast on internal errors with no catch-and-downgrade layer, and a pi.dev payload/response schema we define ourselves because pi has no external hook to conform to.

Newly opened by verification, and deliberately left open: why an `agent_end` handler plus `--session-id` hangs pi. It does not block anything — `agent_end` is proven correct in interactive use, which is the only use that matters — so chasing it would be curiosity, not work. `agent_settled` is the semantically truer analogue of Claude Code's Stop and would be worth revisiting only if `agent_end` ever proves to fire too early in practice.
