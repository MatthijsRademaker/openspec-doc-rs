# Scoping: session and change keys

Every route and every sidecar is keyed one of two ways. This is the most load-bearing idea in the
codebase, and most of the file layout follows from it.

- **Session-keyed** while an exploration is still pre-proposal. There is no change directory yet, so the
  only artifact is a scratch note.
- **Change-keyed** once the exploration has been formalized into `openspec/changes/<name>/`.

```
.openspec-doc/comments/_session/<session-id>.jsonl   session-keyed
.openspec-doc/comments/<change-name>.jsonl           change-keyed
```

Session-keyed files sit one directory deeper. That is not tidiness — it is what makes it **impossible**
for a session id to collide with a change name. Both are user-supplied strings in the same namespace
otherwise.

Session ids are additionally validated: empty, `.`, `..`, or anything containing a path separator is
refused, so a session id can never escape its directory.

## Promotion moves the key

When an exploration becomes a change, its note and its comments move together:

| Before | After |
|---|---|
| `scratch/_session/<id>.md` | `scratch/<change>.md` |
| `comments/_session/<id>.jsonl` | `comments/<change>.jsonl` |

The vacated session path is not deleted. It gets a machine-readable redirect:

```markdown
<!-- openspec-doc:moved-to .openspec-doc/scratch/add-widget.md -->

This scratch note moved to `.openspec-doc/scratch/add-widget.md`.
```

Relocating the sidecar appends a **relocation event** rather than rewriting history, so anchors recorded
against the old path keep resolving. Append-only means replaying the stream in order reconstructs current
state, and a later record never rewrites an earlier one.

## Promotion requires a claim

Promotion happens only for a session that has claimed the change its exploration became:

```bash
openspec-doc scratch claim --session <id> --change <name>
```

The claim is a marker inside the session's own note. Promotion reads that and nothing else.

### Why not just detect it?

It used to. Promotion diffed the active-change list against a per-session snapshot and treated the single
new directory as that session's change. It was wrong in two ways, both found by inspection rather than by
any test:

**A change directory says nothing about which session created it.** With two sessions open, one session
creating a change made *every* session whose snapshot predated it eligible to claim it. Live instance: one
session was a single turn boundary away from renaming a 29 KB exploration onto an unrelated change and
dragging its comment sidecar along. The `>1 candidate = ambiguous` guard did not help, since only one
directory had appeared.

**An already-promoted session stayed eligible forever.** After promotion the vacated path holds a
redirect, and the check tested only that the note path *existed*. The next single new change would make
the session rename its own 160-byte redirect onto that change, destroying the note already there.

Both are gone, along with the ambiguity guard — which only ever caught the two-at-once case and waved the
common single-change case straight through. A guard that fires on the harder case and not the easier one
is not worth keeping.

### The cost

Promotion is no longer fully automatic: it depends on the agent running one command in the turn where it
creates the change. If it forgets, the note stays at its session key and its comments never follow —
visible, recoverable by running the command yourself, and not destructive.

That trade is the point. The alternative was never "automatic and correct"; it was "automatic and
occasionally destructive".
