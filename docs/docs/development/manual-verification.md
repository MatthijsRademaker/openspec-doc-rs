# Manual verification

How to exercise every review interaction by hand, against a throwaway project so nothing touches this
repo's own `openspec/` directory.

Every command here has been run verbatim. If one fails, that is a bug, not a typo in the doc.

Looking to *use* the tool rather than test it? That is the [Quickstart](/quickstart.md).

## 0. Automated tests first

```bash
cargo test -p openspec-doc-core -p openspec-doc-cli
```

157 tests. They cover the whole server side of the review loop — including that a new comment pushes an SSE
event and that the review fragment reflects it. `openspec` must be on `PATH`; `scratch::promote`'s
validation tests shell out to it.

`cargo test --workspace` adds the 33 server tests for 190 in total. All pass — the three `watch.rs` failures
this document used to warn about are gone, unexplained; see [Testing](/development/testing.md).

What automated tests **cannot** cover is the reason this document exists, and it is more than the browser:

- the client JavaScript — the mouse-selection gesture and the live swap (§5)
- whether a hook matcher actually fires, since a wrong one is a silent no-op
- whether an agent *complies* with an injected directive, as opposed to receiving it

The last two need a real agent session and are covered by §6.

## 1. Build

```bash
cargo build
```

The binary lands at `target/debug/openspec-doc`. Below it is referred to as `$BIN`.

## 2. Create a throwaway project

The dashboard discovers a project by finding `openspec/config.yaml`. It lists a **session** only if
that session has a directive record — a scratch note alone is not enough, which is the most common
reason a hand-made fixture shows an empty session list.

```bash
export BIN="$PWD/target/debug/openspec-doc"
export PROJ=$(mktemp -d)
export SID=0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74

mkdir -p "$PROJ/openspec/changes/add-widget/specs/widget" \
         "$PROJ/.openspec-doc/directives/_session" \
         "$PROJ/.openspec-doc/scratch/_session"

: > "$PROJ/openspec/config.yaml"

# What makes the session appear in the dashboard at all.
cat > "$PROJ/.openspec-doc/directives/_session/$SID.json" <<'JSON'
{"pending":false,"reason":"none","createdAt":"2026-07-31T08:00:00Z","consumedAt":null}
JSON

# The session's one commentable artifact.
cat > "$PROJ/.openspec-doc/scratch/_session/$SID.md" <<'MD'
# Exploration

The widget should cache aggressively.

## Open

Unclear how invalidation works.
MD

cat > "$PROJ/openspec/changes/add-widget/proposal.md" <<'MD'
## Why

Widgets are slow today.

## What Changes

- Add a widget cache.
MD

cat > "$PROJ/openspec/changes/add-widget/specs/widget/spec.md" <<'MD'
## ADDED Requirements

### Requirement: Widget cache
The system SHALL cache widgets.
MD

"$BIN" --root "$PROJ" summary
```

`summary` should report the root and one active change, `add-widget`.

## 3. Start the dashboard

```bash
"$BIN" --root "$PROJ" serve --port 8791 --no-open
```

Leave it running and use a second terminal for the commands below. `export` does not cross terminals,
so either re-run the three `export` lines there (`PROJ` needs the actual path — `echo $PROJ` in the
first terminal), or background the server in this one with `&`. Drop `--port` to let the OS pick one,
and drop `--no-open` to have it open a browser for you.

Open <http://127.0.0.1:8791/>. You should see one session and one change, both linked.

## 4. Walk the interactions from a terminal

This proves the server half without a browser. Selection is the only thing a browser does that `curl`
cannot, and the server does not trust it anyway.

### 4.1 Session page renders the scratch note

```bash
curl -s "http://127.0.0.1:8791/sessions/$SID" | grep -E 'data-artifact-path|Unclear how'
```

The note's source appears verbatim inside `<pre class="source">`, and the `<article>` around it carries
`data-artifact-path=".openspec-doc/scratch/_session/<id>.md"` — the path a comment will be anchored to.

