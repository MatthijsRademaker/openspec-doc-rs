## 1. Confirm the pi event before building on it

- [x] 1.1 Read the installed pi version's extension type declarations and confirm `pi.on("input")` exists, what its payload field carrying raw text is called, and what a handler may return
- [x] 1.2 Confirm raw input reaches that handler before skill and template expansion, by observing what `/opsx-explore` and `/skill:openspec-explore` each look like at that point in a live session
- [x] 1.3 If `input` does not exist or cannot see raw text on that version, stop and re-decide with the repository owner rather than falling back to `before_agent_start`

## 2. Explore detection in the pi extension

- [x] 2.1 Add an `input` handler to `crates/cli/assets/openspec-doc-hook.ts` that matches the explore command at the start of the raw input, covering both `/opsx-explore` and `/skill:openspec-explore`
- [x] 2.2 Run `openspec-doc hook explore --agent pi` from that handler with the same `{"sessionId": …}` payload the existing handlers send, reusing `runHook` rather than adding a second spawn path
- [x] 2.3 Deliver the command's stdout to the model as an `openspec-doc-directive` message with `display: true`, matching how `before_agent_start` delivers a directive
- [x] 2.4 Surface a spawn failure or non-zero exit through `ctx.ui.notify` and let the input proceed unchanged
- [x] 2.5 Skip the handler for a session with no session file, as the existing handlers do
- [x] 2.6 Document on the handler why matching is on raw input rather than the expanded prompt, and why the command is not registered with `pi.registerCommand`
- [x] 2.7 Regenerate this project's copy with `openspec-doc init --yes` and confirm `init` then reports `.pi/extensions/openspec-doc-hook.ts` unchanged

## 3. Retire the gap reporting

- [x] 3.1 Remove the pi exploration-note warning from `crates/cli/src/init/mod.rs`
- [x] 3.2 Remove or invert the init test asserting that warning, so a reappearing warning fails the suite
- [x] 3.3 Delete the `**Known gap, pi:**` paragraph and the two paragraphs of route analysis following it from `AGENTS.md`, leaving the Claude Code `Skill`-tool known gap untouched
- [x] 3.4 Confirm `crates/cli/assets/agents-block.md` needs no change, and that `openspec-doc init` still reports `AGENTS.md` unchanged after the deletion

## 4. Verification

- [x] 4.1 In a live pi session, run `/opsx-explore`, write an exploration, and confirm `.openspec-doc/scratch/_session/<session-id>.md` holds it
- [x] 4.2 In a live pi session, invoke `/skill:openspec-explore` and confirm the same
- [x] 4.3 Confirm the resolved note path and the session review URL reach the model as a displayed message, rather than the agent guessing a path
- [x] 4.4 Confirm the pi session appears in the dashboard index and its page renders the note with text that can be selected and commented on
- [x] 4.5 Submit a pi message that mentions the explore command without invoking it, and confirm no exploration is started
- [x] 4.6 Confirm `/opsx-explore` still expands to `.pi/prompts/opsx-explore.md` and was not shadowed
- [x] 4.7 Claim the note to a change from the pi session and confirm the next turn boundary promotes it
- [x] 4.8 Run `make check` and `make test`
