---
name: browser-verification
description: Verify openspec-doc's embedded dashboard with the repository Playwright lane; use Playwright CLI for inspection without a global install.
license: MIT
compatibility: Requires Bun, Cargo, and the repository Playwright dependency.
metadata:
  author: openspec-doc
  version: "1.0"
---

# openspec-doc browser verification

Use this skill for browser behavior, responsive review, visual inspection, and asset-health
claims.

## Completion lane

From `web/`, run:

```text
bun install --frozen-lockfile
bunx --bun playwright install chromium
bun run test:e2e
```

`bun run test:e2e` is the completion gate. Its launcher builds the frontend and Rust binary,
creates deterministic temporary OpenSpec data, starts `openspec-doc` on fixed port 8792,
opens the embedded shell, and tears down processes and fixture files. It is not a Vite-only
smoke test.

The configured projects are deterministic desktop (1280×800) and narrow (390×844). The
suite must cover fixture data, theme persistence, Router navigation, reachability without
horizontal clipping, console/page errors, and failed asset/API requests. A missing font,
script, stylesheet, favicon, or API response is a failure, not an acceptable partial render.
Trace, screenshot, and video retain on failure; inspect those artifacts before guessing.

## Interactive inspection

Use the repository-pinned CLI through Bun when you need a manual check:

```text
bunx --bun playwright test --config playwright.config.ts --project=desktop --headed
bunx --bun playwright test --config playwright.config.ts --project=narrow --headed
```

Do not make a global Playwright install a hidden prerequisite. Do not substitute a browser
MCP or a Vite-only screenshot for the embedded lane. Record which viewport, theme, route,
fixture, and console/request result you inspected.

## Visual review checklist

- Light and dark themes preserve focus, contrast, and semantic state labels.
- Long identifiers wrap without pushing primary controls off-screen.
- Artifact content precedes comments; Reviewer and Agent labels remain explicit.
- No unexpected console/page/request failures occur.
- Reduced-motion and keyboard behavior remain usable where the change touches motion or
  interaction.
