## Context

`promote::check` decides that a session's exploration became change `X` because `X` appeared in `openspec/changes/` since the session's last check. Nothing in that inference involves the session. It is a correlation that holds when one session works at a time and breaks silently otherwise, and "silently" here means renaming an exploration and relocating its comment sidecar onto the wrong change.

The mechanism was chosen deliberately over asking the agent to record the change name — the current spec's purpose statement says so. What has changed since is the evidence: `add-directive-verdict-loop` proved on real sessions, across two different agents, that agents act on openspec-doc's instructions rather than ignoring or challenging them. An instruction to record what it just created is strictly easier than the ones already being followed.

## Goals / Non-Goals

**Goals:**
- Promotion happens because a session said which change it created, not because a directory appeared.
- Failing to claim degrades to "not promoted", never to promoting the wrong thing.
- An already-promoted session cannot be promoted again.
- The old mechanism is gone, not left as a fallback.

**Non-Goals:**
- No change to what promotion *does* once the target is known: rename, redirect, relocate comments, validate all stay exactly as they are.
- No attempt to infer a claim from a change's contents, git history, or timestamps.
- No migration of existing snapshots into claims — there is nothing in a snapshot that identifies a change as this session's.

## Decisions

- **The claim is written by a command, not hand-authored.** `openspec-doc scratch claim --session <id> --change <name>` writes the marker in the one format the reader accepts. Asking an agent to emit an exact HTML comment invites a near-miss that parses as no claim at all; asking it to run a command with resolved arguments is the shape it has already been observed following. The move-to-proposal template names the command with both arguments filled in, consistent with the existing rule that directives carry resolved paths rather than templates to interpolate.
- **The claim lives in the note, as a marker comment.** It reuses the `<!-- openspec-doc:… -->` convention already established by the moved-to pointer, and it travels with the note when promoted, which leaves the promoted note stating what it came from. Alternative considered: a separate `<session>.change` sidecar — rejected as another file to create, orphan and reason about, for no gain. The marker is visible in the dashboard because notes render as verbatim source; that is a small cosmetic cost against a data-corruption bug.
- **No fallback to the diff when a claim is absent.** A fallback would preserve exactly the behaviour being removed, and would fire precisely in the case the claim exists to disambiguate. An unclaimed exploration stays at its session key and stays visible; the reviewer can run the claim command themselves. Losing an automatic convenience is the correct failure.
- **The already-promoted guard is separate from the claim.** A session whose note begins with a moved-to pointer is refused outright, checked before anything else. This defect is independent — it would still exist if attribution were fixed by other means — so it gets its own guard and its own scenario rather than being absorbed into the claim logic.
- **Delete the snapshots rather than ignoring them.** Leaving `*.changes.json` on disk would leave a reader reconstructing a mechanism that no longer exists. Removing `Promotion::Ambiguous` at the same time is what makes the deletion real: while that variant exists, so does the concept it encodes.

## Risks / Trade-offs

- [Risk] **The agent formalizes an exploration and never claims it.** The note is then never promoted, its comments never follow it, and the reviewer sees a session-keyed note beside a change that clearly came from it. → Mitigation: accepted as the designed failure mode, and the reason the claim command is documented for human use. This trades a silent wrong promotion for a visible missing one, which is the whole point. If it turns out to be frequent, the escalation is for `hook stop` to report "this session has a note and no claim" on stderr — a nudge, not a guess.
- [Risk] The claim names a change directory that does not exist, or one belonging to another session. → Mitigation: promotion verifies the claimed change is an active change before acting, and refuses otherwise. A claim naming a change another session already promoted into is refused by the existing `SidecarExists` guard in `comments::relocate`, which is already the check that stops two notes landing on one change.
- [Risk] Removing the ambiguity guard removes a safety net that has in fact been catching real misfires — three stale sessions were saved by it during this investigation. → Mitigation: it was catching them by accident, only in the case where *more than one* change appeared. The single-change case, which is the common one, went straight through. A guard that fires on the harder case and not the easier one is not a net worth keeping.
- [Trade-off] Promotion is no longer fully automatic — it now depends on one extra action inside the turn that creates the change. That is a real reduction in autonomy, and it is the price of correctness. The alternative on offer is not "automatic and correct", it is "automatic and occasionally destructive".