### 4.2 Comment on a selection

The browser sends exactly these three fields. `selected_text` must appear in the file as written.

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST "http://127.0.0.1:8791/sessions/$SID/comments" \
  --data-urlencode "artifact_path=.openspec-doc/scratch/_session/$SID.md" \
  --data-urlencode 'selected_text=Unclear how invalidation works.' \
  --data-urlencode 'body=Which cache layer are you invalidating?'
```

`303` — the form redirects back to the page. Confirm it rendered and anchored:

```bash
curl -s "http://127.0.0.1:8791/sessions/$SID" | grep -E 'Comments \(|anchor exact|<blockquote>'
"$BIN" --root "$PROJ" comment list --session "$SID"
```

### 4.3 A stale selection is refused, not guessed at

```bash
curl -s -w '\n-> %{http_code}\n' -X POST "http://127.0.0.1:8791/sessions/$SID/comments" \
  --data-urlencode "artifact_path=.openspec-doc/scratch/_session/$SID.md" \
  --data-urlencode 'selected_text=text that was never written' \
  --data-urlencode 'body=x' | grep -E '<p>|->'
```

`400`, with `selected text "…" was not found in …`. Nothing is recorded.

### 4.4 Keep-exploring verdict

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST "http://127.0.0.1:8791/sessions/$SID/verdict" \
  --data-urlencode 'verdict=keep-exploring' \
  --data-urlencode 'notes=Answer the invalidation question before proposing.'

cat "$PROJ/.openspec-doc/verdicts/_session/$SID.jsonl"
```

Empty notes are a 400 — a keep-exploring verdict whose whole content is what remains open says nothing
if that content is blank:

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST "http://127.0.0.1:8791/sessions/$SID/verdict" \
  --data-urlencode 'verdict=keep-exploring' --data-urlencode 'notes=   '
```

### 4.5 Move-to-proposal verdict

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST "http://127.0.0.1:8791/sessions/$SID/verdict" \
  --data-urlencode 'verdict=move-to-proposal' --data-urlencode 'notes='
```

The page lists verdicts most-recent-first, so `move-to-proposal` now leads.

### 4.6 Change page: artifacts and spec deltas

```bash
curl -s http://127.0.0.1:8791/changes/add-widget | grep -E 'data-artifact-path|SHALL cache'
```

`proposal.md` and `specs/widget/spec.md` both render. `design.md` and `tasks.md` are absent from the
fixture, and absent artifacts are left out rather than rendered empty — add them and reload to see them
appear.

### 4.7 Comment on the proposal, then send to agent

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:8791/changes/add-widget/comments \
  --data-urlencode 'artifact_path=openspec/changes/add-widget/proposal.md' \
  --data-urlencode 'selected_text=Widgets are slow today.' \
  --data-urlencode 'body=Slow by what measure?'

curl -s -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:8791/changes/add-widget/verdict \
  --data-urlencode 'verdict=comment-resolution'

cat "$PROJ/.openspec-doc/verdicts/add-widget.jsonl"
cat "$PROJ/.openspec-doc/comments/add-widget.jsonl"
```

A verdict aimed at the wrong kind of scope is a 400 — a change has no explore phase to keep exploring:

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:8791/changes/add-widget/verdict \
  --data-urlencode 'verdict=keep-exploring' --data-urlencode 'notes=Notes.'
```

### 4.8 Anchor states: what happens when the file moves underneath a comment

The point of anchoring is that it degrades visibly rather than silently. Each step below is a
progressively worse fate for the same comment. Run them in order.

Grep for `anchor <state>` rather than `data-anchor-state`, since the latter also appears in the page's
CSS.

**Text pushed to a new offset → `fuzzy`.** The recorded offset is stale, but the text is still there:

```bash
printf '## Note\n\nInserted above.\n\n%s' "$(cat "$PROJ/openspec/changes/add-widget/proposal.md")" \
  > "$PROJ/openspec/changes/add-widget/proposal.md"
curl -s http://127.0.0.1:8791/changes/add-widget | grep -oE 'anchor (exact|fuzzy|orphaned|missing)[^<]*'
```

