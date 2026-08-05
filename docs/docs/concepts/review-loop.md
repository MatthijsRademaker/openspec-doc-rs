# The review loop

The whole tool is one cycle. Each step is a plain file write that some other step reads.

## 1. The exploration becomes readable

When the owner types the explore command, a `UserPromptExpansion` hook runs `openspec-doc hook explore`.
That readies `.openspec-doc/scratch/_session/<session-id>/` and prints the note's **resolved** path into
the agent's context, telling it to write its exploration there.

Two details that look like accidents and are not:

- It readies the **directory**, not the file. Claude Code refuses to write over a file it has not read,
  so an empty placeholder turns the agent's first write into a failure. Observed live before it was
  fixed.
- The path is printed already resolved. An agent asked to interpolate its own session id into a template
  can interpolate it wrongly, and a note nobody is reading looks exactly like an exploration nobody
  wrote.

A note therefore exists exactly when an exploration has been written down.

## 2. The session becomes visible

The dashboard discovers a session by its directive record, so `hook stop` writes an **empty directive
slot** — never pending, never consumed — the first time it sees a session.

Without this the loop has no entry point at all: a session is listed because it has a directive record, a
directive comes from a verdict, and a verdict can only be submitted from the session's own page. That
circle was closed by hand for a while before anyone noticed it was a circle.

## 3. The reviewer responds

On the session page the note renders as its own markdown source. The reviewer selects text and comments
(see [Anchoring](/concepts/anchoring.md)), and submits a **phase verdict**:

| Verdict | Scope | Meaning |
|---|---|---|
| `keep-exploring` | session | Stay in the explore phase; notes say what is still open (**required**) |
| `move-to-proposal` | session | Ready to be formalized into a change |
| `comment-resolution` | change | Address the change's open comments |

Comments and verdicts are different channels. Comments accumulate while the reviewer is still forming a
view; **the verdict is the trigger that sends anything to the agent**. A comment with no verdict behind it
reaches nobody — deliberate, since otherwise every comment would nag an agent mid-thought.

## 4. The verdict becomes a directive

At the next turn boundary `hook stop` translates the *standing* untranslated verdict into a pending
directive and blocks the turn, feeding the directive's reason text back into the session.

The reason text is an attributed pointer, never a quotation of the reviewer — see
[Pointer, not embed](/concepts/pointer-not-embed.md), which is the single most load-bearing decision here.

A directive is consumed exactly once, marked before it is emitted, so a failed write can never inject
something that fires again at the next boundary.

:::tip Nothing wakes an idle agent
Translation happens *inside* a turn boundary. Submitting a verdict while the agent sits idle does
nothing until you send that session a message. There is no watcher process by design — a directive is
only ever consumed at a turn boundary anyway, and a daemon would add a lifecycle nobody wants to own.

The cost is that the turn you use to poke the agent runs blind: the feedback lands at the *end* of it.
That is what `add-prompt-time-directive-delivery` addresses.
:::

## 5. The exploration is formalized

A `move-to-proposal` directive tells the agent to create the change and then run:

```bash
openspec-doc scratch claim --session <id> --change <name>
```

That writes a claim marker into the note. At the following turn boundary, promotion sees the claim,
renames the note to `.openspec-doc/scratch/<change>.md`, leaves a redirect behind, relocates the comment
sidecar so existing anchors keep resolving, and runs `openspec validate <change>`.

**Promotion acts only on a claim.** It used to diff the active-change list against a per-session snapshot
and credit the single new directory to that session — which, with more than one session open, silently
renamed the wrong exploration onto the wrong change. An unclaimed exploration now simply stays where it
is. Losing an automatic convenience beats destroying an exploration; see
[Scoping](/concepts/scoping.md).

## What the loop does not do

It informs; it does not gate. Nothing blocks implementation on a reviewer's approval, and no verdict is
mandatory. That is [deliberately out of scope](/vision.md#what-is-deliberately-excluded).
