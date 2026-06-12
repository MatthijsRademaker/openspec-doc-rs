import { cp } from "node:fs/promises";
import path from "node:path";

export function fixturePath(name: string): string {
  return path.resolve("tests", "fixtures", name);
}

export async function copyFixture(name: string, destination: string): Promise<void> {
  await cp(fixturePath(name), destination, { recursive: true });
}
