---
pageType: home
title: openspec-doc
titleSuffix: Review agent work while it happens
hero:
  name: openspec-doc
  text: Review agent work while it happens
  tagline: A local browser dashboard for OpenSpec projects, plus an agent hook bridge.
  actions:
    - theme: brand
      text: Start with Quickstart
      link: /quickstart
    - theme: alt
      text: Understand the review loop
      link: /concepts/review-loop
    - theme: alt
      text: Project overview
      link: /overview
features:
  - title: See work as it happens
    details: Read an agent's exploration or proposal in a browser while it is still being written.
    icon: 👁️
  - title: Comment with context
    details: Anchor feedback to the exact markdown span, then keep it attached across edits and promotion.
    icon: 💬
  - title: Keep the loop closed
    details: Submit a phase verdict and deliver it back to the agent at its next turn boundary.
    icon: 🔁
---

## Review loop, without cloud infrastructure

openspec-doc keeps review state in plain files under `.openspec-doc/`. No database, daemon, or hosted service sits between the agent and reviewer.

Read the [project overview](/overview) for context, explore the [review loop](/concepts/review-loop) for the model, or follow the [Quickstart](/quickstart) to run one end to end.
