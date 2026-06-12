import { randomUUID } from "node:crypto";
import { appendFile, mkdir, readFile } from "node:fs/promises";
import path from "node:path";
import {
  createReviewAgentBundle,
  renderReviewAgentBundlePrompt,
  type ReviewAgentBundle,
} from "../core/review-agent-bundle.js";
import { isSafeChangeName, listResolvedReviewComments } from "./review-feedback.js";
import type { ViewerChange, ViewerModel } from "../viewer/model.js";

export interface ReviewAgentSessionRequestInput {
  reviewerInstruction?: unknown;
  selectedCommentIds?: unknown;
  includeAllOpenFeedback?: unknown;
}

export type ReviewAgentValidationResult =
  | {
      ok: true;
      change: ViewerChange;
      reviewerInstruction: string;
      selectedCommentIds: string[];
      includeAllOpenFeedback: boolean;
    }
  | { ok: false; status: number; message: string };

export interface ReviewAgentSessionRecord {
  id: string;
  changeName: string;
  status: "created" | "running" | "completed" | "failed" | "aborted";
  createdAt: string;
  updatedAt: string;
  bundle: ReviewAgentBundle;
  responseText?: string;
  error?: string;
}

type ReviewAgentSessionEvent =
  | { type: "session_created"; session: ReviewAgentSessionRecord }
  | { type: "response_completed"; sessionId: string; responseText: string; createdAt: string }
  | { type: "session_failed"; sessionId: string; error: string; createdAt: string }
  | { type: "session_aborted"; sessionId: string; createdAt: string };

const REVIEW_AGENT_DIRECTORY = path.join(".pi", "openspec-review", "sessions");

export function reviewAgentSessionDirectory(rootPath: string): string {
  return path.join(rootPath, REVIEW_AGENT_DIRECTORY);
}

export function reviewAgentSessionPath(rootPath: string, changeName: string): string {
  return path.join(reviewAgentSessionDirectory(rootPath), `${changeName}.jsonl`);
}

export async function createBrowserReviewAgentSession(
  model: ViewerModel,
  changeName: string | null,
  input: ReviewAgentSessionRequestInput,
): Promise<ReviewAgentSessionRecord> {
  const validation = await validateReviewAgentSessionRequest(model, changeName, input);
  if (!validation.ok) {
    throw new ReviewAgentRequestError(validation.status, validation.message);
  }

  const feedback = await listResolvedReviewComments(model.rootPath, validation.change);
  const bundle = createReviewAgentBundle({
    id: randomUUID(),
    changeName: validation.change.name,
    reviewerInstruction: validation.reviewerInstruction,
    selectedCommentIds: validation.selectedCommentIds,
    includeAllOpenFeedback: validation.includeAllOpenFeedback,
    artifacts: validation.change.artifacts,
    warnings: validation.change.warnings,
    feedback,
  });
  const now = new Date().toISOString();
  const record: ReviewAgentSessionRecord = {
    id: randomUUID(),
    changeName: validation.change.name,
    status: "created",
    createdAt: now,
    updatedAt: now,
    bundle,
  };

  await appendReviewAgentSessionEvent(model.rootPath, validation.change.name, {
    type: "session_created",
    session: record,
  });
  return record;
}

export async function validateReviewAgentSessionRequest(
  model: ViewerModel,
  changeName: string | null,
  input: ReviewAgentSessionRequestInput,
): Promise<ReviewAgentValidationResult> {
  if (model.reviewAgent.mode !== "enabled") {
    return {
      ok: false,
      status: 403,
      message:
        model.reviewAgent.reason ??
        "Browser Pi review analysis is not enabled for this serve session.",
    };
  }

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
        ? `Cannot start Pi review analysis for archived change: ${changeName}`
        : `Active change not found: ${changeName}`,
    };
  }

  if (change.artifacts.length === 0) {
    return { ok: false, status: 400, message: "Active change has no reviewable artifacts." };
  }

  const reviewerInstruction = normalizeInstruction(input.reviewerInstruction);
  const selectedCommentIds = normalizeSelectedCommentIds(input.selectedCommentIds);
  const includeAllOpenFeedback = input.includeAllOpenFeedback !== false;
  const selected = new Set(selectedCommentIds);
  if (selected.size > 0) {
    const feedback = await listResolvedReviewComments(model.rootPath, change);
    const knownCommentIds = new Set(feedback.map((thread) => thread.id));
    for (const commentId of selected) {
      if (!knownCommentIds.has(commentId)) {
        return {
          ok: false,
          status: 400,
          message: `Selected feedback comment is not part of active change ${change.name}: ${commentId}`,
        };
      }
    }
  }

  return { ok: true, change, reviewerInstruction, selectedCommentIds, includeAllOpenFeedback };
}

