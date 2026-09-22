# Manual verification

How to exercise every review interaction by hand, against a throwaway project so nothing touches this
repo's own `openspec/` directory.

Every command here has been run verbatim. If one fails, that is a bug, not a typo in the doc.

Looking to *use* the tool rather than test it? That is the [Quickstart](/quickstart.md).

## 0. Automated tests first

```bash
cargo test -p openspec-doc-core -p openspec-doc-cli
```

346 tests. They cover the whole server side of the review loop — including that a new comment pushes an SSE
event and that the review fragment reflects it. `openspec` must be on `PATH`; `scratch::promote`'s
validation tests shell out to it.

`cargo test --workspace` adds the 63 server tests for 409 in total. All pass — the three `watch.rs` failures
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

## 4. Walk JSON boundary from terminal

Browser routes serve embedded Vue shell. Inspect scope data and mutation boundary under `/api`.

### 4.1 Scope detail returns rendered blocks

```bash
curl -s "http://127.0.0.1:8791/api/sessions/$SID" | jq '{kind,key,title,artifacts,commentCounts,standingVerdict}'
curl -s "http://127.0.0.1:8791/api/changes/add-widget" | jq '.artifacts[] | {path,blocks}'
```

Session artifact may be absent before exploration starts. Change response includes every present artifact in reading order. Each block carries sanitized `html`, exact `source`, and byte `range`.

### 4.2 Create comments

Anchored comment names source occurrence with `searchFrom`:

```bash
curl -s -X POST "http://127.0.0.1:8791/api/sessions/$SID/comments" \
  -H 'content-type: application/json' \
  --data '{"kind":"anchored","artifactPath":".openspec-doc/scratch/_session/'"$SID"'.md","selectedText":"Unclear how invalidation works.","searchFrom":0,"body":"Which cache layer are you invalidating?"}' | jq
```

Unanchored scope comment needs no artifact:

```bash
curl -s -X POST "http://127.0.0.1:8791/api/sessions/$SID/comments" \
  -H 'content-type: application/json' \
  --data '{"kind":"unanchored","body":"Reconcile cache terminology across note."}' | jq
```

Submit absent selection and confirm `400` JSON reason. Nothing is recorded:

```bash
curl -s -w '
-> %{http_code}
' -X POST "http://127.0.0.1:8791/api/sessions/$SID/comments" \
  -H 'content-type: application/json' \
  --data '{"kind":"anchored","artifactPath":".openspec-doc/scratch/_session/'"$SID"'.md","selectedText":"text never written","searchFrom":0,"body":"x"}'
```

### 4.3 Reply and reviewer status transitions

Recover comment id from scope detail, then exercise same core writers browser uses:

```bash
COMMENT=$(curl -s "http://127.0.0.1:8791/api/sessions/$SID" | jq -r '.comments[0].comment.id')
curl -s -X POST "http://127.0.0.1:8791/api/sessions/$SID/comments/$COMMENT/replies" \
  -H 'content-type: application/json' --data '{"body":"Response recorded."}' | jq -e '.author == "reviewer"'
curl -s -X POST "http://127.0.0.1:8791/api/sessions/$SID/comments/$COMMENT/status" \
  -H 'content-type: application/json' --data '{"status":"resolved"}' | jq
curl -s -X POST "http://127.0.0.1:8791/api/sessions/$SID/comments/$COMMENT/status" \
  -H 'content-type: application/json' --data '{"status":"open"}' | jq
```

`addressed` is intentionally rejected by dashboard endpoint. Agent sets that claim through CLI; reviewer can only accept it as resolved or reopen it.

### 4.4 Verdict without notes

Dashboard verdict body has no notes field. Feedback lives in comments, and empty composer is valid:

```bash
curl -s -X POST "http://127.0.0.1:8791/api/sessions/$SID/verdict" \
  -H 'content-type: application/json' --data '{"verdict":"keep-exploring"}' | jq
curl -s "http://127.0.0.1:8791/api/sessions/$SID" | jq '.standingVerdict'
```

Change uses `comment-resolution`; session advancing control uses `move-to-proposal`. Wrong-scope verdict returns `400`, and so does `{"verdict":"approved"}` — an approval carries the fingerprint of what it approves, so it goes through §4.6 instead.

### 4.6 Approval, staleness, and the bulk sweep

Approving over outstanding feedback is refused with the counts that blocked it; sweeping and approving is one request:

