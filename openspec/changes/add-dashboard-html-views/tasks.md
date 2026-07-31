## 1. Templating setup

- [x] 1.1 Choose a minimal approach for shared page structure — **decided against a template engine**: rendering is shared Rust functions under `crates/server/src/page/`, matching the `format!`/`escape()` style the placeholder pages already used. `askama` would have added a dependency and a parallel `templates/` tree for sharing a function already provides. See design.md.
- [x] 1.2 Implement a shared comment-list rendering partial usable by both session and change pages — `page::review::fragment`, called by both pages and also served on its own at `/{scope}/{key}/review` for live updates

## 2. Session page

- [x] 2.1 Render the scratch note markdown and its anchored comments at `/sessions/<session_id>`
- [x] 2.2 Implement the select-to-comment UI, capturing selection against the raw markdown source and submitting via `add-anchored-comments` — the artifact is rendered as its own source, so the browser sends only `selection.toString()` and the server creates the anchor from the file on disk
- [x] 2.3 Implement phase-verdict controls (keep-exploring with free-text notes, move-to-proposal) and their own verdict record — `core::verdict`, an append-only sidecar at `.openspec-doc/verdicts/`

## 3. Change page

- [x] 3.1 Render proposal/design/tasks/spec-delta artifacts and their anchored comments at `/changes/<name>`
- [x] 3.2 Reuse the select-to-comment UI from the session page for change artifacts
- [x] 3.3 Implement the comment-resolution verdict control ("send to agent: address open comments")

## 4. Live updates

- [x] 4.1 Wire client-side SSE subscription (from `add-dashboard-server`) to re-render the comment list and verdict state without a full page reload — required extending the watcher to span both the change tree and the `.openspec-doc` sidecar tree, which it previously did not watch at all
- [ ] 4.2 Manually verify a comment added in one browser tab appears in a second tab open on the same page without reload — **needs a human with a browser.** The server half is proven: `a_new_comment_is_pushed_to_the_scopes_event_stream` and `a_new_verdict_is_pushed_to_the_scopes_event_stream` cover the push, `the_review_fragment_reflects_a_comment_added_after_the_page_was_served` covers the refetch, and both were also exercised against a running server. What is unverified is the ~30 lines of client JS that join them.

## 5. Manual verification

- [x] 5.1 Walk through: open a session page, comment on the scratch note, submit "keep exploring" with a note, confirm the verdict record is written — done against a running server over HTTP; the comment came back rendered with `anchor exact` and the verdict landed in `.openspec-doc/verdicts/_session/<id>.jsonl`. The mouse-selection gesture itself is 4.2's browser check.
- [x] 5.2 Walk through: open a change page, comment on `proposal.md`, submit "send to agent", confirm the verdict record is written — done the same way; the comment anchored under heading path `["Why"]` and the verdict landed in `.openspec-doc/verdicts/add-widget.jsonl`
