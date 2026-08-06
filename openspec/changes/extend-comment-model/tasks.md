## 1. An anchor becomes optional

- [ ] 1.1 Change `Comment.anchor` to `Option<Anchor>` in `crates/core/src/comments/record.rs`
- [ ] 1.2 Add a test that a sidecar line written before this change — with an `anchor` object present — still reads into a thread. The `.openspec-doc/comments/` records in this repository are real data and must survive.
- [ ] 1.3 Add a sidecar writer for an unanchored comment that takes a scope key and a body and no selection
- [ ] 1.4 Do not route it through `anchor::create`. That function's contract is that the selected text is found in the artifact, and an unanchored comment has no artifact to find it in; reusing it with an empty string would make the error case meaningless.
- [ ] 1.5 Add a test that adding an unanchored comment to a scope with no artifacts on disk succeeds — that is the state a session is in before the agent writes anything

## 2. Resolution for a comment with no anchor

- [ ] 2.1 Report an unanchored comment's resolution as its own state rather than as `Orphaned`. Orphaned means the anchor was lost, which is a defect the reviewer should see; unanchored means there was never one, which is normal.
- [ ] 2.2 Add a test asserting the two are distinguishable, so an unanchored comment can never be rendered as drift

## 3. Editing

- [ ] 3.1 Add an edit event to `Event` carrying the comment id, the new body, and a timestamp
- [ ] 3.2 Add an `edit` writer that appends it, and fold it in `read` so the thread reports the latest body
- [ ] 3.3 Keep the original comment event in the file untouched. Append-only is the project's stated storage rule and the edit history is the audit trail.
- [ ] 3.4 Refuse an edit naming a comment id the sidecar does not contain, with an error that names the id
- [ ] 3.5 Add tests: an edited comment reads back with the new body, two successive edits read back with the last, the original event is still present in the file, and an edit of an unknown id is an error
- [ ] 3.6 Decide and record whether an edit may target a reply. Recommendation: no — a reply is the agent's report of what it did, and letting it be rewritten removes the record the reviewer is judging.

## 4. CLI

- [ ] 4.1 Add unanchored comment creation to `openspec-doc comment add`, by making the selection argument optional rather than adding a second subcommand
- [ ] 4.2 Add `openspec-doc comment edit --comment <id> --body <text>`
- [ ] 4.3 Make `comment list` show an unanchored comment as unanchored rather than printing an empty quotation
- [ ] 4.4 Add tests for both operations through the CLI surface, since the CLI is how the dashboard's behaviour is verified

## 5. Do not break the current dashboard

- [ ] 5.1 Render a comment with no anchor in `crates/server/src/page/review.rs` without the artifact path and quotation lines
- [ ] 5.2 Add a test that a scope containing one anchored and one unanchored comment renders both. This page is being replaced, but not yet, and a panic here would take the reviewer's dashboard down.

## 6. Specs

- [ ] 6.1 Update `openspec/specs/anchored-comments/spec.md` with the optional anchor and the edit operation
- [ ] 6.2 Restate the CLI subcommand requirement to include `edit`, and check it against the `cli-surface` spec, which the roadmap already records as drifted

## 7. Coordinate with `add-comment-thread-actions`

- [ ] 7.1 Before starting, settle whether that change is being re-scoped or merged into this one. Both edit `Status`, `Event`, and the same CLI group, and running them in parallel means resolving the same conflict twice.
