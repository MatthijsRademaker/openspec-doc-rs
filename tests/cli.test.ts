import { execFile, spawn } from "node:child_process";
import { promisify } from "node:util";
import { describe, expect, it } from "vitest";
import { fixturePath } from "./helpers.js";

const execFileAsync = promisify(execFile);
const cliPath = "dist/cli/index.js";

async function runCli(args: string[]) {
  try {
    const result = await execFileAsync("node", [cliPath, ...args], { cwd: process.cwd() });
    return { code: 0, stdout: result.stdout, stderr: result.stderr };
  } catch (error) {
    const execError = error as { code?: number; stdout?: string; stderr?: string };
    return {
      code: execError.code ?? 1,
      stdout: execError.stdout ?? "",
      stderr: execError.stderr ?? "",
    };
  }
}

function waitForOutput(child: ReturnType<typeof spawn>, expected: string): Promise<string> {
  return new Promise((resolve, reject) => {
    let output = "";
    const timer = setTimeout(() => reject(new Error(`Timed out waiting for ${expected}`)), 5000);

    child.stdout?.on("data", (chunk: Buffer) => {
      output += chunk.toString("utf8");
      if (output.includes(expected)) {
        clearTimeout(timer);
        resolve(output);
      }
    });
    child.stderr?.on("data", (chunk: Buffer) => {
      output += chunk.toString("utf8");
    });
    child.on("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.on("exit", (code) => {
      if (!output.includes(expected)) {
        clearTimeout(timer);
        reject(new Error(`Process exited with ${code}: ${output}`));
      }
    });
  });
}

describe("CLI", () => {
  it("prints human summary", async () => {
    const result = await runCli(["summary", "--root", fixturePath("minimal-openspec")]);

    expect(result.code).toBe(0);
    expect(result.stdout).toContain("OpenSpec Summary");
    expect(result.stdout).toContain("Current specs: 1");
    expect(result.stdout).toContain("Warnings: 0");
  });

  it("prints JSON summary", async () => {
    const result = await runCli([
      "summary",
      "--root",
      fixturePath("minimal-openspec"),
      "--format",
      "json",
    ]);

    expect(result.code).toBe(0);
    const summary = JSON.parse(result.stdout) as { currentSpecs: number; warnings: number };
    expect(summary.currentSpecs).toBe(1);
    expect(summary.warnings).toBe(0);
  });

  it("returns zero for check without warnings", async () => {
    const result = await runCli(["check", "--root", fixturePath("minimal-openspec")]);

    expect(result.code).toBe(0);
    expect(result.stdout).toContain("no warnings found");
  });

  it("returns one for check with warnings", async () => {
    const result = await runCli(["check", "--root", fixturePath("warnings-openspec")]);

    expect(result.code).toBe(1);
    expect(result.stdout).toContain("warning(s)");
    expect(result.stdout).toContain("missing-purpose");
  });

  it("reports that serve starts with companion mode by default", async () => {
    const child = spawn(
      "node",
      [cliPath, "serve", "--root", fixturePath("changes-openspec"), "--port", "0", "--no-open"],
      { cwd: process.cwd(), stdio: ["ignore", "pipe", "pipe"] },
    );

    try {
      const output = await waitForOutput(child, "Browser auto-open disabled.");
      expect(output).toContain("Serving OpenSpec viewer at http://127.0.0.1:");
      expect(output).toContain("feedback writes are stored as sidecar records");
      expect(output).toContain("Browser auto-open disabled.");
    } finally {
      child.kill("SIGTERM");
    }
  });

  it("reports when serve review companion mode is disabled", async () => {
    const child = spawn(
      "node",
      [
        cliPath,
        "serve",
        "--root",
        fixturePath("minimal-openspec"),
        "--port",
        "0",
        "--no-review-companion",
        "--no-open",
      ],
      { cwd: process.cwd(), stdio: ["ignore", "pipe", "pipe"] },
    );

    try {
      const output = await waitForOutput(child, "Browser auto-open disabled.");
      expect(output).toContain("Serving OpenSpec viewer at http://127.0.0.1:");
      expect(output).toContain("browser feedback writes are rejected");
      expect(output).toContain("Browser auto-open disabled.");
    } finally {
      child.kill("SIGTERM");
    }
  });

  it("reports when serve watch mode is enabled", async () => {
    const child = spawn(
      "node",
      [
        cliPath,
        "serve",
        "--root",
        fixturePath("minimal-openspec"),
        "--port",
        "0",
        "--watch",
        "--no-open",
      ],
      { cwd: process.cwd(), stdio: ["ignore", "pipe", "pipe"] },
    );

    try {
      const output = await waitForOutput(child, "Watch mode enabled");
      expect(output).toContain("Serving OpenSpec viewer at http://127.0.0.1:");
      expect(output).toContain("OpenSpec changes will refresh the viewer");
    } finally {
      child.kill("SIGTERM");
    }
  });

  it("does not start serve when config is missing", async () => {
    const result = await runCli(["serve", "--root", fixturePath("does-not-exist"), "--port", "0"]);

    expect(result.code).toBe(1);
    expect(result.stderr).toContain("Missing OpenSpec config");
  });
});
