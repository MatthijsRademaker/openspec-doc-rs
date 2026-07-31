import { describe, expect, it } from "vitest";
import {
  buildProjectLifecycleGuidance,
  inferChangeLifecycleGuidance,
  type ChangeLifecycleInput,
} from "../src/core/lifecycle.js";

const baseChange: ChangeLifecycleInput = {
  name: "add-core",
  archived: false,
  hasProposal: true,
  hasDesign: true,
  hasTasks: true,
  hasSpecDeltas: true,
  taskCounts: { total: 2, complete: 0, incomplete: 2 },
  warnings: [],
};

describe("lifecycle guidance", () => {
  it("shows ordered project phases and explore guidance when there are no active changes", () => {
    const guidance = buildProjectLifecycleGuidance([]);

    expect(guidance.phase.id).toBe("explore");
    expect(guidance.phases.map((phase) => phase.id)).toEqual([
      "explore",
      "propose",
      "review",
      "apply",
      "archive",
    ]);
    expect(guidance.summary).toContain("No active OpenSpec changes");
    expect(guidance.actions.map((action) => action.command)).toContain("openspec list");
    expect(guidance.safetyNotes.join(" ")).toContain("allowlisted commands");
  });

  it("builds an active-change lifecycle queue", () => {
    const guidance = buildProjectLifecycleGuidance([
      { ...baseChange, taskCounts: { total: 2, complete: 1, incomplete: 1 } },
    ]);

    expect(guidance.activeQueue).toHaveLength(1);
    expect(guidance.activeQueue[0].name).toBe("add-core");
    expect(guidance.activeQueue[0].guidance.phase.id).toBe("apply");
    expect(guidance.actions.some((action) => action.href === "/changes")).toBe(true);
  });

  it("infers propose when required change artifacts are missing", () => {
    const guidance = inferChangeLifecycleGuidance({
      ...baseChange,
      hasDesign: false,
      hasTasks: false,
      hasSpecDeltas: false,
      taskCounts: { total: 0, complete: 0, incomplete: 0 },
    });

    expect(guidance.phase.id).toBe("propose");
    expect(guidance.reasons).toEqual([
      "Missing design.md.",
      "Missing tasks.md.",
      "Missing spec delta artifact.",
    ]);
    expect(guidance.actions.map((action) => action.command)).toContain(
      "openspec validate add-core",
    );
  });

  it("infers review when artifacts are complete before implementation starts", () => {
    const guidance = inferChangeLifecycleGuidance(baseChange);

    expect(guidance.phase.id).toBe("review");
    expect(guidance.reasons.join(" ")).toContain("implementation work in progress yet");
    expect(guidance.actions.map((action) => action.href)).toContain("/changes/add-core/review");
  });

  it("infers apply when implementation tasks are incomplete after work has started", () => {
    const guidance = inferChangeLifecycleGuidance({
      ...baseChange,
      taskCounts: { total: 3, complete: 1, incomplete: 2 },
    });

    expect(guidance.phase.id).toBe("apply");
    expect(guidance.reasons).toContain("2 task(s) remain incomplete.");
    expect(guidance.actions.map((action) => action.command)).toContain("/opsx-apply add-core");
    expect(guidance.safetyNotes.join(" ")).toContain(
      "Run validation before marking implementation complete",
    );
  });

  it("infers archive when tasks are complete and no active warnings apply", () => {
    const guidance = inferChangeLifecycleGuidance({
      ...baseChange,
      taskCounts: { total: 2, complete: 2, incomplete: 0 },
    });

    expect(guidance.phase.id).toBe("archive");
    expect(guidance.reasons).toContain("No analyzer warnings apply to this active change.");
    expect(guidance.actions.map((action) => action.command)).toEqual([
      "openspec validate add-core",
      "openspec archive add-core",
    ]);
  });

  it("keeps completed changes with warnings in review", () => {
    const guidance = inferChangeLifecycleGuidance({
      ...baseChange,
      taskCounts: { total: 2, complete: 2, incomplete: 0 },
      warnings: [
        {
          code: "missing-change-artifact",
          path: "openspec/changes/add-core",
          message: "Active change add-core is missing design.md.",
        },
      ],
    });

    expect(guidance.phase.id).toBe("review");
    expect(guidance.reasons.join(" ")).toContain("warning");
  });

  it("reports archived changes as complete without active actions", () => {
    const guidance = inferChangeLifecycleGuidance({ ...baseChange, archived: true });

    expect(guidance.phase.id).toBe("archive");
    expect(guidance.summary).toContain("active lifecycle is complete");
    expect(guidance.actions).toEqual([]);
  });
});
