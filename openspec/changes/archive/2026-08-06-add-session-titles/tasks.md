## 1. Read a title out of a scratch note

- [x] 1.1 Add a title read to `crates/core/src/scratch/` that returns the text of the note's first level-one ATX heading, trimmed
- [x] 1.2 Return `None` rather than a placeholder when the note is absent, empty, or has no level-one heading — a session with nothing written has no topic to report, and inventing one is a guess the reviewer would have to un-learn
- [x] 1.3 Ignore headings inside fenced code blocks, so a note quoting `# Example` in a shell snippet does not become the session's title
- [x] 1.4 Add tests: a normal note titles correctly, a note whose first heading is level two returns `None`, a `#` inside a fence is skipped, and a missing note is `None` rather than an error

## 2. A per-scope summary

- [x] 2.1 Add a summary type in `crates/server/src/scope.rs` carrying the scope key, the title, the last-modified time of its artifacts, the open-comment count, and the standing verdict
- [x] 2.2 Change session and change enumeration to return summaries rather than `Vec<String>`; delete the string-returning path rather than keeping both
- [x] 2.3 Read the open-comment count and the standing verdict through the existing `comments` and `verdict` core readers — no new file parsing
- [x] 2.4 Add a test that a scope with no comments and no verdict summarises without error, since that is every scope on its first render

## 3. Render it

- [x] 3.1 Replace `page::list` with a rendering that shows title, id, last-modified, open-comment count and standing verdict per row
- [x] 3.2 Show the id as secondary text on every row, not only when the title is missing — it is what gets pasted into `openspec-doc comment list --session`
- [x] 3.3 Fall back to the id as the row's primary text when there is no title
- [x] 3.4 Use the title as the `<h1>` and the `<title>` on scoped pages, with the id still shown
- [x] 3.5 Keep every URL keyed on the id. Add a test asserting the index links to `/sessions/<id>`, so a later change cannot quietly make links depend on a mutable title
- [x] 3.6 Add a test that a session whose note has no heading still renders and still links correctly

## 4. Mark the live session

- [x] 4.1 Mark the session with the most recent directive or verdict activity as the live one, using the sidecar timestamps already on disk
- [x] 4.2 Do not add a heartbeat or a liveness file for this. `add-dashboard-lifecycle` introduces a real heartbeat; until it lands, "most recently active" is the honest claim and the label should say that rather than "live"

## 5. Specs and docs

- [x] 5.1 Update `openspec/specs/scratch-note-workflow/spec.md` with the title requirement — done in the spec delta, not by hand: `openspec archive` applies the deltas to the main specs, so editing both would duplicate the requirement
- [x] 5.2 Update the `dashboard-html-views` index requirement — same. Both deltas were corrected to match what was built: the title is the note's *opening* heading, and the most-recently-active mark is now specified
- [x] 5.3 Check whether `docs/docs/` describes the index anywhere and update it if so

## 6. Verify against a real project

- [x] 6.1 Run `serve` against this repository and confirm the index shows titles for the existing sessions, including the one whose note begins `# Exploring: the dashboard's look and feel`
- [x] 6.2 Confirm a change page and a session page both render their title with the id still visible

## 7. Name a promoted session (added during review)

- [x] 7.1 Name a session whose note has been promoted away after the change it became, rather than falling back to its id — the redirect already records it, and the id-only row is the one the proposal set out to fix
- [x] 7.2 Keep the row's link and the page's identifier on the session id, so naming a session after a change never changes where it lives
