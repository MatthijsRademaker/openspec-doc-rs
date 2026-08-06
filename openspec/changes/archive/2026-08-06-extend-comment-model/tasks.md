## 1. An anchor becomes optional

- [x] 1.1 Change `Comment.anchor` to `Option<Anchor>` in `crates/core/src/comments/record.rs`
- [x] 1.2 Add a test that a sidecar line written before this change — with an `anchor` object present — still reads into a thread. The `.openspec-doc/comments/` records in this repository are real data and must survive.
- [x] 1.3 Add a sidecar writer for an unanchored comment that takes a scope key and a body and no selection
- [x] 1.4 Do not route it through `anchor::create`. That function's contract is that the selected text is found in the artifact, and an unanchored comment has no artifact to find it in; reusing it with an empty string would make the error case meaningless.
- [x] 1.5 Add a test that adding an unanchored comment to a scope with no artifacts on disk succeeds — that is the state a session is in before the agent writes anything

## 2. Resolution for a comment with no anchor

- [x] 2.1 Report an unanchored comment's resolution as its own state rather than as `Orphaned`. Orphaned means the anchor was lost, which is a defect the reviewer should see; unanchored means there was never one, which is normal.
- [x] 2.2 Add a test asserting the two are distinguishable, so an unanchored comment can never be rendered as drift

## 3. Editing

- [x] 3.1 Add an edit event to `Event` carrying the comment id, the new body, and a timestamp
- [x] 3.2 Add an `edit` writer that appends it, and fold it in `read` so the thread reports the latest body
- [x] 3.3 Keep the original comment event in the file untouched. Append-only is the project's stated storage rule and the edit history is the audit trail.
- [x] 3.4 Refuse an edit naming a comment id the sidecar does not contain, with an error that names the id
- [x] 3.5 Add tests: an edited comment reads back with the new body, two successive edits read back with the last, the original event is still present in the file, and an edit of an unknown id is an error
- [x] 3.6 Decide and record whether an edit may target a reply. **Settled: no.** A reply is the agent's report of what it did, and letting it be rewritten removes the record the reviewer is judging. This needed no guard of its own — `edit` resolves its id against comment ids only, so a reply id is an unknown comment. Recorded on `record::Edit`, on `sidecar::edit`, and in the spec delta, and asserted by test.

## 4. CLI

- [x] 4.1 Add unanchored comment creation to `openspec-doc comment add`, by making the selection argument optional rather than adding a second subcommand
- [x] 4.2 Add `openspec-doc comment edit --comment <id> --body <text>`
- [x] 4.3 Make `comment list` show an unanchored comment as unanchored rather than printing an empty quotation
- [x] 4.4 Add tests for both operations through the CLI surface, since the CLI is how the dashboard's behaviour is verified

## 5. Do not break the current dashboard

- [x] 5.1 Render a comment with no anchor in `crates/server/src/page/review.rs` without the artifact path and quotation lines
- [x] 5.2 Add a test that a scope containing one anchored and one unanchored comment renders both. This page is being replaced, but not yet, and a panic here would take the reviewer's dashboard down.

## 6. Specs

- [x] 6.1 State the optional anchor and the edit operation in this change's `specs/anchored-comments/` delta. The base `openspec/specs/anchored-comments/spec.md` is written by the archive step's delta sync, which is how `add-comment-thread-actions` updated it — editing it here would apply this delta twice.
- [x] 6.2 Restate the CLI subcommand requirement over the version `add-comment-thread-actions` left in the base spec, which lists `address` and `reopen`. The delta as originally written predates that change and would have dropped both operations from the requirement on sync.
- [x] 6.3 At archive, the base spec's Purpose paragraph lists the anchor states as `exact`, `fuzzy`, `orphaned`, or `missing`. It gains `unanchored`. A delta cannot restate a Purpose, so this rides with the sync rather than being expressible above. Applied during the sync: the list now ends "or as `unanchored` for a comment that was never attached to a passage at all".
- [x] 6.4 Checked the restated CLI requirement against the `cli-surface` spec: **no conflict, and nothing to change there.** That spec claims only the top-level subcommand list (`summary`, `serve`, `hook`, `comment`, `scratch`) and that each has its own `--help`; it makes no claim about `comment`'s operation set, so `edit` is covered by the existing help requirement rather than falsifying it. Whatever drift the roadmap records is elsewhere.

## 7. Coordinate with `add-comment-thread-actions`

- [x] 7.1 Settled before starting: that change was re-scoped to its model and CLI half, implemented, and archived as `2026-08-06-add-comment-thread-actions`. This change lands second and builds on the shape it left — `Status::Addressed`, `set_status`, and the `address`/`reopen` subcommands are all already there and untouched here. Nothing was merged into this change; the conflict was resolved by ordering.
