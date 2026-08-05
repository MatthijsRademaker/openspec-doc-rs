## Why

Promotion guesses. It diffs the active-change list against a per-session snapshot and credits the single new directory to that session, with nothing establishing that the session had anything to do with creating it. In a repo with one session at a time that guess is usually right. This repo routinely has several, and then it is wrong silently and destructively.

Three defects, all observed on real state rather than reasoned about:

- **No attribution.** A change directory created from one session is credited to every session whose snapshot predates it. Creating `add-prompt-time-directive-delivery` from a second session left session `a020f7e4` one turn boundary away from promoting a 29KB exploration onto it and relocating that exploration's comment sidecar with it. Caught by hand, before the Stop fired.
- **No already-promoted guard.** After promotion the vacated session path holds a redirect pointer, and `check` tests only that the note path *exists*. A promoted session therefore stays eligible forever: the next single new change makes it rename its own 160-byte redirect onto that change's note path, destroying the note already there.
- **Stale sessions accumulate.** Five sessions currently hold snapshots; three were stale by two or three changes, two of those still holding live 9KB explorations. Each is a latent mis-promotion that fires the moment exactly one change appears. They are presently safe only because *more than one* change appeared and tripped the ambiguity guard — luck, not design.

The snapshot diff was chosen over asking the agent to record the change name it created. That call was made before there was any evidence about whether agents follow openspec-doc's instructions. There is now: across `add-directive-verdict-loop`'s end-to-end proof, agents followed directive templates precisely on both Claude Code and pi.dev — creating the change, reporting back with `openspec-doc comment reply`, and leaving thread status alone as instructed. The premise the original decision rested on no longer holds.

## What Changes

- **BREAKING** Removes snapshot-diff promotion detection entirely: the per-session `*.changes.json` snapshots, the baseline-only first check, and the multiple-candidates ambiguity guard all go. There is nothing left to be ambiguous about.
- Adds an explicit claim: a session records which change its exploration became, and promotion acts on that claim alone.
- Adds `openspec-doc scratch claim` so the claim is written by a command rather than by an agent hand-authoring a marker.
- Extends the move-to-proposal directive template to name that command with its arguments already resolved.
- Promotion refuses a session whose note is a redirect from an earlier promotion, closing the second defect independently of the first.
- A session that never claims is never promoted. Its note stays at the session key, visible in the dashboard. Degrading to "not promoted" is the point: it loses an automatic convenience instead of destroying an exploration.

## Capabilities

### New Capabilities
(none — this replaces the detection mechanism inside an existing capability.)

### Modified Capabilities
- `scratch-note-workflow`: promotion detection changes from a snapshot diff to an explicit claim; the ambiguity guard and baseline snapshot are removed; promotion gains a guard against an already-promoted session.
- `explore-note-capture`: the instruction that starts an exploration names the claim command instead of promising promotion when a change directory appears. Found during the end-to-end run, not by inspection: the text was telling every agent that entered explore mode how the removed mechanism worked.

## Impact

Deletes `crates/core/src/scratch/snapshot.rs` and its tests, rewrites `promote::check`, and adds a CLI subcommand. `Promotion::Ambiguous` disappears from the public enum, so the `hook stop` reporting path narrows with it. The move-to-proposal reason template gains one sentence.

Existing `*.changes.json` files become dead and SHALL be deleted rather than left to confuse the next reader. Sessions currently mid-exploration lose nothing: they were never going to be promoted correctly without a claim.

The purpose statement in `openspec/specs/scratch-note-workflow/spec.md` names the snapshot diff and the rejected alternative explicitly, and must be rewritten — a delta that changes requirements while leaving the purpose contradicting them is worse than not having written it.