**Text rewritten, context intact → still `fuzzy`.** Deleting the commented sentence is *not* enough to
orphan the anchor: its heading (`## Why`) and the 80 bytes either side still resolve, so it lands where
that context ends. This is the design working, not a stale read:

```bash
sed -i 's/Widgets are slow today./Entirely different prose./' \
  "$PROJ/openspec/changes/add-widget/proposal.md"
curl -s http://127.0.0.1:8791/changes/add-widget | grep -oE 'anchor (exact|fuzzy|orphaned|missing)[^<]*'
```

**No landmark left at all → `orphaned`** (red border). Every fallback has to fail — text, before/after
context, and heading:

```bash
printf 'Completely unrelated content with no shared landmarks.\n' \
  > "$PROJ/openspec/changes/add-widget/proposal.md"
curl -s http://127.0.0.1:8791/changes/add-widget | grep -oE 'anchor (exact|fuzzy|orphaned|missing)[^<]*'
```

**File gone → `missing`.** Reported apart from `orphaned` so "the artifact left" and "the text left" are
distinguishable:

```bash
rm "$PROJ/openspec/changes/add-widget/proposal.md"
curl -s http://127.0.0.1:8791/changes/add-widget | grep -oE 'anchor (exact|fuzzy|orphaned|missing)[^<]*'
```

The comment survives all four; only its reported confidence changes. Restore the file to continue:

```bash
cat > "$PROJ/openspec/changes/add-widget/proposal.md" <<'MD'
## Why

Widgets are slow today.

## What Changes

- Add a widget cache.
MD
```

### 4.9 The live-update push

What an already-open page relies on. Hold the stream open, write a comment, watch the push arrive:

```bash
curl -siN http://127.0.0.1:8791/changes/add-widget/events > /tmp/sse.log &
SSE=$!
until grep -qi 'text/event-stream' /tmp/sse.log; do :; done   # wait for a real subscription

curl -s -o /dev/null -X POST http://127.0.0.1:8791/changes/add-widget/comments \
  --data-urlencode 'artifact_path=openspec/changes/add-widget/proposal.md' \
  --data-urlencode 'selected_text=Add a widget cache.' \
  --data-urlencode 'body=Comment made while the stream was open.'

until grep -q 'data: changed' /tmp/sse.log; do :; done
echo "pushed"; kill $SSE
```

> Wait for the `text/event-stream` header before posting. The handler subscribes while building the
> response, so a comment written too early lands before the watcher exists and no event fires. This
> race is why the automated test posts on a loop.

Then the fragment the page refetches — one element, not a document:

```bash
curl -s http://127.0.0.1:8791/changes/add-widget/review | head -5
```

## 5. The browser check (the only part that needs a human)

Everything above is server-side. These two steps are the client JavaScript, and nothing else verifies
it.

**5.1 Select-to-comment.** Open <http://127.0.0.1:8791/changes/add-widget>. Drag-select a phrase inside
one of the artifact boxes. The comment composer should appear, showing the artifact path and the text
you selected. Type a body and submit — the page reloads with your comment listed, anchored `exact`.

Selections that should behave sensibly:

- A phrase inside one paragraph → works.
- Markdown syntax itself, e.g. `## Why` including the hashes → works; the source is what is rendered.
- A selection spanning two artifact boxes → the composer records whichever artifact the selection
  started in; submitting will 400 if the combined text is not in that file. Loud, not silent.
- Selecting nothing (a click) → composer stays as it was.

**5.2 Two tabs, no reload.** *(This is `tasks.md` 4.2, the one open task.)*

1. Open the same change page in two browser tabs.
2. In tab A, select text and submit a comment.
3. **Without touching tab B**, watch tab B's comment list.

Tab B's comment count and list should update within about a second, and tab B must **not** flash or
reload — only the review-state block is replaced. Verify the no-reload part deliberately: start typing
into tab B's composer *before* commenting in tab A, and confirm your half-typed text survives the
update. A full reload would eat it, and that regression is exactly what this check exists to catch.

