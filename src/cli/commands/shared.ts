import { analyzeProject } from "../../core/analyzer.js";
import { parseProject } from "../../core/parser.js";
import { scanProject } from "../../core/scanner.js";
import type { AnalysisResult, FatalError } from "../../core/types.js";

export interface CommandOptions {
  root?: string;
}

export interface LoadedProject {
  analysis: AnalysisResult;
}

export async function loadProject(options: CommandOptions): Promise<LoadedProject> {
  const scanResult = await scanProject(options.root);

  if (!scanResult.ok) {
    throw new CliFatalError(scanResult.fatalError);
  }

  const parsed = await parseProject(scanResult.project);
  return { analysis: analyzeProject(parsed) };
}

export class CliFatalError extends Error {
  readonly fatalError: FatalError;

  constructor(fatalError: FatalError) {
    super(fatalError.message);
    this.name = "CliFatalError";
    this.fatalError = fatalError;
  }
}

export function renderFatalError(error: unknown): string {
  if (error instanceof CliFatalError) {
    return `Error: ${error.fatalError.message}`;
  }

  return error instanceof Error ? `Error: ${error.message}` : "Error: Unknown failure";
}
