import { Command } from "commander";
import { loadProject, renderFatalError, type CommandOptions } from "./shared.js";

export function createCheckCommand(): Command {
  return new Command("check")
    .description("Report OpenSpec structural warnings")
    .option("--root <path>", "target project root", process.cwd())
    .action(async (options: CommandOptions) => {
      try {
        const { analysis } = await loadProject(options);

        if (analysis.warnings.length === 0) {
          console.log("OpenSpec check passed: no warnings found.");
          return;
        }

        console.log(`OpenSpec check found ${analysis.warnings.length} warning(s):`);
        for (const warning of analysis.warnings) {
          console.log(`- [${warning.code}] ${warning.path}: ${warning.message}`);
        }
        process.exitCode = 1;
      } catch (error) {
        console.error(renderFatalError(error));
        process.exitCode = 1;
      }
    });
}
