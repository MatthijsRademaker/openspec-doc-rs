import type { ReviewAgentSessionRecord } from "./review-agent.js";
import { buildReviewAgentPrompt } from "./review-agent.js";

export type BrowserReviewAgentEvent =
  | { type: "session_started"; sessionId: string }
  | { type: "agent_start"; sessionId: string }
  | { type: "text_delta"; sessionId: string; delta: string }
  | { type: "tool_start"; sessionId: string; toolName: string }
  | { type: "tool_end"; sessionId: string; toolName: string; isError: boolean }
  | { type: "agent_end"; sessionId: string }
  | { type: "session_finished"; sessionId: string; responseText: string }
  | { type: "session_aborted"; sessionId: string }
  | { type: "error"; sessionId: string; error: string; retryable: boolean };

export interface PiReviewSessionHandle {
  id: string;
  events: BrowserReviewAgentEvent[];
  subscribe(listener: (event: BrowserReviewAgentEvent) => void): () => void;
  followUp(message: string): Promise<void>;
  abort(): Promise<void>;
}

export interface PiReviewBridge {
  start(record: ReviewAgentSessionRecord): Promise<PiReviewSessionHandle>;
}

type PiModule = {
  AuthStorage: { create: () => unknown };
  ModelRegistry: { create: (authStorage: unknown) => { getAvailable: () => Promise<unknown[]> } };
  SessionManager: { inMemory: () => unknown };
  createAgentSession: (options: Record<string, unknown>) => Promise<{ session: SdkSession }>;
};

type SdkSession = {
  subscribe: (listener: (event: SdkEvent) => void) => void;
  prompt: (text: string) => Promise<void>;
  followUp: (text: string) => Promise<void>;
  abort: () => Promise<void>;
  isStreaming: boolean;
};

type SdkEvent = { type: string; [key: string]: unknown };

export class SdkPiReviewBridge implements PiReviewBridge {
  async start(record: ReviewAgentSessionRecord): Promise<PiReviewSessionHandle> {
    let pi: PiModule;
    try {
      pi = (await dynamicImport("@earendil-works/pi-coding-agent")) as PiModule;
    } catch (error) {
      throw new Error(
        `Pi SDK is unavailable (${error instanceof Error ? error.message : "module could not be loaded"}).`,
      );
    }

    const authStorage = pi.AuthStorage.create();
    const modelRegistry = pi.ModelRegistry.create(authStorage);
    const availableModels = await modelRegistry.getAvailable();
    if (availableModels.length === 0) {
      throw new Error("No Pi model credentials are available for browser review analysis.");
    }

    const { session } = await pi.createAgentSession({
      cwd: process.cwd(),
      authStorage,
      modelRegistry,
      sessionManager: pi.SessionManager.inMemory(),
      tools: ["read", "grep", "find", "ls"],
    });

    return createSdkHandle(record, session, buildReviewAgentPrompt(record));
  }
}

export class UnavailablePiReviewBridge implements PiReviewBridge {
  constructor(private readonly reason = "Pi SDK bridge is not configured.") {}

  async start(): Promise<PiReviewSessionHandle> {
    throw new Error(this.reason);
  }
}

function createSdkHandle(
  record: ReviewAgentSessionRecord,
  session: SdkSession,
  prompt: string,
): PiReviewSessionHandle {
  const listeners = new Set<(event: BrowserReviewAgentEvent) => void>();
  const events: BrowserReviewAgentEvent[] = [];
  let responseText = "";

  const emit = (event: BrowserReviewAgentEvent) => {
    events.push(event);
    for (const listener of listeners) {
      listener(event);
    }
  };

  session.subscribe((event) => {
    const normalized = normalizeSdkEvent(record.id, event);
    for (const item of normalized) {
      if (item.type === "text_delta") {
        responseText += item.delta;
      }
      emit(item);
    }
  });

  queueMicrotask(() => {
    emit({ type: "session_started", sessionId: record.id });
    session
      .prompt(prompt)
      .then(() => {
        emit({ type: "session_finished", sessionId: record.id, responseText });
      })
      .catch((error: unknown) => {
        emit({
          type: "error",
          sessionId: record.id,
          error: error instanceof Error ? error.message : "Pi review analysis failed.",
          retryable: true,
        });
      });
  });

  return {
    id: record.id,
    events,
    subscribe(listener) {
      listeners.add(listener);
      for (const event of events) {
        listener(event);
      }
      return () => listeners.delete(listener);
    },
    async followUp(message: string) {
      if (session.isStreaming) {
        await session.followUp(message);
        return;
      }
      await session.prompt(message);
    },
    async abort() {
      await session.abort();
      emit({ type: "session_aborted", sessionId: record.id });
    },
  };
}

function normalizeSdkEvent(sessionId: string, event: SdkEvent): BrowserReviewAgentEvent[] {
  if (event.type === "agent_start") {
    return [{ type: "agent_start", sessionId }];
  }
  if (event.type === "agent_end") {
    return [{ type: "agent_end", sessionId }];
  }
  const assistantMessageEvent = event.assistantMessageEvent as
    | { type?: unknown; delta?: unknown }
    | undefined;
  if (event.type === "message_update" && assistantMessageEvent?.type === "text_delta") {
    return [
      {
        type: "text_delta",
        sessionId,
        delta: typeof assistantMessageEvent.delta === "string" ? assistantMessageEvent.delta : "",
      },
    ];
  }
  if (event.type === "tool_execution_start") {
    return [
      {
        type: "tool_start",
        sessionId,
        toolName: typeof event.toolName === "string" ? event.toolName : "tool",
      },
    ];
  }
  if (event.type === "tool_execution_end") {
    return [
      {
        type: "tool_end",
        sessionId,
        toolName: typeof event.toolName === "string" ? event.toolName : "tool",
        isError: event.isError === true,
      },
    ];
  }
  return [];
}

function dynamicImport(specifier: string): Promise<unknown> {
  return new Function("specifier", "return import(specifier)")(specifier) as Promise<unknown>;
}
