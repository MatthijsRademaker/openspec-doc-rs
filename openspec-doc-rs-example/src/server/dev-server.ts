import { existsSync, watch, type FSWatcher } from "node:fs";
import http, { type IncomingMessage, type ServerResponse } from "node:http";
import path from "node:path";
import { reviewAgentStatusMessage } from "../core/review-agent.js";
import {
  handleLifecycleAction,
  LifecycleActionError,
  type LifecycleActionRequest,
} from "./lifecycle-actions.js";
import {
  SdkPiReviewBridge,
  type BrowserReviewAgentEvent,
  type PiReviewBridge,
  type PiReviewSessionHandle,
} from "./pi-bridge.js";
import {
  appendReviewAgentResponseCompleted,
  appendReviewAgentSessionAborted,
  appendReviewAgentSessionFailed,
  createBrowserReviewAgentSession,
  listRecentReviewAgentSessions,
  reviewAgentUnavailableError,
  ReviewAgentRequestError,
  validateReviewAgentSessionRequest,
  type ReviewAgentSessionRequestInput,
} from "./review-agent.js";
import {
  appendCommentEdit,
  appendFeedbackRecord,
  appendStatusUpdate,
  deleteFeedbackThread,
  appendThreadReply,
  feedbackSidecarDirectory,
  listResolvedReviewComments,
  validateFeedbackChange,
  type ReviewCommentEditInput,
  type ReviewFeedbackInput,
  type ReviewStatusUpdateInput,
  type ReviewThreadReplyInput,
} from "./review-feedback.js";
import { buildViewerModel, type ViewerModel } from "../viewer/model.js";
import { renderRoute } from "../viewer/render.js";

export interface ServeOptions {
  root?: string;
  host?: string;
  port?: number;
  reviewCompanion?: boolean;
  reviewAgent?: boolean;
  reviewAgentBridge?: PiReviewBridge;
  watch?: boolean;
}

export interface StartedServer {
  url: string;
  host: string;
  port: number;
  reviewAgentStatus: string;
  close: () => Promise<void>;
}

interface ServerState {
  model: ViewerModel;
  version: number;
  refreshError?: string;
  reviewAgentBridge: PiReviewBridge;
  reviewAgentSessions: Map<string, PiReviewSessionHandle>;
  refreshModel: () => Promise<void>;
}

const WATCH_DEBOUNCE_MS = 100;

export async function startViewerServer(options: ServeOptions = {}): Promise<StartedServer> {
  const host = options.host ?? "127.0.0.1";
  const port = options.port ?? 0;
  const reviewCompanionEnabled = options.reviewCompanion ?? true;
  const reviewAgentEnabled = reviewCompanionEnabled && (options.reviewAgent ?? true);
  const watchEnabled = options.watch ?? false;
  const state: ServerState = {
    version: 1,
    model: await buildViewerModel(options.root, {
      reviewCompanionEnabled,
      reviewAgentEnabled,
      watchEnabled,
      modelVersion: 1,
    }),
    reviewAgentBridge: options.reviewAgentBridge ?? new SdkPiReviewBridge(),
    reviewAgentSessions: new Map(),
    refreshModel: async () => {},
  };
  state.refreshModel = async () => {
    const nextVersion = state.version + 1;
    state.model = await buildViewerModel(options.root, {
      reviewCompanionEnabled,
      reviewAgentEnabled,
      watchEnabled,
      modelVersion: nextVersion,
    });
    state.version = nextVersion;
    state.refreshError = undefined;
  };
  const watcher = watchEnabled
    ? createModelWatcher(state, options.root, reviewCompanionEnabled, reviewAgentEnabled)
    : undefined;
  const server = http.createServer((request, response) => {
    void handleRequest(
      state,
      request,
      response,
      reviewCompanionEnabled,
      watchEnabled,
      watcher?.scheduleRefresh,
    );
  });

  await new Promise<void>((resolve, reject) => {
    const onError = (error: Error) => {
      server.off("listening", onListening);
      reject(error);
    };
    const onListening = () => {
      server.off("error", onError);
      resolve();
    };

    server.once("error", onError);
    server.once("listening", onListening);
    server.listen(port, host);
  });

  const address = server.address();
  const resolvedPort = typeof address === "object" && address ? address.port : port;
  const url = `http://${host}:${resolvedPort}`;

  return {
    url,
    host,
    port: resolvedPort,
    reviewAgentStatus: reviewAgentStatusMessage(state.model.reviewAgent),
    close: () =>
      new Promise<void>((resolve, reject) => {
        watcher?.close();
        server.close((error) => (error ? reject(error) : resolve()));
      }),
  };
}

