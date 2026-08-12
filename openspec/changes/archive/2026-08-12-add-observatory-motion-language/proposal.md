## Why

The dashboard's static observatory composition is distinctive, but its interaction feedback still relies on conventional fades, scrolling, and panel movement. Review actions should feel like precise instrument operations without turning document review into animated spectacle.

## What Changes

- Establish a shared event-driven motion language built around acquiring coordinates, triangulating source and discussion, transmitting reviewer intent, receiving remote updates, resolving state, and reconfiguring instrument chrome.
- Give index-to-scope and artifact-selection navigation a brief coordinate-acquisition response tied to real titles and exact paths.
- Make bidirectional source/thread navigation visibly connect its origin and destination across desktop and narrow layouts.
- Report successful comment, reply, status, and verdict mutations through truthful transmission and arrival feedback, while preserving loud failure behavior.
- Make open, addressed, resolved, verdict, and delivery changes readable through glyph and line-geometry transitions rather than color changes alone.
- Replace generic drawer and rail movement with restrained hard-edged chassis transitions.
- Preserve immediate content access, keyboard and history behavior, the 100–200ms transition envelope, global reduced-motion suppression, and the ban on perpetual ornament, parallax, decorative loaders, sound, and pointer-following effects.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-visual-system`: Define the observatory event-motion vocabulary, motion budget, responsive degradation, truthful mutation feedback, and reduced-motion behavior.
- `dashboard-html-views`: Apply coordinate acquisition, thread triangulation, transmission, receive, state metamorphosis, and chassis reconfiguration to existing index and scope interactions without changing their navigation, focus, mutation, or live-update contracts.

## Impact

Affected code is limited to the Vue dashboard under `web/src`: Router-owned index and scope navigation, artifact and conversation components, decision controls, centralized visual tokens and transitions, arrival-mark behavior, deterministic mock fixtures, component tests, and embedded Playwright coverage. No Rust API contract, runtime raster set, frontend package manager, motion dependency, or embedded-asset boundary changes are expected.
