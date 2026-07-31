import { readFile } from "node:fs/promises";
import path from "node:path";
import { countTasks, emptyTaskCounts } from "./task-parser.js";
import type { DiscoveredChange, ParsedChange } from "./types.js";

export async function parseChange(
  change: DiscoveredChange,
  archived: boolean,
): Promise<ParsedChange> {
  const taskFile = change.files.find((file) => path.basename(file.path) === "tasks.md");
  const taskCounts = taskFile
    ? countTasks(await readFile(taskFile.path, "utf8"))
    : emptyTaskCounts();

  return {
    name: change.name,
    path: change.path,
    relativePath: change.relativePath,
    hasProposal: hasFile(change, "proposal.md"),
    hasDesign: hasFile(change, "design.md"),
    hasTasks: Boolean(taskFile),
    hasSpecDeltas: hasSpecDeltas(change),
    taskCounts,
    archived,
  };
}

function hasFile(change: DiscoveredChange, fileName: string): boolean {
  return change.files.some((file) => path.basename(file.path) === fileName);
}

function hasSpecDeltas(change: DiscoveredChange): boolean {
  return change.files.some((file) => file.relativePath.includes("/specs/"));
}
