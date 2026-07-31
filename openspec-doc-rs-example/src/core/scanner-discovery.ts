import path from "node:path";
import { safeReaddir, walkFiles } from "./file-system.js";
import { compareRelativePath, toRelative } from "./path-utils.js";
import type { DiscoveredChange, DiscoveredChangeFile, DiscoveredSpecFile } from "./types.js";

export async function discoverSpecFiles(
  rootPath: string,
  openspecPath: string,
): Promise<DiscoveredSpecFile[]> {
  const specsPath = path.join(openspecPath, "specs");
  const files = await walkFiles(specsPath);

  return files
    .filter((filePath) => path.basename(filePath) === "spec.md")
    .map((filePath) => ({
      path: filePath,
      relativePath: toRelative(rootPath, filePath),
      name: path.basename(path.dirname(filePath)),
    }))
    .sort(compareRelativePath);
}

export async function discoverActiveChanges(
  rootPath: string,
  openspecPath: string,
): Promise<DiscoveredChange[]> {
  const changesPath = path.join(openspecPath, "changes");
  const entries = await safeReaddir(changesPath);
  const changes: DiscoveredChange[] = [];

  for (const entry of entries) {
    if (!entry.isDirectory() || entry.name === "archive") {
      continue;
    }

    const changePath = path.join(changesPath, entry.name);
    changes.push({
      name: entry.name,
      path: changePath,
      relativePath: toRelative(rootPath, changePath),
      files: await discoverChangeFiles(rootPath, changePath),
    });
  }

  return changes.sort(compareRelativePath);
}

export async function discoverArchivedChanges(
  rootPath: string,
  openspecPath: string,
): Promise<DiscoveredChange[]> {
  const archivePath = path.join(openspecPath, "changes", "archive");
  const files = await walkFiles(archivePath);
  const changeMap = new Map<string, DiscoveredChange>();

  for (const filePath of files) {
    const changeName = getArchivedChangeName(archivePath, filePath);
    if (!changeName) {
      continue;
    }

    const existing = getOrCreateArchivedChange(changeMap, rootPath, archivePath, changeName);
    existing.files.push({ path: filePath, relativePath: toRelative(rootPath, filePath) });
  }

  return [...changeMap.values()]
    .map((change) => ({ ...change, files: change.files.sort(compareRelativePath) }))
    .sort(compareRelativePath);
}

async function discoverChangeFiles(
  rootPath: string,
  changePath: string,
): Promise<DiscoveredChangeFile[]> {
  const files = await walkFiles(changePath);
  return files
    .map((filePath) => ({ path: filePath, relativePath: toRelative(rootPath, filePath) }))
    .sort(compareRelativePath);
}

function getArchivedChangeName(archivePath: string, filePath: string): string | undefined {
  const relativeToArchive = path.relative(archivePath, filePath);
  const [changeName, ...nestedParts] = relativeToArchive.split(path.sep);

  return changeName && nestedParts.length > 0 ? changeName : undefined;
}

function getOrCreateArchivedChange(
  changeMap: Map<string, DiscoveredChange>,
  rootPath: string,
  archivePath: string,
  changeName: string,
): DiscoveredChange {
  const existing = changeMap.get(changeName);
  if (existing) {
    return existing;
  }

  const changePath = path.join(archivePath, changeName);
  const change = {
    name: changeName,
    path: changePath,
    relativePath: toRelative(rootPath, changePath),
    files: [],
  };
  changeMap.set(changeName, change);

  return change;
}
