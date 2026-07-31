import { parseChange } from "./change-parser.js";
import { parseSpecFile } from "./spec-parser.js";
import type { ParsedProject, ScannedProject } from "./types.js";

export async function parseProject(scanned: ScannedProject): Promise<ParsedProject> {
  return {
    scanned,
    specs: await Promise.all(scanned.specFiles.map(parseSpecFile)),
    activeChanges: await Promise.all(
      scanned.activeChanges.map((change) => parseChange(change, false)),
    ),
    archivedChanges: await Promise.all(
      scanned.archivedChanges.map((change) => parseChange(change, true)),
    ),
  };
}

export { parseSpecFile, parseSpecMarkdown } from "./spec-parser.js";
export { parseChange } from "./change-parser.js";
export { extractSection } from "./markdown.js";
export { countTasks } from "./task-parser.js";