Also worth a look: edit `proposal.md` in your editor while a tab is open. The artifact does not
re-render (only the review state swaps), but any comment on the text you changed should shift to
`fuzzy` or `orphaned`.

## 6. The hook end of the loop

### 6.1 A verdict becomes a directive

The verdict written in §4.4 is enough — no hand-written directive needed.

```bash
PAYLOAD='{"session_id":"'"$SID"'","transcript_path":"/tmp/t.jsonl","cwd":"'"$PROJ"'"}'
echo "$PAYLOAD" | "$BIN" --root "$PROJ" hook stop --agent claude
echo "$PAYLOAD" | "$BIN" --root "$PROJ" hook stop --agent claude
```

The first prints `{"decision":"block","reason":…}`; the second prints `{"continue":true}`, because a
directive is consumed exactly once. Check `consumedAt` in the directive file for the audit trail, and
`verdicts/_session/$SID.translated` for the id of the verdict that produced it.

Confirm the reason text is a pointer: it should name the sidecar paths and **not** contain the verdict
notes you typed. See [Pointer, not embed](/concepts/pointer-not-embed.md).

### 6.2 The same directive delivered at prompt time

`hook prompt` is the other delivery point. §6.1 consumed the directive, so submit a fresh verdict first:

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST "http://127.0.0.1:8791/sessions/$SID/verdict" \
  --data-urlencode 'verdict=keep-exploring' \
  --data-urlencode 'notes=Still unsettled.'

echo '{"session_id":"'"$SID"'"}' | "$BIN" --root "$PROJ" hook prompt --agent claude
```

Prints the same reason text §6.1 produced, on **stdout** — that is what the prompt-submission contract adds
to model context. The `openspec-doc: injected …` line goes to stderr; check with `2>/dev/null` that stdout
carries the directive and nothing else, because anything else on stdout is injected too.

Consume-once spans both points:

```bash
echo "$PAYLOAD" | "$BIN" --root "$PROJ" hook stop --agent claude    # {"continue":true}, not a second block
echo '{"session_id":"'"$SID"'"}' | "$BIN" --root "$PROJ" hook prompt --agent claude   # nothing
```

**Then check it cannot refuse a prompt.** This hook sits in front of the human's own input, and on Claude
Code a `UserPromptSubmit` hook exiting 2 blocks the prompt outright, so every failure path must still exit
zero:

```bash
printf 'not json\n' > "$PROJ/.openspec-doc/verdicts/_session/broken.jsonl"
echo '{"session_id":"broken"}' | "$BIN" --root "$PROJ" hook prompt --agent claude; echo "exit=$?"
echo 'garbage'                 | "$BIN" --root "$PROJ" hook prompt --agent claude; echo "exit=$?"
echo '{"sessionId":"x"}'       | "$BIN" --root "$PROJ" hook prompt --agent claude; echo "exit=$?"
```

All three: `exit=0`, empty stdout, a reported error chain on stderr. A non-zero exit here is a defect even
though the same failure in `hook stop` is correct behaviour.

:::danger Two failure classes escape the fail-soft, and one of them refuses the prompt
The guarantee holds only for failures *inside* the delivery path. `main.rs` parses arguments and resolves
the project root **before** dispatching, so anything failing there never reaches the fail-soft wrapper:

```bash
cd /tmp && echo '{"session_id":"x"}' | openspec-doc hook prompt --agent claude; echo $?   # 1
echo '{"session_id":"x"}' | openspec-doc hook prompt --agnet claude;            echo $?   # 2
echo '{"session_id":"x"}' | openspec-doc hook prompt;                           echo $?   # 2
```

Exit 1 is survivable — the prompt goes through with a warning. **Exit 2 is not**: Claude Code refuses the
prompt, so a single typo in the hook config makes the session unusable, returning clap's usage text instead
of an answer. Verified on 2.1.223.

Every clap usage error takes this path. Until it is fixed, treat the hook command string as load-bearing
and paste it rather than typing it.
:::

### 6.3 The explore hook

```bash
rm -f "$PROJ/.openspec-doc/scratch/_session/$SID.md"
echo '{"session_id":"'"$SID"'","prompt_id":"p1","transcript_path":"/tmp/t.jsonl","cwd":"'"$PROJ"'","permission_mode":"default","hook_event_name":"UserPromptExpansion"}' \
  | "$BIN" --root "$PROJ" hook explore --agent claude
