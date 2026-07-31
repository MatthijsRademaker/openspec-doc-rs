export type ReviewAgentMode = "enabled" | "disabled" | "unavailable";

export interface ReviewAgentState {
  mode: ReviewAgentMode;
  reason?: string;
}

export interface ReviewAgentOptions {
  enabled?: boolean;
  unavailableReason?: string;
}

export function createReviewAgentState(options: ReviewAgentOptions = {}): ReviewAgentState {
  if (options.enabled === false) {
    return {
      mode: "disabled",
      reason: "Browser Pi review analysis was explicitly disabled for this serve session.",
    };
  }

  if (options.unavailableReason) {
    return {
      mode: "unavailable",
      reason: options.unavailableReason,
    };
  }

  return { mode: "enabled" };
}

export function reviewAgentStatusMessage(state: ReviewAgentState): string {
  if (state.mode === "enabled") {
    return "Browser Pi review analysis enabled by default; sessions are non-mutating.";
  }
  if (state.mode === "disabled") {
    return "Browser Pi review analysis disabled; Ask Pi actions are unavailable.";
  }
  return `Browser Pi review analysis unavailable${state.reason ? `: ${state.reason}` : "."}`;
}
