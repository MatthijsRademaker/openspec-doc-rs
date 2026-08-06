## Why

The dashboard index is a list of UUIDs. Two sessions render as `22222222-3333-4444-5555-666666666666` and `e62a2a6b-2417-4786-bf37-dde3a12281c3`, and choosing between them is guesswork. Nothing distinguishes the session running right now from one abandoned last week, and nothing says which is the exploration with feedback outstanding.

Everything needed to fix that is already on disk. The scratch note opens with a heading the agent wrote — `# Exploring: the dashboard's look and feel` — which is a better description of the session than any identifier could be, and it stays current for free because the agent keeps the note current. The comment and verdict sidecars already carry counts and the standing verdict. The index simply does not read any of it.

## What Changes

- A session gains a **title**, taken from the first level-one heading of its scratch note. No new state, no new file, no command to run: the agent already writes the heading.
- A session with no note, or a note with no heading, keeps showing its id. There is nothing to invent from and inventing one would be a guess.
- The index shows, per session and per change: the title, when its artifacts were last modified, how many comments are open, and the standing verdict.
- The **session id stays the identity and stays in the URL.** It is what the hooks key on and what every sidecar filename is built from, and a URL keyed on a mutable title breaks every link the moment the exploration's topic shifts.
- Scoped pages show the title as their heading, with the id still present and still copyable — it is the thing an operator pastes into `openspec-doc comment list --session`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `scratch-note-workflow`: gains title extraction from the note's first heading, as a read over a file the capability already owns.
- `dashboard-html-views`: the index requirement gains titles and per-scope status; the scoped-page requirements gain the title as the heading.

## Impact

- `crates/core/src/scratch/` — the title read. It belongs in `core` rather than the server because the CLI reports the same thing.
- `crates/server/src/page/mod.rs` — `index`, currently a bare `<ul>` of keys, and `list`, which takes `&[String]`.
- `crates/server/src/scope.rs` — session and change enumeration, which today returns names only and needs to return the per-scope summary the index renders.
- `openspec/specs/scratch-note-workflow/spec.md` and the `dashboard-html-views` spec.

## Relationship to the frontend rewrite

This is deliberately **independent of `replace-dashboard-frontend`** and shippable on the current server-rendered dashboard. The title is a `core` read and a summary struct; both survive the frontend rewrite unchanged, because the rewrite replaces the rendering and not the data behind it. The only throwaway work is the HTML of the index itself, which is roughly fifteen lines.

It is sequenced this way on purpose: the UUID problem is real today, and the rewrite is scheduled for after MVP agreement.
