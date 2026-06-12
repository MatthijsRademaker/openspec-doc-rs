#!/usr/bin/env node
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { Command } from "commander";
import { createCheckCommand } from "./commands/check.js";
import { createServeCommand } from "./commands/serve.js";
import { createSummaryCommand } from "./commands/summary.js";

const packageJson = JSON.parse(
  readFileSync(
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../package.json"),
    "utf8",
  ),
) as { version: string };

export function createProgram(): Command {
  return new Command()
    .name("openspec-doc")
    .description("OpenSpec documentation and review CLI")
    .version(packageJson.version)
    .addCommand(createSummaryCommand())
    .addCommand(createCheckCommand())
    .addCommand(createServeCommand());
}

createProgram()
  .parseAsync(process.argv)
  .catch((error: unknown) => {
    console.error(error instanceof Error ? error.message : "Unknown CLI failure");
    process.exitCode = 1;
  });
