import { describe, expect, it } from "vitest";
import { analyzeProject } from "../src/core/analyzer.js";
import { parseProject } from "../src/core/parser.js";
import { scanProject } from "../src/core/scanner.js";
import { fixturePath } from "./helpers.js";

async function parseFixture(name: string) {
  const scan = await scanProject(fixturePath(name));
  if (!scan.ok) {
    throw new Error(scan.fatalError.message);
  }
  return parseProject(scan.project);
}

describe("parser and analyzer", () => {
  it("extracts purpose, requirements, and scenarios", async () => {
    const parsed = await parseFixture("minimal-openspec");
    const spec = parsed.specs[0];

    expect(spec.purpose).toBe("Manage widgets for testing.");
    expect(spec.requirements).toHaveLength(1);
    expect(spec.requirements[0].title).toBe("Widget summary");
    expect(spec.requirements[0].scenarios).toHaveLength(1);
  });

  it("parses change artifacts and task counts", async () => {
    const parsed = await parseFixture("changes-openspec");
    const change = parsed.activeChanges[0];

    expect(change.hasProposal).toBe(true);
    expect(change.hasDesign).toBe(true);
    expect(change.hasTasks).toBe(true);
    expect(change.hasSpecDeltas).toBe(true);
    expect(change.taskCounts).toEqual({ total: 2, complete: 1, incomplete: 1 });
  });

  it("summarizes counts", async () => {
    const parsed = await parseFixture("changes-openspec");
    const analysis = analyzeProject(parsed);

    expect(analysis.summary.currentSpecs).toBe(1);
    expect(analysis.summary.activeChanges).toBe(1);
    expect(analysis.summary.archivedChanges).toBe(1);
    expect(analysis.summary.requirements).toBe(1);
    expect(analysis.summary.scenarios).toBe(1);
    expect(analysis.summary.tasks).toEqual({ total: 3, complete: 2, incomplete: 1 });
  });

  it("reports structural warnings", async () => {
    const parsed = await parseFixture("warnings-openspec");
    const analysis = analyzeProject(parsed);

    expect(analysis.warnings.map((warning) => warning.code)).toEqual([
      "missing-purpose",
      "requirement-missing-scenario",
      "malformed-scenario-heading",
      "missing-change-artifact",
      "missing-change-artifact",
    ]);
  });
});
