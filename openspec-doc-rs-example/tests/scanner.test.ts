import { describe, expect, it } from "vitest";
import { scanProject } from "../src/core/scanner.js";
import { fixturePath } from "./helpers.js";

describe("scanProject", () => {
  it("discovers config and current specs", async () => {
    const result = await scanProject(fixturePath("minimal-openspec"));

    expect(result.ok).toBe(true);
    if (!result.ok) {
      return;
    }

    expect(result.project.configPath).toContain("openspec/config.yaml");
    expect(result.project.specFiles).toHaveLength(1);
    expect(result.project.specFiles[0].relativePath).toBe("openspec/specs/widgets/spec.md");
  });

  it("discovers active and archived changes separately", async () => {
    const result = await scanProject(fixturePath("changes-openspec"));

    expect(result.ok).toBe(true);
    if (!result.ok) {
      return;
    }

    expect(result.project.activeChanges.map((change) => change.name)).toEqual(["add-core"]);
    expect(result.project.archivedChanges.map((change) => change.name)).toEqual([
      "2026-01-01-old-change",
    ]);
  });

  it("returns an actionable fatal error when config is missing", async () => {
    const result = await scanProject(fixturePath("does-not-exist"));

    expect(result.ok).toBe(false);
    if (result.ok) {
      return;
    }

    expect(result.fatalError.code).toBe("missing-config");
    expect(result.fatalError.message).toContain("openspec/config.yaml");
    expect(result.fatalError.message).toContain("--root <path>");
  });
});
