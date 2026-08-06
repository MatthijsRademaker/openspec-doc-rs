## 1. Read a title out of a scratch note

- [ ] 1.1 Add a title read to `crates/core/src/scratch/` that returns the text of the note's first level-one ATX heading, trimmed
- [ ] 1.2 Return `None` rather than a placeholder when the note is absent, empty, or has no level-one heading — a session with nothing written has no topic to report, and inventing one is a guess the reviewer would have to un-learn
- [ ] 1.3 Ignore headings inside fenced code blocks, so a note quoting `# Example` in a shell snippet does not become the session's title
- [ ] 1.4 Add tests: a normal note titles correctly, a note whose first heading is level two returns `None`, a `#` inside a fence is skipped, and a missing note is `None` rather than an error

## 2. A per-scope summary

- [ ] 2.1 Add a summary type in `crates/server/src/scope.rs` carrying the scope key, the title, the last-modified time of its artifacts, the open-comment count, and the standing verdict
- [ ] 2.2 Change session and change enumeration to return summaries rather than `Vec<String>`; delete the string-returning path rather than keeping both
- [ ] 2.3 Read the open-comment count and the standing verdict through the existing `comments` and `verdict` core readers — no new file parsing
- [ ] 2.4 Add a test that a scope with no comments and no verdict summarises without error, since that is every scope on its first render

## 3. Render it

- [ ] 3.1 Replace `page::list` with a rendering that shows title, id, last-modified, open-comment count and standing verdict per row
- [ ] 3.2 Show the id as secondary text on every row, not only when the title is missing — it is what gets pasted into `openspec-doc comment list --session`
- [ ] 3.3 Fall back to the id as the row's primary text when there is no title
- [ ] 3.4 Use the title as the `<h1>` and the `<title>` on scoped pages, with the id still shown
- [ ] 3.5 Keep every URL keyed on the id. Add a test asserting the index links to `/sessions/<id>`, so a later change cannot quietly make links depend on a mutable title
- [ ] 3.6 Add a test that a session whose note has no heading still renders and still links correctly

## 4. Mark the live session

- [ ] 4.1 Mark the session with the most recent directive or verdict activity as the live one, using the sidecar timestamps already on disk
- [ ] 4.2 Do not add a heartbeat or a liveness file for this. `add-dashboard-lifecycle` introduces a real heartbeat; until it lands, "most recently active" is the honest claim and the label should say that rather than "live"

## 5. Specs and docs

- [ ] 5.1 Update `openspec/specs/scratch-note-workflow/spec.md` with the title requirement
- [ ] 5.2 Update the `dashboard-html-views` index requirement
- [ ] 5.3 Check whether `docs/docs/` describes the index anywhere and update it if so

## 6. Verify against a real project

- [ ] 6.1 Run `serve` against this repository and confirm the index shows titles for the existing sessions, including the one whose note begins `# Exploring: the dashboard's look and feel`
- [ ] 6.2 Confirm a change page and a session page both render their title with the id still visible