async function handleRequest(
  state: ServerState,
  request: IncomingMessage,
  response: ServerResponse,
  reviewCompanionEnabled: boolean,
  watchEnabled: boolean,
  scheduleRefresh?: () => void,
): Promise<void> {
  const requestUrl = new URL(request.url ?? "/", "http://localhost");

  if (requestUrl.pathname === "/api/model-version") {
    if (!watchEnabled) {
      writeJson(response, 404, { error: "Watch mode is not enabled." });
      return;
    }
    writeJson(response, 200, {
      version: state.version,
      refreshError: state.refreshError,
    });
    return;
  }

  const model = state.model;
  if (requestUrl.pathname.startsWith("/api/review-agent/")) {
    await handleReviewAgentRequest(state, request, response, requestUrl);
    return;
  }

  if (requestUrl.pathname === "/api/lifecycle-actions") {
    await handleLifecycleActionRequest(state, request, response);
    return;
  }

  if (requestUrl.pathname.startsWith("/api/review-feedback/")) {
    await handleFeedbackRequest(
      model,
      request,
      response,
      requestUrl,
      reviewCompanionEnabled,
      scheduleRefresh,
    );
    return;
  }

  const page = renderRoute(model, requestUrl.pathname);

  response.writeHead(page.status, {
    "content-type": "text/html; charset=utf-8",
    "cache-control": "no-store",
  });
  response.end(page.html);
}

