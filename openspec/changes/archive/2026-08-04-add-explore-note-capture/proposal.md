## Why

The dashboard's explore phase currently has nothing to show. A session page renders exactly one artifact — the scratch note at `.openspec-doc/scratch/_session/<session-id>.md` — and nothing in production ever creates or fills that file: `scratch::ensure_session` was built by `add-scratch-note-workflow` and left with no non-test caller. Observed live during `add-directive-verdict-loop`'s end-to-end run: a correctly registered session rendered a page with zero artifacts, so there was no text to select, no comment could be anchored, and the whole review loop had no entry point.

`/opsx:explore` will not fix this on its own. It is the OpenSpec CLI's own skill — a conversational thinking partner with no knowledge of `.openspec-doc/` — so an exploration driven through it stays in the transcript where the reviewer cannot see it. Prose in `AGENTS.md` asking the agent to keep a note is the current stopgap, and it is exactly the kind of instruction an agent skims past.

## What Changes

- Adds a `hook explore` command that detects the start of an exploration, creates the session's scratch note, and prints the note's resolved path as context the agent acts on immediately.
- Wires that command to Claude Code's `UserPromptExpansion` event, whose matcher filters on command name — so `/opsx:explore` is detected by name rather than by scraping prompt text.
- Gives `scratch::ensure_session` the production caller it was written for, at the one moment that preserves the invariant promotion depends on: a note exists if and only if an exploration actually started.
- Detection is deliberately narrow in this change: `UserPromptExpansion` fires only for commands the user types. An exploration the model starts by invoking the skill itself goes through the `Skill` tool and is not covered here.

## Capabilities

### New Capabilities
- `explore-note-capture`: explore-command detection, session scratch-note creation, and the context injection that tells the agent where to keep its exploration.

### Modified Capabilities
(none)

## Impact

Adds a subcommand to the `cli` crate and a caller for existing `core` scratch-note code; no library behaviour changes. Requires a `UserPromptExpansion` hook entry alongside the existing `Stop` entry in the agent's hook configuration. Unblocks the explore-phase half of `add-directive-verdict-loop`'s end-to-end proof, which cannot run without a note to comment on.

A wrong matcher string fails silently — the hook never fires and the page stays empty exactly as it does today — so this change is not complete on passing tests alone; it requires a live check that the matcher fires.
