# Conventions

## Crate layout

```
crates/core     project discovery, comments, scratch notes, verdicts, hook bridge   (no I/O framework)
crates/server   the axum dashboard: routes, pages, filesystem watcher, SSE
crates/cli      the openspec-doc binary
```

`core` holds **every rule and every file format**. `server` and `cli` are two front ends over it and
contain no persistence logic of their own. This is what makes the CLI a real check on the dashboard: both go
through the same code, so `comment list` shows exactly what a browser interaction recorded.

It is also what keeps two agents supportable. Decision logic lives in `core`; the agent-specific parts are
a thin adapter that translates wire formats and nothing else.

## Working rules

`AGENTS.md` holds the full set. The ones that show up most in the code:

**Invalid states fail loudly.** No silent defaults, swallowed errors, or fake success values. A malformed
sidecar line is an error, not a skipped record.

**Errors propagate with their source chain intact.** The CLI prints `caused by:` lines.

One deliberate exception, and it is written into a spec so it cannot be mistaken for sloppiness: a hook
standing between the reviewer and their own prompt fails **soft**. Refusing the human's input is worse than
the problem being reported. Everywhere else, fail fast.

**Prefer small feature-named files over large ones.** Navigation beats file count.

**Prefer executable truth in `src/` when docs disagree.** Including these docs.

**No backwards compatibility.** Old paths, flags, and schemas get replaced and deleted, not preserved. When
snapshot-based promotion was replaced, `snapshot.rs`, `Promotion::Ambiguous` and `Error::Snapshot` all went
with it — while the variant exists, so does the concept it encodes.

**Remove dead code.** Twice now, a function has been found with no non-test caller
(`scratch::ensure_session`, and the whole snapshot API). Both times that absence *was* the bug: the note was
never being created, and the mechanism was never being invoked.

## Documenting decisions in code

Comments explain **why**, especially where the obvious thing is wrong. Some examples worth imitating:

- `translate.rs` opens by explaining that a bare imperative directive was refused as prompt injection, so
  the templates below are attributed pointers.
- `explore.rs` explains that only the *directory* is created, because an empty placeholder makes the
  agent's first write fail.
- `note.rs` explains that a note exists exactly when an exploration was written, and what promotion leans
  on that for.

Each of those encodes a live failure. Without the comment the next person deletes the guard.

## OpenSpec workflow

Changes go through `openspec/changes/<name>/` with proposal, design, spec deltas, and tasks, then archive
into `openspec/changes/archive/` with their specs synced into `openspec/specs/`.

Two habits worth keeping:

**Record what a live run found, in the task.** Several tasks now carry a `**PASSED**` note with the session
id and what the agent actually did. That is the only durable record of behaviour no test captures.

**When implementation reveals a defect, add the task rather than absorbing it silently.** Tasks like
`1.4`–`1.7` and `3.3a` exist because something was found mid-implementation. The alternative — fixing it
quietly — is how a change's scope runs away from whoever is reviewing it.