async function handleReviewAgentRequest(
  state: ServerState,
  request: IncomingMessage,
  response: ServerResponse,
  requestUrl: URL,
): Promise<void> {
  const action = decodeReviewAgentAction(requestUrl.pathname);

  if (!action) {
    writeJson(response, 404, { error: "Review agent endpoint not found." });
    return;
  }

  if (action.kind === "change-sessions") {
    if (request.method === "GET") {
      const validation = validateFeedbackChange(state.model, action.changeName);
      if (!validation.ok) {
        writeJson(response, validation.status, { error: validation.message });
        return;
      }
      const sessions = await listRecentReviewAgentSessions(
        state.model.rootPath,
        validation.change.name,
      );
      writeJson(response, 200, { sessions });
      return;
    }

    if (request.method === "POST") {
      try {
        const input = (await readJsonBody(request)) as ReviewAgentSessionRequestInput;
        const record = await createBrowserReviewAgentSession(state.model, action.changeName, input);
        const handle = await state.reviewAgentBridge.start(record);
        state.reviewAgentSessions.set(record.id, handle);
        handle.subscribe((event) => {
          void persistReviewAgentEvent(state.model.rootPath, record.changeName, event);
          if (
            event.type === "session_finished" ||
            event.type === "session_aborted" ||
            event.type === "error"
          ) {
            state.reviewAgentSessions.delete(record.id);
          }
        });
        writeJson(response, 201, {
          session: record,
          eventsUrl: `/api/review-agent/sessions/${encodeURIComponent(record.id)}/events`,
        });
      } catch (error) {
        if (error instanceof ReviewAgentRequestError) {
          writeJson(response, error.status, { error: error.message });
          return;
        }
        writeJson(response, 503, reviewAgentUnavailableError(error));
      }
      return;
    }
  }

  if (action.kind === "session-events" && request.method === "GET") {
    const handle = state.reviewAgentSessions.get(action.sessionId);
    if (!handle) {
      writeJson(response, 404, {
        error: "Review agent session is not running or cannot be streamed.",
      });
      return;
    }
    streamReviewAgentEvents(response, handle);
    return;
  }

  if (action.kind === "session-message" && request.method === "POST") {
    const handle = state.reviewAgentSessions.get(action.sessionId);
    if (!handle) {
      writeJson(response, 404, { error: "Review agent session is not running." });
      return;
    }
    const input = (await readJsonBody(request)) as { message?: unknown };
    if (typeof input.message !== "string" || input.message.trim() === "") {
      writeJson(response, 400, { error: "Follow-up message is required." });
      return;
    }
    await handle.followUp(input.message.trim());
    writeJson(response, 202, { accepted: true });
    return;
  }

  if (action.kind === "session-abort" && request.method === "POST") {
    const handle = state.reviewAgentSessions.get(action.sessionId);
    if (!handle) {
      writeJson(response, 404, { error: "Review agent session is not running." });
      return;
    }
    await handle.abort();
    state.reviewAgentSessions.delete(action.sessionId);
    writeJson(response, 202, { aborted: true });
    return;
  }

  if (action.kind === "session-retry" && request.method === "POST") {
    const input = (await readJsonBody(request)) as ReviewAgentSessionRequestInput & {
      changeName?: unknown;
    };
    if (typeof input.changeName !== "string") {
      writeJson(response, 400, { error: "Retry requires an active changeName." });
      return;
    }
    const validation = await validateReviewAgentSessionRequest(
      state.model,
      input.changeName,
      input,
    );
    if (!validation.ok) {
      writeJson(response, validation.status, { error: validation.message });
      return;
    }
    try {
      const record = await createBrowserReviewAgentSession(state.model, input.changeName, input);
      const handle = await state.reviewAgentBridge.start(record);
      state.reviewAgentSessions.set(record.id, handle);
      handle.subscribe((event) => {
        void persistReviewAgentEvent(state.model.rootPath, record.changeName, event);
      });
      writeJson(response, 201, { session: record, retryOf: action.sessionId });
    } catch (error) {
      writeJson(response, 503, reviewAgentUnavailableError(error));
    }
    return;
  }

  writeJson(response, 405, { error: "Method not allowed." });
}

async function handleLifecycleActionRequest(
  state: ServerState,
  request: IncomingMessage,
  response: ServerResponse,
): Promise<void> {
  if (request.method !== "POST") {
    writeJson(response, 405, { error: "Method not allowed." });
    return;
  }

  try {
    const input = (await readJsonBody(request)) as LifecycleActionRequest;
    const result = await handleLifecycleAction(state.model, input);
    if (result.refreshed) {
      await state.refreshModel();
    }
    writeJson(response, result.ok ? 200 : 422, { result, modelVersion: state.version });
  } catch (error) {
    if (error instanceof LifecycleActionError) {
      writeJson(response, error.status, { error: error.message });
      return;
    }
    writeJson(response, 500, {
      error: error instanceof Error ? error.message : "Lifecycle action failed.",
    });
  }
}

