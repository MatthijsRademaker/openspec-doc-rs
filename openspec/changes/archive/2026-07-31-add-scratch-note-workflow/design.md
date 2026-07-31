## Context

During exploration, no `openspec/changes/<name>/` directory exists yet, so there is nothing for the anchor engine (`add-anchored-comments`) to bind comments to. The scratch note fills that gap. Once the agent formalizes the exploration by creating a real change directory, the note needs to move from a session-scoped key to a change-name-scoped key, matching the two scoping regimes already settled during exploration (session id pre-proposal, change name post-proposal).

## Goals / Non-Goals

**Goals:**
- Give explore-phase commenting a stable markdown file to anchor against, from the moment a session starts.
- Detect promotion from session-scoped to change-name-scoped without requiring the agent to follow an extra, easily-forgotten bookkeeping convention.
- Fold `openspec validate <name>` into the promotion round-trip so a newly formalized proposal's structural validity is known immediately, not discovered later.

**Non-Goals:**
- No dashboard UI (`add-dashboard-html-views`).
- No comment/anchor engine itself (`add-anchored-comments`).
- No wiring of promotion into the live hook invocation path — this change delivers promotion as a standalone library operation; `add-directive-verdict-loop` calls it from within the actual `hook stop` flow.

## Decisions

- **Promotion is detected by diffing the active-change list against a prior snapshot**, scoped to sessions with a pending scratch note, rather than requiring the agent to explicitly record which change name it chose. Alternative considered: have the agent write the chosen change name back to a session-side pointer file as part of formalizing — rejected as an invisible convention that adds a failure mode (the agent forgets or gets it wrong) for a problem that observational diffing already solves in the common single-session case.
- **Rename, don't copy-and-link**, per the earlier decision to keep one key per change forever rather than carrying two permanent key namespaces. The old session-scoped path is left with a "moved" pointer record (new path only, not a duplicate of content) so an already-open dashboard tab can redirect.
- **Validation runs synchronously as part of promotion**, and its outcome (pass/fail, output) is recorded alongside the promotion result — a failing validation is data the agent/dashboard needs immediately, not an afterthought.

## Risks / Trade-offs

- [Risk] If two unrelated new active changes appear between two consecutive checks for the same session (e.g. a human manually runs `openspec new change` for something else while an explore session is also active), diff-based promotion could misattribute the wrong change to the session's scratch note → Mitigation: concurrent multi-session/multi-actor usage was explicitly deprioritized for this MVP; if more than one new active change appears between checks, skip auto-promotion entirely and record an explicit ambiguity notice rather than guessing.
- [Risk] Renaming a file the dashboard/agent might be actively reading could race with a concurrent read → Mitigation: the moved-pointer record is written atomically after the rename completes, and readers that hit a missing file at the old path check for the pointer before treating it as an error.
