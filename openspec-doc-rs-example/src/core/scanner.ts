import { stat } from "node:fs/promises";
import path from "node:path";
import {
  discoverActiveChanges,
  discoverArchivedChanges,
  discoverSpecFiles,
} from "./scanner-discovery.js";
import type { ScanResult } from "./types.js";

export async function scanProject(root = process.cwd()): Promise<ScanResult> {
  const rootPath = path.resolve(root);
  const openspecPath = path.join(rootPath, "openspec");
  const configPath = path.join(openspecPath, "config.yaml");

  if (!(await isFile(configPath))) {
    return missingConfig(configPath);
  }

  try {
    const specFiles = await discoverSpecFiles(rootPath, openspecPath);
    const activeChanges = await discoverActiveChanges(rootPath, openspecPath);
    const archivedChanges = await discoverArchivedChanges(rootPath, openspecPath);

    return {
      ok: true,
      project: {
        rootPath,
        openspecPath,
        configPath,
        specFiles,
        activeChanges,
        archivedChanges,
      },
    };
  } catch (error) {
    return {
      ok: false,
      fatalError: {
        code: "scan-failed",
        message: error instanceof Error ? error.message : "Failed to scan OpenSpec project",
      },
    };
  }
}

async function isFile(filePath: string): Promise<boolean> {
  try {
    return (await stat(filePath)).isFile();
  } catch {
    return false;
  }
}

function missingConfig(configPath: string): ScanResult {
  return {
    ok: false,
    fatalError: {
      code: "missing-config",
      path: configPath,
      message: `Missing OpenSpec config at ${configPath}. Run from a project root or pass --root <path>.`,
    },
  };
}
