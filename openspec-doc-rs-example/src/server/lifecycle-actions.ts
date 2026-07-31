import { spawn } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { TASK_CHECKLIST_PATTERN } from "../core/task-checklist.js";
import type { ViewerChange, ViewerModel } from "../viewer/model.js";

export type LifecycleActionKind =
  | "explore"
  | "new-change-handoff"
  | "proposal-handoff"
  | "apply-handoff"
  | "validate"
  | "archive"
  | "mark-task";

export interface LifecycleActionRequest {
  kind?: unknown;
  changeName?: unknown;
  line?: unknown;
  checked?: unknown;
}

export interface LifecycleActionResult {
  ok: boolean;
  kind: LifecycleActionKind;
  summary: string;
  output?: string;
  command?: string[];
  exitCode?: number | null;
  refreshed?: boolean;
}

export class LifecycleActionError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

export async function handleLifecycleAction(
  model: ViewerModel,
  input: LifecycleActionRequest,
): Promise<LifecycleActionResult> {
  const kind = normalizeActionKind(input.kind);
  const change = actionRequiresChange(kind)
    ? validateActiveChange(model, input.changeName)
    : undefined;

  switch (kind) {
    case "explore":
      return handoffResult(kind, "Explore an OpenSpec idea", [
        "Ask Pi: explore this OpenSpec change idea for the current project.",
        "Clarify the problem, users, constraints, and acceptance criteria before creating a proposal.",
      ]);
    case "new-change-handoff":
      return handoffResult(kind, "Create a new OpenSpec change", [
        "Ask Pi: propose a new OpenSpec change for this idea.",
        "Or run: openspec new change <change-name>",
      ]);
    case "proposal-handoff":
      return handoffResult(kind, `Refine proposal artifacts for ${change?.name}`, [
        `Ask Pi: continue proposal work for ${change?.name}.`,
        `Run validation afterwards: openspec validate ${change?.name}`,
      ]);
    case "apply-handoff":
      return handoffResult(kind, `Apply implementation tasks for ${change?.name}`, [
        `Ask Pi or the agent: /opsx-apply ${change?.name}`,
        `Run validation before archive: openspec validate ${change?.name}`,
      ]);
    case "validate":
      return runOpenSpecCommand(model.rootPath, kind, ["openspec", "validate", change?.name ?? ""]);
    case "archive":
      return runOpenSpecCommand(model.rootPath, kind, [
        "openspec",
        "archive",
        change?.name ?? "",
        "--yes",
      ]);
    case "mark-task":
      return markTask(model, change, input);
  }
}

export async function markTask(
  model: ViewerModel,
  change: ViewerChange | undefined,
  input: LifecycleActionRequest,
): Promise<LifecycleActionResult> {
  if (!change) {
    throw new LifecycleActionError("Active change is required for task mutation.", 400);
  }
  if (typeof input.line !== "number" || !Number.isInteger(input.line)) {
    throw new LifecycleActionError("Task line must be an integer.", 400);
  }
  if (typeof input.checked !== "boolean") {
    throw new LifecycleActionError("Task checked state must be boolean.", 400);
  }
  const line = input.line;
  const checked = input.checked;

  const tasksArtifact = change.artifacts.find((artifact) => artifact.name === "Tasks");
  if (!tasksArtifact) {
    throw new LifecycleActionError(`Active change ${change.name} has no tasks.md artifact.`, 404);
  }

  const filePath = resolveOpenSpecRelativePath(model.rootPath, tasksArtifact.relativePath);
  const tasksRoot = path.resolve(change.path);
  if (!filePath.startsWith(`${tasksRoot}${path.sep}`)) {
    throw new LifecycleActionError("Task artifact path escapes the active change directory.", 400);
  }

  const markdown = await readFile(filePath, "utf8");
  const lines = markdown.split(/\r?\n/);
  const lineIndex = line - 1;
  const lineText = lines[lineIndex];
  if (lineText === undefined) {
    throw new LifecycleActionError("Task line is no longer present.", 409);
  }

  const match = lineText.match(TASK_CHECKLIST_PATTERN);
  if (!match) {
    throw new LifecycleActionError("Requested task line is not a checklist item.", 409);
  }

  const currentChecked = match[2].toLowerCase() === "x";
  if (currentChecked === checked) {
    return {
      ok: true,
      kind: "mark-task",
      summary: `Task line ${line} is already ${checked ? "complete" : "incomplete"}.`,
      refreshed: false,
    };
  }

  lines[lineIndex] = `${match[1]}- [${checked ? "x" : " "}]${match[3]}`;
  await writeFile(filePath, lines.join("\n"), "utf8");

  return {
    ok: true,
    kind: "mark-task",
    summary: `Marked task line ${line} ${checked ? "complete" : "incomplete"}.`,
    refreshed: true,
  };
}

