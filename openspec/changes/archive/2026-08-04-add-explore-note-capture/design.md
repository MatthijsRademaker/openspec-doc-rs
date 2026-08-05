## Context

Three pieces already exist and only need connecting. `scratch::ensure_session` creates a session's note and is idempotent, but has no non-test caller. `scope::session` renders that note as the session page's only artifact and silently skips it when the file is absent. `add-directive-verdict-loop` made sessions discoverable, so the page now loads — and renders nothing, because the note was never created. What is missing is a moment at which creating the note is unambiguously correct.

Claude Code supplies that moment. Its `UserPromptExpansion` event fires "when a user-typed command expands into a prompt, before it reaches Claude", and its matcher filters on **command name** rather than on prompt text. For `UserPromptSubmit`, `UserPromptExpansion`, and `SessionStart`, a hook's plain stdout is added as context the model can see and act on. `session_id` is a field common to every hook's stdin, so the command needs nothing the documentation leaves unspecified.

## Goals / Non-Goals

**Goals:**
- Creating the note happens exactly when an exploration starts, so "a note exists if and only if an exploration happened" holds.
- The agent learns the note's resolved path at the moment it enters explore mode, from the hook's own output, rather than from documentation it has to remember.
- The command depends only on `session_id`, a documented common field.

**Non-Goals:**
- No enforcement in this change: nothing blocks a turn because the note is empty.
- No coverage of an exploration the model starts through the `Skill` tool.
- No change to how the note is promoted, rendered, or commented on.

## Decisions

- **Detect via `UserPromptExpansion` matched on command name, not by scanning prompt text in `UserPromptSubmit`.** Matching on name is what the event is for; scanning prompt text would fire on any message that merely mentions the command, including one asking a question about it. Alternative considered: `SessionStart`, which needs no detection at all — rejected because it creates a note for every session, including sessions that never explore, which breaks the promotion invariant below.
- **Create the note only on explore detection, never unconditionally.** `promote::check` treats "no note" as "nothing to promote". A note for every session would make every open session promote on the first new change directory, and two concurrent sessions would rename their notes onto the same `.openspec-doc/scratch/<change>.md` and collide in `comments::relocate`. Gating creation on an actual exploration keeps the guard meaningful.
- **Instruct via plain stdout rather than a `hookSpecificOutput` JSON envelope.** The documentation states stdout is added as context for this event specifically, so plain text is the supported path and avoids depending on a JSON shape that is documented for `UserPromptSubmit` and only implied for this event.
- **Print the fully resolved note path rather than a formula.** The hook knows `session_id`; making the agent interpolate `$CLAUDE_CODE_SESSION_ID` into a path template is an avoidable step that can silently produce the wrong file.
- **Reuse `scratch::ensure_session` rather than writing a seeded template.** An empty note is honest about the fact that nothing has been explored yet, and a placeholder would be indistinguishable, to the reviewer, from an exploration that produced nothing.

## Risks / Trade-offs

- [Risk] **A wrong matcher string fails silently.** The documentation gives the matcher's semantics ("command name", exact string or regex) but not a worked example for a colon-namespaced skill like `opsx:explore`, and a matcher that does not match produces no error — the hook simply never runs and the page stays empty exactly as it does now. This is indistinguishable from the bug being fixed. → Mitigation: a live check that the hook fires on a real `/opsx:explore` is a required task, not an optional one; no amount of unit testing can substitute for it.
- [Risk] Detection covers only user-typed commands, so an exploration the model begins through the `Skill` tool produces no note and the reviewer sees an empty page — the original symptom, in a narrower case. → Mitigation: documented as a known gap in this change rather than left to be rediscovered; covering it needs a `PreToolUse` hook whose matcher is the tool name `Skill` and whose command inspects `tool_input.skill`, which is a separate wiring with its own verification.
- [Risk] The agent receives the instruction and still writes nothing, leaving an empty note that renders as an empty artifact. → Mitigation: accepted for this change. An empty note is at least visible to the reviewer as evidence the exploration was detected, which an absent file is not. Enforcement via the Stop hook is the escalation if injection proves unreliable, and is deliberately deferred until there is evidence it is needed.
- [Trade-off] The hook fires on every `/opsx:explore`, including repeat invocations within one session. `ensure_session` is idempotent and leaves an existing note untouched, so the cost is a repeated instruction rather than lost work.
