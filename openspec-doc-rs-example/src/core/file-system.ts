import { readdir } from "node:fs/promises";
import path from "node:path";

export async function walkFiles(directory: string): Promise<string[]> {
  const entries = await safeReaddir(directory);
  const files: string[] = [];

  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await walkFiles(entryPath)));
    } else if (entry.isFile()) {
      files.push(entryPath);
    }
  }

  return files;
}

export async function safeReaddir(directory: string) {
  try {
    return await readdir(directory, { withFileTypes: true });
  } catch (error) {
    if (isMissingDirectory(error)) {
      return [];
    }
    throw error;
  }
}

function isMissingDirectory(error: unknown): boolean {
  return Boolean(error && typeof error === "object" && "code" in error && error.code === "ENOENT");
}
