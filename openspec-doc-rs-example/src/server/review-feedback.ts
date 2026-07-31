import { randomUUID } from "node:crypto";
import { appendFile, mkdir, readFile, rename, writeFile } from "node:fs/promises";
import path from "node:path";
import { createAnchorContext, resolveAnchorContext } from "../core/review-anchors.js";
import type {
  AnchoredReviewComment,
  ReviewAnchorContext,
  ReviewCommentEdit,
  ReviewCommentStatus,
  ResolvedReviewComment,
  ReviewStatusUpdate,
  ReviewThreadReply,
} from "../core/review-companion.js";
import type { ViewerChange, ViewerModel } from "../viewer/model.js";

export interface ReviewFeedbackInput {
  body?: unknown;
  artifactPath?: unknown;
  artifactKind?: unknown;
  selectedText?: unknown;
  headingPath?: unknown;
  beforeText?: unknown;
  afterText?: unknown;
  startOffset?: unknown;
  endOffset?: unknown;
  category?: unknown;
  severity?: unknown;
  author?: unknown;
}

export interface ReviewThreadReplyInput {
  body?: unknown;
  author?: unknown;
}

export interface ReviewStatusUpdateInput {
  status?: unknown;
  author?: unknown;
}

export interface ReviewCommentEditInput {
  body?: unknown;
  author?: unknown;
}

export type ReviewFeedbackRecord = AnchoredReviewComment & {
  timestamp: string;
  selectedText?: string;
  category?: string;
  severity?: string;
};

interface ReviewSidecarCommentEvent {
  type: "comment";
  comment: ReviewFeedbackRecord;
}

interface ReviewSidecarReplyEvent {
  type: "reply";
  reply: ReviewThreadReply;
}

interface ReviewSidecarStatusEvent {
  type: "status";
  status: ReviewStatusUpdate;
}

interface ReviewSidecarEditEvent {
  type: "edit";
  edit: ReviewCommentEdit;
}

export type ReviewSidecarEvent =
  | ReviewSidecarCommentEvent
  | ReviewSidecarReplyEvent
  | ReviewSidecarStatusEvent
  | ReviewSidecarEditEvent;

export type FeedbackValidationResult =
  | {
      ok: true;
      change: ViewerChange;
      filePath: string;
      artifactPaths: Set<string>;
    }
  | {
      ok: false;
      status: number;
      message: string;
    };

const SIDE_CAR_DIRECTORY = path.join(".pi", "openspec-review", "comments");

export function feedbackSidecarDirectory(rootPath: string): string {
  return path.join(rootPath, SIDE_CAR_DIRECTORY);
}

export function feedbackSidecarPath(rootPath: string, changeName: string): string {
  return path.join(feedbackSidecarDirectory(rootPath), `${changeName}.jsonl`);
}

export function validateFeedbackChange(
  model: ViewerModel,
  changeName: string | null,
): FeedbackValidationResult {
  if (!changeName) {
    return { ok: false, status: 400, message: "Missing change name." };
  }

  if (!isSafeChangeName(changeName)) {
    return { ok: false, status: 400, message: "Invalid change name." };
  }

  const change = model.activeChanges.find((item) => item.name === changeName);
  if (!change) {
    const archived = model.archivedChanges.some((item) => item.name === changeName);
    return {
      ok: false,
      status: 404,
      message: archived
        ? `Cannot write feedback for archived change: ${changeName}`
        : `Active change not found: ${changeName}`,
    };
  }

  const sidecarPath = feedbackSidecarPath(model.rootPath, changeName);
  const sidecarRoot = `${path.resolve(feedbackSidecarDirectory(model.rootPath))}${path.sep}`;
  const resolvedPath = path.resolve(sidecarPath);
  if (!resolvedPath.startsWith(sidecarRoot)) {
    return { ok: false, status: 400, message: "Feedback path escapes sidecar storage." };
  }

  return {
    ok: true,
    change,
    filePath: resolvedPath,
    artifactPaths: new Set(change.artifacts.map((artifact) => artifact.relativePath)),
  };
}

export function validateAnchoredFeedbackInput(
  input: ReviewFeedbackInput,
  validation: Extract<FeedbackValidationResult, { ok: true }>,
): void {
  const artifactPath = normalizeRequiredString(input.artifactPath, "artifactPath");
  const selectedText = normalizeRequiredString(input.selectedText, "selectedText");

  if (!validation.artifactPaths.has(artifactPath)) {
    throw new Error(
      `Feedback artifactPath is not part of active change ${validation.change.name}.`,
    );
  }

  const artifact = validation.change.artifacts.find((item) => item.relativePath === artifactPath);
  if (!artifact?.markdown.includes(selectedText)) {
    throw new Error("Feedback selectedText must exist in the referenced artifact.");
  }
}

export async function listFeedbackRecords(
  rootPath: string,
  changeName: string,
): Promise<ReviewFeedbackRecord[]> {
  return [...(await readReviewThreads(rootPath, changeName)).values()].map(
    ({ comment }) => comment,
  );
}

