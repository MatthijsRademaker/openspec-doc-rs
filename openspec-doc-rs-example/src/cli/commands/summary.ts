import { Command } from "commander";
import type { ProjectSummary } from "../../core/types.js";
import { loadProject, renderFatalError, type CommandOptions } from "./shared.js";

interface SummaryOptions extends CommandOptions {
  format?: "human" | "json";
}

export function createSummaryCommand(): Command {
  return new Command("summary")
    .description("Print a read-only overview of an OpenSpec project")
    .option("--root <path>", "target project root", process.cwd())
    .option("--format <format>", "output format: human or json", "human")
    .action(async (options: SummaryOptions) => {
      try {
        if (options.format !== "human" && options.format !== "json") {
          throw new Error(`Unsupported summary format: ${options.format}`);
        }

        const { analysis } = await loadProject(options);
        if (options.format === "json") {
          console.log(JSON.stringify(analysis.summary, null, 2));
          return;
        }

        console.log(renderHumanSummary(analysis.summary));
      } catch (error) {
        console.error(renderFatalError(error));
        process.exitCode = 1;
      }
    });
}

function renderHumanSummary(summary: ProjectSummary): string {
  return [
    "OpenSpec Summary",
    `Root: ${summary.rootPath}`,
    `Config: ${summary.configPath}`,
    "",
    `Current specs: ${summary.currentSpecs}`,
    `Active changes: ${summary.activeChanges}`,
    `Archived changes: ${summary.archivedChanges}`,
    `Requirements: ${summary.requirements}`,
    `Scenarios: ${summary.scenarios}`,
    `Tasks: ${summary.tasks.complete}/${summary.tasks.total} complete`,
    `Warnings: ${summary.warnings}`,
  ].join("\n");
}