async function handleFeedbackRequest(
  model: ViewerModel,
  request: IncomingMessage,
  response: ServerResponse,
  requestUrl: URL,
  reviewCompanionEnabled: boolean,
  scheduleRefresh?: () => void,
): Promise<void> {
  if (!reviewCompanionEnabled) {
    writeJson(response, 403, { error: "Review companion mode is not enabled." });
    return;
  }

  const action = decodeFeedbackAction(requestUrl.pathname);
  const changeName = action?.changeName ?? null;
  const validation = validateFeedbackChange(model, changeName);
  if (!validation.ok) {
    writeJson(response, validation.status, { error: validation.message });
    return;
  }

  if (request.method === "GET") {
    const feedback = await listResolvedReviewComments(model.rootPath, validation.change);
    writeJson(response, 200, { feedback });
    return;
  }

  if (request.method === "DELETE") {
    try {
      if (!action || action.kind === "comment") {
        writeJson(response, 405, { error: "Method not allowed." });
        return;
      }
      const deleted = await deleteFeedbackThread(
        model.rootPath,
        validation.change.name,
        action.commentId,
      );
      scheduleRefresh?.();
      writeJson(response, 200, deleted);
    } catch (error) {
      writeJson(response, 404, {
        error: error instanceof Error ? error.message : "Feedback comment not found.",
      });
    }
    return;
  }

  if (request.method === "POST") {
    try {
      const input = await readJsonBody(request);
      if (action?.kind === "reply") {
        const reply = await appendThreadReply(
          model.rootPath,
          validation.change.name,
          action.commentId,
          input as ReviewThreadReplyInput,
        );
        scheduleRefresh?.();
        writeJson(response, 201, { reply });
        return;
      }
      if (action?.kind === "status") {
        const status = await appendStatusUpdate(
          model.rootPath,
          validation.change.name,
          action.commentId,
          input as ReviewStatusUpdateInput,
        );
        scheduleRefresh?.();
        writeJson(response, 201, { status });
        return;
      }
      if (action?.kind === "edit") {
        const edit = await appendCommentEdit(
          model.rootPath,
          validation.change.name,
          action.commentId,
          input as ReviewCommentEditInput,
        );
        scheduleRefresh?.();
        writeJson(response, 201, { edit });
        return;
      }
      if (action?.kind === "thread") {
        writeJson(response, 405, { error: "Method not allowed." });
        return;
      }
      const record = await appendFeedbackRecord(
        model.rootPath,
        validation.change.name,
        input as ReviewFeedbackInput,
        validation,
      );
      scheduleRefresh?.();
      writeJson(response, 201, { feedback: record });
    } catch (error) {
      writeJson(response, 400, {
        error: error instanceof Error ? error.message : "Invalid feedback request.",
      });
    }
    return;
  }

  writeJson(response, 405, { error: "Method not allowed." });
}

function createModelWatcher(
  state: ServerState,
  root: string | undefined,
  reviewCompanionEnabled: boolean,
  reviewAgentEnabled: boolean,
): { close: () => void; scheduleRefresh: () => void } {
  const rootPath = path.resolve(root ?? process.cwd());
  const watchers: FSWatcher[] = [];
  let timer: NodeJS.Timeout | undefined;
  let refreshing = false;
  let refreshQueued = false;

  const scheduleRefresh = () => {
    if (timer) {
      clearTimeout(timer);
    }
    timer = setTimeout(() => {
      timer = undefined;
      void refreshModel();
    }, WATCH_DEBOUNCE_MS);
  };

  const refreshModel = async (): Promise<void> => {
    if (refreshing) {
      refreshQueued = true;
      return;
    }

    refreshing = true;
    try {
      await state.refreshModel();
      console.log(`OpenSpec viewer refreshed (version ${state.version}).`);
    } catch (error) {
      state.refreshError = error instanceof Error ? error.message : "Unknown refresh failure.";
      state.model = {
        ...state.model,
        refreshError: state.refreshError,
      };
      console.error(`OpenSpec viewer refresh failed: ${state.refreshError}`);
    } finally {
      refreshing = false;
      if (refreshQueued) {
        refreshQueued = false;
        scheduleRefresh();
      }
    }
  };

  const watchDirectory = (directory: string) => {
    if (!existsSync(directory)) {
      return;
    }

    try {
      watchers.push(
        watch(directory, { recursive: true }, (_eventType, filename) => {
          if (isRelevantWatchedFile(filename?.toString())) {
            scheduleRefresh();
          }
        }),
      );
    } catch (error) {
      console.error(
        `Unable to watch ${directory}: ${error instanceof Error ? error.message : "unknown error"}`,
      );
    }
  };

  watchDirectory(path.join(rootPath, "openspec"));
  if (reviewCompanionEnabled) {
    watchDirectory(feedbackSidecarDirectory(rootPath));
  }

  return {
    scheduleRefresh,
    close: () => {
      if (timer) {
        clearTimeout(timer);
      }
      for (const watcher of watchers) {
        watcher.close();
      }
    },
  };
}

