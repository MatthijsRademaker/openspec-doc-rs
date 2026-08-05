## Why

A directive can only reach the agent when a turn *ends*, because the Stop hook is the only delivery point. That works when the agent is mid-work: it was about to stop, the block turns the stop into an informed continuation, and nothing is wasted. It works badly in the case reviewers actually hit.

The reviewing rhythm is: the agent finishes and goes idle, the reviewer reads the note and submits a verdict, then pokes the session to wake it. Observed in session `a020f7e4` — verdict submitted at 08:48:54, directive not delivered until 08:52:48, at the end of a turn the agent had already spent on the reviewer's poke while blind to the feedback that poke existed to deliver. The reviewer's own prompt is the one turn guaranteed to run without the review context, which is precisely backwards.

`UserPromptSubmit` fires before the model sees a prompt, and its stdout is added to context. Delivering there puts the feedback at the *start* of the turn it belongs to.

## What Changes

- Points the move-to-proposal reason text at the session's comment sidecar, and instructs that the open comments be accounted for in the proposal. It currently names the exploration note and the verdict record but not the comments, so an agent told to formalize an exploration is not told that anchored feedback on it exists — the keep-exploring template names the sidecar, this one never did. Folded in here rather than shipped separately because it is the same defect as the timing one seen from a different angle: both are about the reviewer's comments failing to reach the agent at the moment it needs them.
- Adds a `hook prompt` command that delivers any standing directive as context at prompt time, reusing the existing verdict translation and consume-once machinery unchanged.
- Wires it to `UserPromptSubmit`, alongside the existing `Stop` and `UserPromptExpansion` entries.
- Whichever hook fires first consumes the directive, so a directive is delivered exactly once whether it arrives at prompt time or turn end. The `Stop` path keeps its role: it is what stops an agent from ending a turn while feedback is outstanding, and it remains the only path that works when no human prompt is coming.
- **BREAKING** for hook configuration only: getting the benefit requires adding the new entry. An unmodified configuration keeps working exactly as it does now, delivering at Stop.

## Capabilities

### New Capabilities
- `prompt-time-directive-delivery`: delivering a standing directive as prompt context, and sharing consume-once with the turn-end path.

### Modified Capabilities
- `directive-verdict-loop`: the move-to-proposal reason text gains the session's comment sidecar path and an instruction to account for the open comments. The `Stop` path's delivery behaviour is otherwise unchanged; this change adds a second delivery point in front of it.

## Impact

Adds one subcommand to the `cli` crate and one settings entry. No change to `translate`, to the directive file format, or to the verdict sidecars.

One risk carries real weight and is not present in the `Stop` path: this hook sits in front of the human's own input. A hook that fails here could refuse the prompt outright, which would leave the reviewer unable to type at all — a strictly worse failure than the missing feedback it exists to fix. The exit-code semantics for a failing `UserPromptSubmit` hook must be established by observation before this is considered done, not assumed from the `Stop` hook's behaviour.