```bash
curl -s -X POST "http://127.0.0.1:8791/api/changes/add-widget/approval" \
  -H 'content-type: application/json' --data '{"act":"approve"}' | jq
curl -s -X POST "http://127.0.0.1:8791/api/changes/add-widget/approval" \
  -H 'content-type: application/json' --data '{"act":"resolve-all-and-approve"}' | jq -e '.approval.state == "approved"'
curl -s "http://127.0.0.1:8791/api/changes/add-widget" | jq '.approval'
```

Then check what the fingerprint covers, which is the decision most easily got wrong:

```bash
# tasks.md is not part of what was approved
printf -- '- [x] 1.1 Done\n' >> "$PROJ/openspec/changes/add-widget/tasks.md"
"$BIN" --root "$PROJ" approval state --change add-widget; echo "exit $?"   # approved, exit 0

# the proposal is
printf '\nRevised.\n' >> "$PROJ/openspec/changes/add-widget/proposal.md"
"$BIN" --root "$PROJ" approval state --change add-widget; echo "exit $?"   # stale, exit 1

"$BIN" --root "$PROJ" approval state --change no-such-change; echo "exit $?"  # error, exit 1
```

An unknown change is an error, not an unapproved change: a precheck answering "not approved" for a typo reports it as a review problem.

```bash
curl -s -X POST "http://127.0.0.1:8791/api/changes/add-widget/approval" \
  -H 'content-type: application/json' --data '{"act":"withdraw"}' | jq -e '.approval.state == "not-approved"'
```

### 4.5 Live-update push

```bash
curl -siN "http://127.0.0.1:8791/api/changes/add-widget/events" > /tmp/sse.log &
SSE=$!
until grep -qi 'text/event-stream' /tmp/sse.log; do :; done

curl -s -X POST "http://127.0.0.1:8791/api/changes/add-widget/comments" \
  -H 'content-type: application/json' \
  --data '{"kind":"unanchored","body":"Comment made while stream was open."}' >/dev/null

until grep -q 'artifactsChanged' /tmp/sse.log; do :; done
echo pushed; kill $SSE
```

Client refetches scope detail after event. Review-state events reconcile immediately; artifact events replace rendered blocks when no composer is dirty and defer with a visible notice while unsent text exists.

## 5. Embedded browser check

Run deterministic desktop and 390px lane:

```bash
cd web
bun run test:e2e
```

Lane builds frontend, embeds it in Rust binary, creates realistic scope data, and verifies:

- block comment on second repeated occurrence resolves `exact`
- rendered selection crossing inline markup reports server refusal
- anchored, fuzzy, orphaned, unanchored, addressed, and resolved states remain reachable
- reply, resolve, reopen, and second-tab SSE reconciliation work without leaving browser
- empty composer submits verdict without comment
- document remains primary, persistent controls do not cause horizontal overflow, and artwork yields at 390px
- keyboard focus, reduced motion, bundled fonts, same-origin assets, and console/request health

For manual inspection use repository-pinned Playwright CLI, not Vite screenshot:

```bash
cd web
bunx --bun playwright test --config playwright.config.ts --project=desktop --headed
bunx --bun playwright test --config playwright.config.ts --project=narrow --headed
```

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

Confirm reason text is a pointer: it should name comment sidecar and **not** contain comment bodies.
See [Pointer, not embed](/concepts/pointer-not-embed.md).

### 6.2 The same directive delivered at prompt time

`hook prompt` is the other delivery point. §6.1 consumed the directive, so submit a fresh verdict first:

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST "http://127.0.0.1:8791/api/sessions/$SID/verdict" \
-H 'content-type: application/json' --data '{"verdict":"keep-exploring"}'

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
- **Does an unapproved implementation get noticed?** Tick a task on a change nobody approved and end a
  turn. The turn should be blocked with a directive saying implementation ran ahead of the review, and the
  next turn boundary with the same state should say nothing — the report is once per state, not once per
  turn. Note what this does *not* do: it never stopped the work, and the directive says so itself.
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
| --- | --- |
| Session list is empty | no `.openspec-doc/directives/_session/<id>.json`; sessions are discovered from directive records, not scratch notes |
| `no OpenSpec project found` | no `openspec/config.yaml` in the cwd or any parent — pass `--root` |
| `404` on a page you expected | the change directory or directive record does not exist; unknown keys are a 404 by design |
| `400 selected text … was not found` | the artifact changed since the page was rendered; reload and reselect |
| Comment posts fine but no SSE event | the stream was opened after the write, or the watcher fell back to polling (check stderr) and needs up to a second |
| `warning: no filesystem watcher … polling every 1s` | the `notify` backend could not start; updates still work, just slower |
| Two tabs do not sync | the client JS — check the browser console; the server side is testable with §4.9 |