function isRelevantWatchedFile(filename: string | undefined): boolean {
  if (!filename) {
    return true;
  }

  const normalized = filename.split(path.sep).join("/");
  return (
    normalized.endsWith(".md") ||
    normalized.endsWith(".yaml") ||
    normalized.endsWith(".yml") ||
    normalized.endsWith(".jsonl")
  );
}

type ReviewAgentAction =
  | { kind: "change-sessions"; changeName: string }
  | {
      kind: "session-events" | "session-message" | "session-abort" | "session-retry";
      sessionId: string;
    };

function decodeReviewAgentAction(pathname: string): ReviewAgentAction | null {
  const prefix = "/api/review-agent/";
  if (!pathname.startsWith(prefix)) {
    return null;
  }
  const parts = pathname.slice(prefix.length).split("/").map(decodeURIComponent);
  if (parts.length === 2 && parts[0] && parts[1] === "sessions") {
    return { kind: "change-sessions", changeName: parts[0] };
  }
  if (parts.length === 3 && parts[0] === "sessions" && parts[1]) {
    if (parts[2] === "events") return { kind: "session-events", sessionId: parts[1] };
    if (parts[2] === "message") return { kind: "session-message", sessionId: parts[1] };
    if (parts[2] === "abort") return { kind: "session-abort", sessionId: parts[1] };
    if (parts[2] === "retry") return { kind: "session-retry", sessionId: parts[1] };
  }
  return null;
}

function streamReviewAgentEvents(response: ServerResponse, handle: PiReviewSessionHandle): void {
  response.writeHead(200, {
    "content-type": "text/event-stream; charset=utf-8",
    "cache-control": "no-store",
    connection: "keep-alive",
  });
  const unsubscribe = handle.subscribe((event) => {
    response.write(`event: ${event.type}\n`);
    response.write(`data: ${JSON.stringify(event)}\n\n`);
  });
  response.on("close", unsubscribe);
}

async function persistReviewAgentEvent(
  rootPath: string,
  changeName: string,
  event: BrowserReviewAgentEvent,
): Promise<void> {
  if (event.type === "session_finished") {
    await appendReviewAgentResponseCompleted(
      rootPath,
      changeName,
      event.sessionId,
      event.responseText,
    );
  }
  if (event.type === "error") {
    await appendReviewAgentSessionFailed(rootPath, changeName, event.sessionId, event.error);
  }
  if (event.type === "session_aborted") {
    await appendReviewAgentSessionAborted(rootPath, changeName, event.sessionId);
  }
}

type FeedbackAction =
  | { kind: "comment"; changeName: string }
  | { kind: "thread" | "reply" | "status" | "edit"; changeName: string; commentId: string };

function decodeFeedbackAction(pathname: string): FeedbackAction | null {
  const prefix = "/api/review-feedback/";
  if (!pathname.startsWith(prefix)) {
    return null;
  }
  const parts = pathname.slice(prefix.length).split("/").map(decodeURIComponent);
  if (parts.length === 1 && parts[0]) {
    return { kind: "comment", changeName: parts[0] };
  }
  if (parts.length === 2 && parts[0] && parts[1]) {
    return { kind: "thread", changeName: parts[0], commentId: parts[1] };
  }
  if (
    parts.length === 3 &&
    parts[0] &&
    parts[1] &&
    (parts[2] === "reply" || parts[2] === "status" || parts[2] === "edit")
  ) {
    return { kind: parts[2], changeName: parts[0], commentId: parts[1] };
  }
  return null;
}

async function readJsonBody(request: IncomingMessage): Promise<unknown> {
  const chunks: Buffer[] = [];
  for await (const chunk of request) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }

  const body = Buffer.concat(chunks).toString("utf8").trim();
  return body ? JSON.parse(body) : {};
}

function writeJson(response: ServerResponse, status: number, payload: unknown): void {
  response.writeHead(status, {
    "content-type": "application/json; charset=utf-8",
    "cache-control": "no-store",
  });
  response.end(JSON.stringify(payload));
}