export async function listResolvedReviewComments(
  rootPath: string,
  change: ViewerChange,
): Promise<ResolvedReviewComment[]> {
  const threads = await readReviewThreads(rootPath, change.name);
  const artifacts = new Map(
    change.artifacts.map((artifact) => [artifact.relativePath, artifact.markdown]),
  );

  return [...threads.values()].map(({ comment, replies, statusHistory, editHistory }) => {
    const resolution = resolveAnchorContext(comment.anchor, artifacts);
    return {
      ...comment,
      replies,
      statusHistory,
      editHistory,
      anchorState: resolution.state,
      resolvedOffset: resolution.offset,
    };
  });
}

async function readReviewThreads(
  rootPath: string,
  changeName: string,
): Promise<
  Map<
    string,
    {
      comment: ReviewFeedbackRecord;
      replies: ReviewThreadReply[];
      statusHistory: ReviewStatusUpdate[];
      editHistory: ReviewCommentEdit[];
    }
  >
> {
  const events = await readReviewSidecarEvents(rootPath, changeName);
  const threads = new Map<
    string,
    {
      comment: ReviewFeedbackRecord;
      replies: ReviewThreadReply[];
      statusHistory: ReviewStatusUpdate[];
      editHistory: ReviewCommentEdit[];
    }
  >();

  for (const event of events) {
    if (event.type === "comment") {
      threads.set(event.comment.id, {
        comment: event.comment,
        replies: [],
        statusHistory: [],
        editHistory: [],
      });
    }
    if (event.type === "reply") {
      threads.get(event.reply.commentId)?.replies.push(event.reply);
    }
    if (event.type === "status") {
      const thread = threads.get(event.status.commentId);
      if (thread) {
        thread.statusHistory.push(event.status);
        thread.comment.status = event.status.status;
        thread.comment.updatedAt = event.status.createdAt;
        thread.comment.timestamp = event.status.createdAt;
      }
    }
    if (event.type === "edit") {
      const thread = threads.get(event.edit.commentId);
      if (thread) {
        thread.editHistory.push(event.edit);
        thread.comment.body = event.edit.body;
        thread.comment.updatedAt = event.edit.createdAt;
        thread.comment.timestamp = event.edit.createdAt;
      }
    }
  }

  return threads;
}

export async function readReviewSidecarEvents(
  rootPath: string,
  changeName: string,
): Promise<ReviewSidecarEvent[]> {
  const filePath = feedbackSidecarPath(rootPath, changeName);
  let content: string;
  try {
    content = await readFile(filePath, "utf8");
  } catch (error) {
    if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") {
      return [];
    }
    throw error;
  }

  return content
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .flatMap((line) => {
      try {
        return [normalizeSidecarEvent(JSON.parse(line))];
      } catch {
        return [];
      }
    });
}

export async function appendFeedbackRecord(
  rootPath: string,
  changeName: string,
  input: ReviewFeedbackInput,
  validation?: Extract<FeedbackValidationResult, { ok: true }>,
): Promise<ReviewFeedbackRecord> {
  const body = normalizeRequiredString(input.body, "body");
  if (validation) {
    validateAnchoredFeedbackInput(input, validation);
  }
  const artifactPath = normalizeRequiredString(input.artifactPath, "artifactPath");
  const selectedText = normalizeRequiredString(input.selectedText, "selectedText");
  const now = new Date().toISOString();
  const anchor: ReviewAnchorContext =
    validation && input.beforeText === undefined && input.afterText === undefined
      ? createAnchorContext({
          artifactPath,
          artifactKind: normalizeOptionalString(input.artifactKind),
          markdown:
            validation.change.artifacts.find((artifact) => artifact.relativePath === artifactPath)
              ?.markdown ?? "",
          selectedText,
        })
      : {
          artifactPath,
          artifactKind: normalizeOptionalString(input.artifactKind),
          selectedText,
          headingPath: normalizeHeadingPath(input.headingPath),
          beforeText: normalizeOptionalString(input.beforeText),
          afterText: normalizeOptionalString(input.afterText),
          startOffset: normalizeOptionalNumber(input.startOffset),
          endOffset: normalizeOptionalNumber(input.endOffset),
        };
  const record: ReviewFeedbackRecord = {
    id: randomUUID(),
    timestamp: now,
    changeName,
    body,
    artifactPath,
    artifactKind: anchor.artifactKind,
    anchor,
    selectedText,
    category: normalizeOptionalString(input.category),
    severity: normalizeOptionalString(input.severity),
    author: normalizeOptionalString(input.author),
    source: "browser",
    status: "open",
    createdAt: now,
    updatedAt: now,
  };

  await appendReviewSidecarEvent(rootPath, changeName, { type: "comment", comment: record });
  return record;
}

