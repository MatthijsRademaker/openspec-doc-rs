## 1. The explore hook command

- [x] 1.1 Add a `hook explore --agent <claude|pi>` subcommand to the CLI
- [x] 1.2 Parse the hook payload for `session_id` only, since it is the one field documented as common to every hook event and the command needs nothing else
- [x] 1.3 Call `scratch::ensure_session` for that session, giving it the production caller it has lacked since `add-scratch-note-workflow`
- [x] 1.4 Print the instruction to stdout as plain text — not a `hookSpecificOutput` envelope — since stdout is the documented context path for this event. Include the fully resolved note path with the session id already substituted, and say the note is what the reviewer reads and anchors comments against.
- [x] 1.5 Add tests: a session with no note gets one, a note with content in it is left untouched, the emitted path carries the real session id, and repeat invocations are idempotent

## 2. Wiring and live verification

- [x] 2.1 Add a `UserPromptExpansion` hook entry matched on the explore command's name, alongside the existing `Stop` entry
- [x] 2.2 Determine the matcher string that actually matches a colon-namespaced skill name, and record the working value. The documentation gives the matcher's semantics but no worked example for this shape. **Answered from a captured payload rather than guessed:** the matcher is matched against `command_name`, which is the bare command with no leading slash and no namespace — `openspec-explore` for `/openspec-explore`. The configured `opsx:explore|openspec-explore` matches. The event also carries `expansion_type`, `command_args`, `command_source` and the full `prompt`; the real payload is now the `CLAUDE_EXPANSION` fixture in `adapter.rs`.
- [x] 2.3 Run a live `/opsx:explore` in a real session and confirm the hook fired: the note exists on disk and the instruction reached the agent. A matcher that does not match produces no error and leaves the session page empty exactly as the defect does, so this check cannot be skipped or inferred from passing tests. **Fired**, and caught a defect no test would have: the hook created an empty note, and the agent's first `Write` of its exploration failed because Claude Code will not write over a file it has not read. Fixed by readying only the directory — see 2.6.
- [x] 2.4 Confirm the note renders on the dashboard's session page and that text in it can be selected and commented on — the outcome the whole change exists for
- [x] 2.6 Replace note creation with directory preparation (`scratch::ensure_session` becomes `prepare_session`), so the agent's first write of the exploration cannot fail on a placeholder it has not read. Revise the spec's first requirement, which asserted the note is created.
- [x] 2.5 Confirm an ordinary `hook stop` for a session that never explored still creates no note, so the promotion invariant is intact

## 3. Documentation and known gaps

- [x] 3.1 Update `AGENTS.md`: the note path now arrives from the hook itself, so the standing instruction becomes a description of the mechanism rather than a path the agent must construct from `$CLAUDE_CODE_SESSION_ID`
- [x] 3.2 Record the uncovered case in this change's artifacts and in `AGENTS.md`: `UserPromptExpansion` fires only for user-typed commands, so an exploration the model starts through the `Skill` tool produces no note. Covering it needs a `PreToolUse` hook matched on the tool name `Skill` whose command inspects `tool_input.skill`.