export function reviewAgentUnavailableError(error: unknown): { error: string; actionable: true } {
  const message = error instanceof Error ? error.message : "Pi review analysis could not start.";
  return {
    error: `${message} Check Pi SDK setup, credentials, and model availability, or restart with --no-review-agent.`,
    actionable: true,
  };
}

export async function listRecentReviewAgentSessions(
  rootPath: string,
  changeName: string,
): Promise<ReviewAgentSessionRecord[]> {
  const events = await readReviewAgentSessionEvents(rootPath, changeName);
  const sessions = new Map<string, ReviewAgentSessionRecord>();
  for (const event of events) {
    if (event.type === "session_created") {
      sessions.set(event.session.id, event.session);
    } else {
      const session = sessions.get(event.sessionId);
      if (!session) continue;
      session.updatedAt = event.createdAt;
      if (event.type === "response_completed") {
        session.status = "completed";
        session.responseText = event.responseText;
      }
      if (event.type === "session_failed") {
        session.status = "failed";
        session.error = event.error;
      }
      if (event.type === "session_aborted") {
        session.status = "aborted";
      }
    }
  }
  return [...sessions.values()]
    .filter((session) => session.status !== "running")
    .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt));
}

export async function appendReviewAgentResponseCompleted(
  rootPath: string,
  changeName: string,
  sessionId: string,
  responseText: string,
): Promise<void> {
  await appendReviewAgentSessionEvent(rootPath, changeName, {
    type: "response_completed",
    sessionId,
    responseText,
    createdAt: new Date().toISOString(),
  });
}

export async function appendReviewAgentSessionFailed(
  rootPath: string,
  changeName: string,
  sessionId: string,
  error: string,
): Promise<void> {
  await appendReviewAgentSessionEvent(rootPath, changeName, {
    type: "session_failed",
    sessionId,
    error,
    createdAt: new Date().toISOString(),
  });
}

export async function appendReviewAgentSessionAborted(
  rootPath: string,
  changeName: string,
  sessionId: string,
): Promise<void> {
  await appendReviewAgentSessionEvent(rootPath, changeName, {
    type: "session_aborted",
    sessionId,
    createdAt: new Date().toISOString(),
  });
}

export function buildReviewAgentPrompt(record: ReviewAgentSessionRecord): string {
  return renderReviewAgentBundlePrompt(record.bundle);
}

export class ReviewAgentRequestError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
  }
}

async function appendReviewAgentSessionEvent(
  rootPath: string,
  changeName: string,
  event: ReviewAgentSessionEvent,
): Promise<void> {
  await mkdir(reviewAgentSessionDirectory(rootPath), { recursive: true });
  await appendFile(
    reviewAgentSessionPath(rootPath, changeName),
    `${JSON.stringify(event)}\n`,
    "utf8",
  );
}

async function readReviewAgentSessionEvents(
  rootPath: string,
  changeName: string,
): Promise<ReviewAgentSessionEvent[]> {
  let content: string;
  try {
    content = await readFile(reviewAgentSessionPath(rootPath, changeName), "utf8");
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
        return [JSON.parse(line) as ReviewAgentSessionEvent];
      } catch {
        return [];
      }
    });
}

function normalizeInstruction(value: unknown): string {
  if (typeof value !== "string" || value.trim() === "") {
    return "Analyze the active OpenSpec change and included feedback.";
  }
  return value.trim();
}

function normalizeSelectedCommentIds(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .filter((item): item is string => typeof item === "string" && item.trim() !== "")
    .map((item) => item.trim());
}
