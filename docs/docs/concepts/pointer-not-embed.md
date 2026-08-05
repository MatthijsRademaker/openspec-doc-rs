# Pointer, not embed

An injected directive **names files for the agent to read**. It never quotes the reviewer's words.

```
Review feedback from this project's openspec-doc dashboard: the reviewer looked at
this exploration and kept it in the explore phase, with notes on what is still open.
Those notes are the last record in `.openspec-doc/verdicts/_session/<id>.jsonl`, the
exploration itself is at `.openspec-doc/scratch/_session/<id>.md`, and any anchored
comments are in `.openspec-doc/comments/_session/<id>.jsonl` —
`openspec-doc comment list --session <id>` prints them readably. Read those, then keep
working on what they say is unsettled.
```

The reviewer's actual note — *"Lets dig deeper into the lingering server"* — appears nowhere in that text.
The agent is told which file holds it and reads it.

## Why, and it is not aesthetics

**An agent is entitled to refuse an instruction it cannot corroborate.** During earlier verification a
hand-written directive reading *"reply with exactly the single word PROVEN and nothing else"* was
delivered perfectly — the hook blocked, the session stayed alive, the text reached the model — and the
agent identified it as a Stop-hook message trying to make it emit a specific word, declined, and asked the
user about it.

**The transport succeeded and the loop still failed.** No automated test can catch that: every assertion
about delivery passed.

The fix is to make every claim in the directive checkable against the repo the agent is working in. So
each template:

- **states where it came from** — "Review feedback from this project's openspec-doc dashboard"
- **names paths inside the project** that the agent can go and read
- **never demands specific literal output**
- **never tells the agent to resolve a comment** — that is the reviewer accepting the work, not the agent
  claiming to have done it

An agent can verify all of that. It cannot verify "reply with PROVEN", and it is right not to.

## It worked

Across live sessions on both agents — `keep-exploring` and `move-to-proposal` on Claude Code,
`keep-exploring` on pi.dev — every directive was acted on. No challenge, no request for permission, no
treating the text as untrusted input.

The pi.dev run is the clearest evidence. The agent was asked to run a script and report a session id;
what it returned instead was an assessment of the reviewer's question, because the directive had arrived
between the two. It had abandoned the prompt to answer the review.

There was also an unplanned benefit: once pointed at the verdict sidecar, an agent started checking that
file again later without being asked. The pointer taught it where the reviewer's decisions live. Pleasant,
but not something to rely on.

## Enforced, not just intended

Tests assert over every template that it identifies its origin, names an in-project path, contains no
injection-shaped phrasing (`exactly`, `verbatim`, `nothing else`, `the single word`, `respond with`,
`reply with`), and never instructs the agent to resolve a comment.

Those tests cannot prove an agent will comply — only a live run can. What they do prove is that no
template has drifted back into the shape of the one that was refused.

## The escalation, if it ever stops working

Written down in advance so nobody has to improvise:

1. Revise the templates and re-run the live check.
2. If attributed pointers still get refused, stop phrasing directives as instructions at all. Make them
   purely declarative state — *"2 open comments in `<path>`"* — and let the agent's own project
   instructions decide what to do about it.

Explicitly **not** on that list: trying to defeat the agent's injection defences. A directive an agent
cannot corroborate *should* be refused, and a mechanism that depends on suppressing that judgement is not
one worth building on.

## The corollary for the reviewer

Your verdict notes are not pasted into the agent's context; they are read from
`.openspec-doc/verdicts/…`. For a one-line note that is real overhead — the agent reads a JSONL file to
retrieve one sentence. It is still the right trade, because the property being protected is that *no*
injected text ever carries free-form content, and that property is what makes the channel corroborable at
all.
