import { readFile } from "node:fs/promises";
import path from "node:path";
import { analyzeProject } from "../core/analyzer.js";
import { parseTaskChecklist, type TaskChecklistItem } from "../core/task-checklist.js";
import { parseProject } from "../core/parser.js";
import { scanProject } from "../core/scanner.js";
import {
  buildProjectLifecycleGuidance,
  inferChangeLifecycleGuidance,
  type LifecycleGuidance,
  type ProjectLifecycleGuidance,
} from "../core/lifecycle.js";
import { createReviewAgentState, type ReviewAgentState } from "../core/review-agent.js";
import type {
  AnalysisResult,
  ParsedChange,
  ParsedProject,
  ParsedSpec,
  ProjectSummary,
  ProjectWarning,
} from "../core/types.js";
import { CliFatalError } from "../cli/commands/shared.js";

export interface ViewerModel {
  rootPath: string;
  configPath: string;
  summary: ProjectSummary;
  specs: ViewerSpec[];
  activeChanges: ViewerChange[];
  archivedChanges: ViewerChange[];
  warnings: ProjectWarning[];
  lifecycle: ProjectLifecycleGuidance;
  reviewCompanionEnabled: boolean;
  reviewAgent: ReviewAgentState;
  watchEnabled: boolean;
  modelVersion: number;
  refreshError?: string;
}

export interface ViewerModelOptions {
  reviewCompanionEnabled?: boolean;
  reviewAgentEnabled?: boolean;
  reviewAgentUnavailableReason?: string;
  watchEnabled?: boolean;
  modelVersion?: number;
  refreshError?: string;
}

export interface ViewerSpec {
  name: string;
  path: string;
  relativePath: string;
  purpose?: string;
  requirements: ParsedSpec["requirements"];
  requirementCount: number;
  scenarioCount: number;
  warnings: ProjectWarning[];
  markdown: string;
}

export interface ViewerChange {
  name: string;
  path: string;
  relativePath: string;
  archived: boolean;
  hasProposal: boolean;
  hasDesign: boolean;
  hasTasks: boolean;
  hasSpecDeltas: boolean;
  taskCounts: ParsedChange["taskCounts"];
  warnings: ProjectWarning[];
  lifecycle: LifecycleGuidance;
  taskItems: TaskChecklistItem[];
  artifacts: ViewerArtifact[];
}

export interface ViewerArtifact {
  name: string;
  relativePath: string;
  markdown: string;
}

export async function buildViewerModel(
  root?: string,
  options: ViewerModelOptions = {},
): Promise<ViewerModel> {
  const scanResult = await scanProject(root);
  if (!scanResult.ok) {
    throw new CliFatalError(scanResult.fatalError);
  }

  const parsed = await parseProject(scanResult.project);
  const analysis = analyzeProject(parsed);

  return buildViewerModelFromProject(parsed, analysis, options);
}

export async function buildViewerModelFromProject(
  parsed: ParsedProject,
  analysis: AnalysisResult,
  options: ViewerModelOptions = {},
): Promise<ViewerModel> {
  const activeChanges = await Promise.all(
    parsed.activeChanges.map((change) => buildViewerChange(change, analysis.warnings)),
  );

  return {
    rootPath: parsed.scanned.rootPath,
    configPath: parsed.scanned.configPath,
    summary: analysis.summary,
    specs: await Promise.all(parsed.specs.map((spec) => buildViewerSpec(spec, analysis.warnings))),
    activeChanges,
    archivedChanges: await Promise.all(
      parsed.archivedChanges.map((change) => buildViewerChange(change, analysis.warnings)),
    ),
    warnings: analysis.warnings,
    lifecycle: buildProjectLifecycleGuidance(activeChanges),
    reviewCompanionEnabled: options.reviewCompanionEnabled ?? true,
    reviewAgent: createReviewAgentState({
      enabled: options.reviewAgentEnabled ?? true,
      unavailableReason: options.reviewAgentUnavailableReason,
    }),
    watchEnabled: options.watchEnabled ?? false,
    modelVersion: options.modelVersion ?? 1,
    refreshError: options.refreshError,
  };
}

async function buildViewerSpec(spec: ParsedSpec, warnings: ProjectWarning[]): Promise<ViewerSpec> {
  return {
    name: spec.name,
    path: spec.path,
    relativePath: spec.relativePath,
    purpose: spec.purpose,
    requirements: spec.requirements,
    requirementCount: spec.requirements.length,
    scenarioCount: spec.requirements.reduce(
      (count, requirement) => count + requirement.scenarios.length,
      0,
    ),
    warnings: warningsForPath(warnings, spec.relativePath),
    markdown: await readFile(spec.path, "utf8"),
  };
}

async function buildViewerChange(
  change: ParsedChange,
  warnings: ProjectWarning[],
): Promise<ViewerChange> {
  const artifacts = await readArtifacts(change);
  const taskSource = artifacts.find((artifact) => artifact.name === "Tasks")?.markdown ?? "";

  return {
    name: change.name,
    path: change.path,
    relativePath: change.relativePath,
    archived: change.archived,
    hasProposal: change.hasProposal,
    hasDesign: change.hasDesign,
    hasTasks: change.hasTasks,
    hasSpecDeltas: change.hasSpecDeltas,
    taskCounts: change.taskCounts,
    warnings: warningsForPath(warnings, change.relativePath),
    lifecycle: inferChangeLifecycleGuidance({
      ...change,
      warnings: warningsForPath(warnings, change.relativePath),
    }),
    taskItems: parseTaskChecklist(taskSource),
    artifacts,
  };
}

async function readArtifacts(change: ParsedChange): Promise<ViewerArtifact[]> {
  const entries = [
    { name: "Proposal", filePath: path.join(change.path, "proposal.md") },
    { name: "Design", filePath: path.join(change.path, "design.md") },
    { name: "Tasks", filePath: path.join(change.path, "tasks.md") },
  ];

  const artifacts: ViewerArtifact[] = [];
  for (const entry of entries) {
    const markdown = await readOptionalFile(entry.filePath);
    if (markdown !== undefined) {
      artifacts.push({
        name: entry.name,
        relativePath: toOpenSpecRelative(change, entry.filePath),
        markdown,
      });
    }
  }

  for (const specDelta of await findSpecDeltaFiles(change.path)) {
    artifacts.push({
      name: `Spec Delta: ${path.basename(path.dirname(specDelta))}`,
      relativePath: toOpenSpecRelative(change, specDelta),
      markdown: await readFile(specDelta, "utf8"),
    });
  }

  return artifacts;
}

async function findSpecDeltaFiles(changePath: string): Promise<string[]> {
  const specsPath = path.join(changePath, "specs");
  return walkMdFiles(specsPath);
}

async function walkMdFiles(directory: string): Promise<string[]> {
  const { walkFiles } = await import("../core/file-system.js");
  return (await walkFiles(directory))
    .filter((filePath) => path.extname(filePath) === ".md")
    .sort((left, right) => left.localeCompare(right));
}

async function readOptionalFile(filePath: string): Promise<string | undefined> {
  try {
    return await readFile(filePath, "utf8");
  } catch (error) {
    if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") {
      return undefined;
    }
    throw error;
  }
}

function warningsForPath(warnings: ProjectWarning[], relativePath: string): ProjectWarning[] {
  return warnings.filter(
    (warning) => warning.path === relativePath || warning.path.startsWith(`${relativePath}/`),
  );
}

function toOpenSpecRelative(change: ParsedChange, filePath: string): string {
  return `${change.relativePath}/${path.relative(change.path, filePath).split(path.sep).join("/")}`;
}
