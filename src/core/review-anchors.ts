import type { ReviewAnchorContext, ReviewAnchorState } from "./review-companion.js";

export interface CreateAnchorContextInput {
  artifactPath: string;
  artifactKind?: string;
  markdown: string;
  selectedText: string;
  contextLength?: number;
}

export interface AnchorResolution {
  state: ReviewAnchorState;
  offset?: number;
}

const DEFAULT_CONTEXT_LENGTH = 80;

export function createAnchorContext(input: CreateAnchorContextInput): ReviewAnchorContext {
  const selectedText = input.selectedText.trim();
  if (selectedText === "") {
    throw new Error("Selected text is required to create an anchor context.");
  }

  const startOffset = input.markdown.indexOf(selectedText);
  if (startOffset < 0) {
    throw new Error("Selected text was not found in the artifact markdown.");
  }

  const endOffset = startOffset + selectedText.length;
  const contextLength = input.contextLength ?? DEFAULT_CONTEXT_LENGTH;

  return {
    artifactPath: input.artifactPath,
    artifactKind: input.artifactKind,
    selectedText,
    headingPath: headingPathBefore(input.markdown, startOffset),
    beforeText: input.markdown.slice(Math.max(0, startOffset - contextLength), startOffset),
    afterText: input.markdown.slice(endOffset, endOffset + contextLength),
    startOffset,
    endOffset,
  };
}

export function resolveAnchorContext(
  anchor: ReviewAnchorContext,
  artifacts: Map<string, string>,
): AnchorResolution {
  const markdown = artifacts.get(anchor.artifactPath);
  if (markdown === undefined) {
    return { state: "missing" };
  }

  const exactOffset = findExactOffset(anchor, markdown);
  if (exactOffset !== undefined) {
    return { state: "exact", offset: exactOffset };
  }

  const fuzzyOffset = findFuzzyOffset(anchor, markdown);
  if (fuzzyOffset !== undefined) {
    return { state: "fuzzy", offset: fuzzyOffset };
  }

  return { state: "orphaned" };
}

function findExactOffset(anchor: ReviewAnchorContext, markdown: string): number | undefined {
  if (
    anchor.startOffset !== undefined &&
    markdown.slice(anchor.startOffset, anchor.startOffset + anchor.selectedText.length) ===
      anchor.selectedText
  ) {
    return anchor.startOffset;
  }

  return undefined;
}

function findFuzzyOffset(anchor: ReviewAnchorContext, markdown: string): number | undefined {
  const headingOffset = findHeadingOffset(anchor.headingPath, markdown);
  const searchStart = headingOffset ?? 0;
  const selectedOffset = markdown.indexOf(anchor.selectedText, searchStart);
  if (selectedOffset >= 0) {
    return selectedOffset;
  }

  const before = anchor.beforeText?.trim();
  if (before) {
    const beforeOffset = markdown.indexOf(before);
    if (beforeOffset >= 0) {
      return beforeOffset + before.length;
    }
  }

  const after = anchor.afterText?.trim();
  if (after) {
    const afterOffset = markdown.indexOf(after);
    if (afterOffset >= 0) {
      return afterOffset;
    }
  }

  return headingOffset;
}

function headingPathBefore(markdown: string, offset: number): string[] | undefined {
  const path: string[] = [];
  const linesBeforeSelection = markdown.slice(0, offset).split(/\r?\n/);

  for (const line of linesBeforeSelection) {
    const match = /^(#{1,6})\s+(.+)$/.exec(line.trim());
    if (!match) {
      continue;
    }
    const level = match[1].length;
    path.length = level - 1;
    path[level - 1] = match[2].trim();
  }

  return path.length > 0 ? path.filter(Boolean) : undefined;
}

function findHeadingOffset(
  headingPath: string[] | undefined,
  markdown: string,
): number | undefined {
  const heading = headingPath?.at(-1);
  if (!heading) {
    return undefined;
  }

  const pattern = new RegExp(`^#{1,6}\\s+${escapeRegExp(heading)}\\s*$`, "m");
  const match = pattern.exec(markdown);
  return match?.index;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
