import type {
  AnalysisResult,
  ParsedChange,
  ParsedProject,
  ProjectWarning,
  TaskCounts,
} from "./types.js";

export function analyzeProject(project: ParsedProject): AnalysisResult {
  const warnings = collectWarnings(project);
  const tasks = sumTasks([...project.activeChanges, ...project.archivedChanges]);

  return {
    summary: {
      rootPath: project.scanned.rootPath,
      configPath: project.scanned.configPath,
      currentSpecs: project.specs.length,
      activeChanges: project.activeChanges.length,
      archivedChanges: project.archivedChanges.length,
      requirements: project.specs.reduce((count, spec) => count + spec.requirements.length, 0),
      scenarios: project.specs.reduce(
        (count, spec) =>
          count + spec.requirements.reduce((specCount, req) => specCount + req.scenarios.length, 0),
        0,
      ),
      tasks,
      warnings: warnings.length,
    },
    warnings,
  };
}

function collectWarnings(project: ParsedProject): ProjectWarning[] {
  const warnings: ProjectWarning[] = [];

  for (const spec of project.specs) {
    if (!spec.purpose) {
      warnings.push({
        code: "missing-purpose",
        path: spec.relativePath,
        message: `Spec ${spec.relativePath} is missing a ## Purpose section.`,
      });
    }

    for (const requirement of spec.requirements) {
      if (requirement.scenarios.length === 0) {
        warnings.push({
          code: "requirement-missing-scenario",
          path: spec.relativePath,
          message: `Requirement "${requirement.title}" in ${spec.relativePath} has no scenarios.`,
        });
      }
    }

    for (const heading of spec.malformedScenarioHeadings) {
      warnings.push({
        code: "malformed-scenario-heading",
        path: spec.relativePath,
        message: `Malformed scenario heading at ${spec.relativePath}:${heading.line}: ${heading.text}`,
      });
    }
  }

  for (const change of project.activeChanges) {
    addMissingArtifactWarnings(warnings, change);
  }

  return warnings;
}

function addMissingArtifactWarnings(warnings: ProjectWarning[], change: ParsedChange): void {
  const expectedArtifacts = [
    { present: change.hasProposal, fileName: "proposal.md" },
    { present: change.hasDesign, fileName: "design.md" },
    { present: change.hasTasks, fileName: "tasks.md" },
  ];

  for (const artifact of expectedArtifacts) {
    if (!artifact.present) {
      warnings.push({
        code: "missing-change-artifact",
        path: change.relativePath,
        message: `Active change ${change.name} is missing ${artifact.fileName}.`,
      });
    }
  }
}

function sumTasks(changes: ParsedChange[]): TaskCounts {
  return changes.reduce(
    (counts, change) => ({
      total: counts.total + change.taskCounts.total,
      complete: counts.complete + change.taskCounts.complete,
      incomplete: counts.incomplete + change.taskCounts.incomplete,
    }),
    { total: 0, complete: 0, incomplete: 0 },
  );
}
