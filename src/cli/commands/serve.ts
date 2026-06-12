import { Command } from "commander";
import { openBrowser } from "../open-browser.js";
import { startViewerServer } from "../../server/dev-server.js";
import { renderFatalError, type CommandOptions } from "./shared.js";

interface ServeCommandOptions extends CommandOptions {
  host?: string;
  port?: string;
  reviewCompanion: boolean;
  reviewAgent: boolean;
  open: boolean;
  watch?: boolean;
}

export function createServeCommand(): Command {
  return new Command("serve")
    .description("Start a local browser viewer for an OpenSpec project")
    .option("--root <path>", "target project root", process.cwd())
    .option("--host <host>", "host interface to bind", "127.0.0.1")
    .option("--port <port>", "port to bind (0 selects an available port)", "0")
    .option("--no-review-companion", "disable review companion mode and sidecar feedback writes")
    .option("--no-review-agent", "disable browser Pi review analysis")
    .option("--no-open", "do not open the viewer in the default browser")
    .option("--watch", "watch OpenSpec files and refresh viewer pages after changes", false)
    .action(async (options: ServeCommandOptions) => {
      try {
        const port = parsePort(options.port);
        const server = await startViewerServer({
          root: options.root,
          host: options.host,
          port,
          reviewCompanion: options.reviewCompanion,
          reviewAgent: options.reviewAgent,
          watch: options.watch,
        });

        console.log(`Serving OpenSpec viewer at ${server.url}`);
        if (options.reviewCompanion) {
          console.log(
            "Review companion mode enabled by default; feedback writes are stored as sidecar records.",
          );
        } else {
          console.log("Review companion mode disabled; browser feedback writes are rejected.");
        }
        console.log(server.reviewAgentStatus);
        if (options.watch) {
          console.log("Watch mode enabled; OpenSpec changes will refresh the viewer.");
        }
        if (options.open) {
          const browser = await openBrowser(server.url);
          if (browser.opened) {
            console.log(`Opened browser at ${server.url}`);
          } else {
            console.log(`Unable to open browser automatically: ${browser.error}`);
          }
        } else {
          console.log("Browser auto-open disabled.");
        }
        console.log("Press Ctrl+C to stop.");

        const stop = async () => {
          await server.close();
          process.exit(0);
        };
        process.once("SIGINT", () => {
          void stop();
        });
        process.once("SIGTERM", () => {
          void stop();
        });
      } catch (error) {
        console.error(renderFatalError(error));
        process.exitCode = 1;
      }
    });
}

function parsePort(value: string | undefined): number {
  const port = Number(value ?? "0");
  if (!Number.isInteger(port) || port < 0 || port > 65_535) {
    throw new Error(`Unsupported port: ${value}`);
  }
  return port;
}