function normalizeActionKind(kind: unknown): LifecycleActionKind {
  if (
    kind === "explore" ||
    kind === "new-change-handoff" ||
    kind === "proposal-handoff" ||
    kind === "apply-handoff" ||
    kind === "validate" ||
    kind === "archive" ||
    kind === "mark-task"
  ) {
    return kind;
  }
  throw new LifecycleActionError("Unknown lifecycle action.", 400);
}

function actionRequiresChange(kind: LifecycleActionKind): boolean {
  return (
    kind === "proposal-handoff" ||
    kind === "apply-handoff" ||
    kind === "validate" ||
    kind === "archive" ||
    kind === "mark-task"
  );
}

function validateActiveChange(model: ViewerModel, changeName: unknown): ViewerChange {
  if (typeof changeName !== "string" || changeName.trim() === "") {
    throw new LifecycleActionError("Active change name is required.", 400);
  }
  if (!/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/.test(changeName)) {
    throw new LifecycleActionError("Invalid change name.", 400);
  }

  const change = model.activeChanges.find((item) => item.name === changeName);
  if (!change) {
    const archived = model.archivedChanges.some((item) => item.name === changeName);
    throw new LifecycleActionError(
      archived
        ? `Lifecycle actions cannot mutate archived change: ${changeName}`
        : `Active change not found: ${changeName}`,
      404,
    );
  }
  return change;
}

async function runOpenSpecCommand(
  cwd: string,
  kind: "validate" | "archive",
  command: [string, ...string[]],
): Promise<LifecycleActionResult> {
  const result = await spawnCommand(command, cwd);
  return {
    ok: result.exitCode === 0,
    kind,
    summary:
      result.exitCode === 0
        ? `${command.join(" ")} completed successfully.`
        : `${command.join(" ")} failed with exit code ${result.exitCode}.`,
    output: [result.stdout, result.stderr].filter(Boolean).join("\n"),
    command,
    exitCode: result.exitCode,
    refreshed: kind === "archive" && result.exitCode === 0,
  };
}

function spawnCommand(
  command: [string, ...string[]],
  cwd: string,
): Promise<{ exitCode: number | null; stdout: string; stderr: string }> {
  return new Promise((resolve, reject) => {
    const child = spawn(command[0], command.slice(1), { cwd, shell: false });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.on("error", reject);
    child.on("close", (exitCode) =>
      resolve({ exitCode, stdout: stdout.trim(), stderr: stderr.trim() }),
    );
  });
}

function handoffResult(
  kind: LifecycleActionKind,
  summary: string,
  lines: string[],
): LifecycleActionResult {
  return {
    ok: true,
    kind,
    summary,
    output: lines.join("\n"),
    refreshed: false,
  };
}

function resolveOpenSpecRelativePath(rootPath: string, relativePath: string): string {
  const root = path.resolve(rootPath);
  const resolved = path.resolve(root, relativePath);
  if (!(resolved === root || resolved.startsWith(`${root}${path.sep}`))) {
    throw new LifecycleActionError("Resolved path escapes target project root.", 400);
  }
  return resolved;
}