```

Prints the instruction naming the note's resolved path. Note it readies the **directory only** — the
`.md` must not exist afterwards, because an empty placeholder makes an agent's first write fail.

### 6.4 Promotion needs a claim

```bash
# A change appearing is not enough on its own.
echo "$PAYLOAD" | "$BIN" --root "$PROJ" hook stop --agent claude
ls "$PROJ/.openspec-doc/scratch/"        # still session-keyed only

printf '# Exploration\n\nSomething worth keeping.\n' > "$PROJ/.openspec-doc/scratch/_session/$SID.md"
"$BIN" --root "$PROJ" scratch claim --session "$SID" --change add-widget
echo "$PAYLOAD" | "$BIN" --root "$PROJ" hook stop --agent claude

cat "$PROJ/.openspec-doc/scratch/add-widget.md"          # promoted, content intact
cat "$PROJ/.openspec-doc/scratch/_session/$SID.md"       # redirect left behind
```

Then run `hook stop` twice more: the redirect must **not** be re-promoted, and the promoted note must not
be overwritten.

### 6.5 The parts that need a real agent

None of these can be faked from a terminal, and each has caught defects nothing else did:

- **Does the matcher fire?** Wire the hooks per [Agent hooks](/reference/hooks.md), type the explore
  command in a real session, and check the note directory exists. A wrong matcher is silent.
- **Does the agent comply?** Submit a `keep-exploring` verdict from the dashboard, poke the session, and
  watch what it does. Reading the named files and carrying on is a pass. Questioning the directive, or
  asking you whether to trust it, is a **failure** — revise the templates and re-run.
- **Does the feedback arrive at the start of the turn?** Same setup, but check *when*. The agent should act
  on the verdict in the same turn as your prompt, not in a turn that follows it. Compare `consumedAt` in the
  directive file against the mtime of whatever the agent wrote.

  Beware one trap: a probe prompt like `Reply with only: OK` is obeyed literally even when the directive is
  in context, which looks exactly like a delivery failure and is not one. Use a neutral prompt such as
  `Say hello.` and check the directive file to see whether delivery actually happened.
- **Does the criterion hold?** The whole point, and the only check that exercises it: leave anchored
  comments on an exploration, submit `move to proposal`, poke the session, and read the proposal the agent
  writes. It passes only if the comments shaped the proposal. A proposal that addresses them in a later
  revision is a failure of prompt-time delivery, not a pass.

## 7. Clean up

Stop the server with Ctrl-C in its terminal (or `kill %1` if you backgrounded it), then:

```bash
rm -rf "$PROJ" /tmp/sse.log
```

`$PROJ` is a `mktemp -d` directory, so nothing outside it was touched.

## Troubleshooting

| Symptom | Cause |
|---|---|
| Session list is empty | no `.openspec-doc/directives/_session/<id>.json`; sessions are discovered from directive records, not scratch notes |
| `no OpenSpec project found` | no `openspec/config.yaml` in the cwd or any parent — pass `--root` |
| `404` on a page you expected | the change directory or directive record does not exist; unknown keys are a 404 by design |
| `400 selected text … was not found` | the artifact changed since the page was rendered; reload and reselect |
| Comment posts fine but no SSE event | the stream was opened after the write, or the watcher fell back to polling (check stderr) and needs up to a second |
| `warning: no filesystem watcher … polling every 1s` | the `notify` backend could not start; updates still work, just slower |
| Two tabs do not sync | the client JS — check the browser console; the server side is testable with §4.9 |
