import { describe, expect, it } from "vitest";
import { createAnchorContext, resolveAnchorContext } from "../src/core/review-anchors.js";

describe("review anchor context and resolution", () => {
  const original = "# Proposal\n\n## Why\n\nAlpha before. Selected sentence. Omega after.\n";

  it("creates context and resolves exact matches", () => {
    const anchor = createAnchorContext({
      artifactPath: "openspec/changes/add-core/proposal.md",
      markdown: original,
      selectedText: "Selected sentence.",
    });

    expect(anchor.headingPath).toEqual(["Proposal", "Why"]);
    expect(anchor.beforeText).toContain("Alpha before.");
    expect(resolveAnchorContext(anchor, new Map([[anchor.artifactPath, original]]))).toEqual({
      state: "exact",
      offset: original.indexOf("Selected sentence."),
    });
  });

  it("resolves moved text as fuzzy when offsets change", () => {
    const anchor = createAnchorContext({
      artifactPath: "openspec/changes/add-core/proposal.md",
      markdown: original,
      selectedText: "Selected sentence.",
    });
    const moved =
      "# Proposal\n\n## Why\n\nNew introduction.\n\nAlpha before. Selected sentence. Omega after.\n";

    expect(resolveAnchorContext(anchor, new Map([[anchor.artifactPath, moved]])).state).toBe(
      "fuzzy",
    );
  });

  it("uses changed surrounding context for fuzzy resolution", () => {
    const anchor = createAnchorContext({
      artifactPath: "openspec/changes/add-core/proposal.md",
      markdown: original,
      selectedText: "Selected sentence.",
    });
    const changed = "# Proposal\n\n## Why\n\nAlpha before. Replacement text. Omega after.\n";

    const resolution = resolveAnchorContext(anchor, new Map([[anchor.artifactPath, changed]]));
    expect(resolution.state).toBe("fuzzy");
    expect(resolution.offset).toBeGreaterThanOrEqual(0);
  });

  it("marks missing text as orphaned when the artifact exists", () => {
    const anchor = createAnchorContext({
      artifactPath: "openspec/changes/add-core/proposal.md",
      markdown: original,
      selectedText: "Selected sentence.",
    });
    const rewritten = "Completely different proposal without useful context.";

    expect(resolveAnchorContext(anchor, new Map([[anchor.artifactPath, rewritten]])).state).toBe(
      "orphaned",
    );
  });

  it("marks anchors as missing when the artifact no longer exists", () => {
    const anchor = createAnchorContext({
      artifactPath: "openspec/changes/add-core/proposal.md",
      markdown: original,
      selectedText: "Selected sentence.",
    });

    expect(resolveAnchorContext(anchor, new Map()).state).toBe("missing");
  });
});