export async function appendThreadReply(
  rootPath: string,
  changeName: string,
  commentId: string,
  input: ReviewThreadReplyInput,
): Promise<ReviewThreadReply> {
  const reply: ReviewThreadReply = {
    id: randomUUID(),
    commentId,
    changeName,
    body: normalizeRequiredString(input.body, "body"),
    author: normalizeOptionalString(input.author),
    source: "browser",
    createdAt: new Date().toISOString(),
  };
  await appendReviewSidecarEvent(rootPath, changeName, { type: "reply", reply });
  return reply;
}

export async function appendStatusUpdate(
  rootPath: string,
  changeName: string,
  commentId: string,
  input: ReviewStatusUpdateInput,
): Promise<ReviewStatusUpdate> {
  const status = normalizeStatus(input.status);
  const update: ReviewStatusUpdate = {
    id: randomUUID(),
    commentId,
    changeName,
    status,
    author: normalizeOptionalString(input.author),
    source: "browser",
    createdAt: new Date().toISOString(),
  };
  await appendReviewSidecarEvent(rootPath, changeName, { type: "status", status: update });
  return update;
}

export async function appendCommentEdit(
  rootPath: string,
  changeName: string,
  commentId: string,
  input: ReviewCommentEditInput,
): Promise<ReviewCommentEdit> {
  if (!(await readReviewThreads(rootPath, changeName)).has(commentId)) {
    throw new Error("Feedback comment not found.");
  }

  const edit: ReviewCommentEdit = {
    id: randomUUID(),
    commentId,
    changeName,
    body: normalizeRequiredString(input.body, "body"),
    author: normalizeOptionalString(input.author),
    source: "browser",
    createdAt: new Date().toISOString(),
  };
  await appendReviewSidecarEvent(rootPath, changeName, { type: "edit", edit });
  return edit;
}

export async function deleteFeedbackThread(
  rootPath: string,
  changeName: string,
  commentId: string,
): Promise<{ deleted: true; commentId: string }> {
  if (!(await readReviewThreads(rootPath, changeName)).has(commentId)) {
    throw new Error("Feedback comment not found.");
  }

  const events = await readReviewSidecarEvents(rootPath, changeName);
  const retainedEvents = events.filter((event) => {
    if (event.type === "comment") return event.comment.id !== commentId;
    if (event.type === "reply") return event.reply.commentId !== commentId;
    if (event.type === "status") return event.status.commentId !== commentId;
    if (event.type === "edit") return event.edit.commentId !== commentId;
    return true;
  });
  await rewriteReviewSidecarEvents(rootPath, changeName, retainedEvents);
  return { deleted: true, commentId };
}

async function appendReviewSidecarEvent(
  rootPath: string,
  changeName: string,
  event: ReviewSidecarEvent,
): Promise<void> {
  const directory = feedbackSidecarDirectory(rootPath);
  await mkdir(directory, { recursive: true });
  await appendFile(feedbackSidecarPath(rootPath, changeName), `${JSON.stringify(event)}\n`, "utf8");
}

async function rewriteReviewSidecarEvents(
  rootPath: string,
  changeName: string,
  events: ReviewSidecarEvent[],
): Promise<void> {
  const directory = feedbackSidecarDirectory(rootPath);
  await mkdir(directory, { recursive: true });
  const filePath = feedbackSidecarPath(rootPath, changeName);
  const temporaryPath = `${filePath}.${randomUUID()}.tmp`;
  const content = events.map((event) => JSON.stringify(event)).join("\n");
  await writeFile(temporaryPath, content ? `${content}\n` : "", "utf8");
  await rename(temporaryPath, filePath);
}

export function isSafeChangeName(changeName: string): boolean {
  return /^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(changeName) && !changeName.includes("..");
}

function normalizeSidecarEvent(value: unknown): ReviewSidecarEvent {
  if (isSidecarEvent(value)) {
    return value;
  }

  const legacy = value as ReviewFeedbackRecord;
  return { type: "comment", comment: legacy };
}

function isSidecarEvent(value: unknown): value is ReviewSidecarEvent {
  return (
    typeof value === "object" &&
    value !== null &&
    "type" in value &&
    (value.type === "comment" ||
      value.type === "reply" ||
      value.type === "status" ||
      value.type === "edit")
  );
}

function normalizeRequiredString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`Feedback ${field} is required.`);
  }
  return value.trim();
}

function normalizeOptionalString(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() !== "" ? value.trim() : undefined;
}

function normalizeOptionalNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function normalizeHeadingPath(value: unknown): string[] | undefined {
  if (!Array.isArray(value)) {
    return undefined;
  }
  const headings = value.filter(
    (item): item is string => typeof item === "string" && item.trim() !== "",
  );
  return headings.length > 0 ? headings.map((item) => item.trim()) : undefined;
}

function normalizeStatus(value: unknown): ReviewCommentStatus {
  if (value === "open" || value === "addressed" || value === "resolved") {
    return value;
  }
  throw new Error("Feedback status must be open, addressed, or resolved.");
}
