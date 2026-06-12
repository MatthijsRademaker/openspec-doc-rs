import path from "node:path";

export function toRelative(rootPath: string, filePath: string): string {
  return path.relative(rootPath, filePath).split(path.sep).join("/");
}

export function compareRelativePath<T extends { relativePath: string }>(left: T, right: T): number {
  return left.relativePath.localeCompare(right.relativePath);
}
