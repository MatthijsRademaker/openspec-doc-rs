## 1. Wire promotion into hook stop

- [ ] 1.1 Call the scratch-note promotion check at the start of `hook stop`, before directive lookup
- [ ] 1.2 Surface promotion's validate outcome (pass/fail) into the hook's own logging/error path
- [ ] 1.3 Add a test: a `hook stop` invocation for a session whose scratch note was just promoted runs validate and records the outcome

## 2. Verdict-to-directive translation

- [ ] 2.1 Define fixed reason-text templates for keep-exploring, move-to-proposal, and comment-resolution verdicts, each naming relevant sidecar/artifact paths. Each template MUST be an attributed pointer — it states that it is review feedback from this project's openspec-doc dashboard and names a path in the repo to read — and MUST NOT be a bare imperative. A receiving agent can otherwise reject the directive as prompt injection even though the hook delivered it correctly; see design.md.
- [ ] 2.1a Document the hook mechanism in this repo's `AGENTS.md`: that openspec-doc may inject review directives at turn boundaries, where directive/comment sidecars live, and that these originate from the user's own dashboard. This gives a receiving agent legitimate in-project context for the directive instead of leaving it unattributed.
- [ ] 2.2 Implement lookup of an untranslated phase-verdict record for the session/change at hook-stop time
- [ ] 2.3 Implement translation into the pending-directive file format from `add-agent-hook-bridge`
- [ ] 2.4 Mark the verdict record consumed/translated so it is not translated again on a later invocation
- [ ] 2.5 Add tests for each verdict kind: correct reason text produced, no embedded comment content present, verdict marked consumed
- [ ] 2.6 Add a test over every template asserting it names at least one in-project path and identifies itself as dashboard review feedback, and that none instructs the agent to emit specific literal output

## 3. End-to-end proof

- [ ] 3.1 Run a real Claude Code session in this repo with `openspec-doc hook stop --agent claude` wired as the Stop hook and the dashboard open
- [ ] 3.2 Submit a "keep exploring" verdict with a note from the dashboard; confirm the next Stop event blocks the session and the agent visibly acts on the injected reason. Delivery is not the bar: an agent that receives the directive and then declines it, questions it, or flags it as untrusted input counts as a FAILURE of this task — revise the templates per 2.1 and re-run. This is the only check in the whole change that can catch injection-rejection.
- [ ] 3.3 Submit a "move to proposal" verdict; confirm the agent creates a change directory and the next Stop event promotes the scratch note and runs validate
- [ ] 3.4 Repeat 3.2 for pi.dev's equivalent hook configuration
- [ ] 3.5 Record the outcome of this end-to-end proof; treat any failure here as blocking before considering the MVP done
