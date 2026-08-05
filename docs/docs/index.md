# openspec-doc

A local review dashboard for [OpenSpec](https://github.com/Fission-AI/OpenSpec) projects, plus an agent
hook bridge.

You read a coding agent's exploration or proposal in a browser **while it is happening**, select text,
leave anchored comments, and submit a phase verdict. The hook bridge feeds that verdict back to the agent
at its next turn boundary, as a directive it acts on.

Everything is a plain file under `.openspec-doc/`. No database, no daemon, no cloud.

```
   agent explores                              reviewer reads
        │                                             │
        ▼                                             ▼
  hook explore ──► .openspec-doc/scratch/…  ──►  127.0.0.1 dashboard
                                                      │
                                        comment + phase verdict
                                                      │
                                                      ▼
   agent acts  ◄── hook stop blocks ◄── directive ◄── verdicts/…
        │
        └──► creates the change, runs `scratch claim` ──► note promoted
```

## Where to go

| If you want to | Read |
|---|---|
| Get it running and drive one loop | [Quickstart](/quickstart.md) |
| Know what is in scope and what is not | [Vision & MVP scope](/vision.md) |
| See what is shipped and what is open | [Roadmap](/roadmap.md) |
| Understand *why* it is built this way | [Concepts](/concepts/review-loop.md) |
| Look up a command, file, or route | [Reference](/reference/cli.md) |
| Work on the tool itself | [Development](/development/testing.md) |

## Status

The loop is closed and verified on live sessions across both Claude Code and pi.dev — including the one
thing no test could establish, that an agent acts on an injected directive rather than refusing it as
prompt injection.

Two things still need a terminal or insider knowledge: closing out a comment, and remembering to start
`serve` in the first place. Whether those sit inside the MVP boundary is an open scope question — see
[Roadmap](/roadmap.md).

This is a Rust rewrite of a TypeScript prototype, kept under `openspec-doc-rs-example/` for reference.
