import { createHash } from "node:crypto";
import type { ProjectWarning } from "./types.js";
import type { ResolvedReviewComment, ReviewAnchorState } from "./review-companion.js";

export interface ReviewAgentArtifactSnapshot {
  path: string;
  name: string;
  markdown: string;
  sha256: string;
}

export interface ReviewAgentFeedbackThread {
  id: string;
  artifactPath: string;
  body: string;
  status: ResolvedReviewComment["status"];
  anchorState: ReviewAnchorState;
  selectedText?: string;
  replies: ResolvedReviewComment["replies"];
  updatedAt: string;
}

export interface ReviewAgentBundle {
  id: string;
  changeName: string;
  createdAt: string;
  reviewerInstruction: string;
  selectedCommentIds: string[];
  includeAllOpenFeedback: boolean;
  artifacts: ReviewAgentArtifactSnapshot[];
  warnings: ProjectWarning[];
  feedback: ReviewAgentFeedbackThread[];
}

export interface CreateReviewAgentBundleInput {
  id: string;
  changeName: string;
  createdAt?: string;
  reviewerInstruction: string;
  selectedCommentIds?: string[];
  includeAllOpenFeedback?: boolean;
  artifacts: Array<{ name: string; relativePath: string; markdown: string }>;
  warnings: ProjectWarning[];
  feedback: ResolvedReviewComment[];
}

export function createReviewAgentBundle(input: CreateReviewAgentBundleInput): ReviewAgentBundle {
  const selectedCommentIds = input.selectedCommentIds ?? [];
  const selected = new Set(selectedCommentIds);
  const includeAllOpenFeedback = input.includeAllOpenFeedback ?? selected.size === 0;
  const feedback = input.feedback.filter((thread) => {
    if (selected.has(thread.id)) {
      return true;
    }
    return includeAllOpenFeedback && thread.status === "open";
  });

  return {
    id: input.id,
    changeName: input.changeName,
    createdAt: input.createdAt ?? new Date().toISOString(),
    reviewerInstruction: input.reviewerInstruction.trim(),
    selectedCommentIds,
    includeAllOpenFeedback,
    artifacts: input.artifacts.map((artifact) => ({
      name: artifact.name,
      path: artifact.relativePath,
      markdown: artifact.markdown,
      sha256: sha256(artifact.markdown),
    })),
    warnings: input.warnings,
    feedback: feedback.map((thread) => ({
      id: thread.id,
      artifactPath: thread.artifactPath,
      body: thread.body,
      status: thread.status,
      anchorState: thread.anchorState,
      selectedText: thread.anchor.selectedText,
      replies: thread.replies,
      updatedAt: thread.updatedAt,
    })),
  };
}

export function renderReviewAgentBundlePrompt(bundle: ReviewAgentBundle): string {
  return [
    "You are reviewing an active OpenSpec change from openspec-doc's browser companion.",
    "This browser-triggered session is non-mutating: do not edit files, mark comments resolved, or apply updates.",
    `Change: ${bundle.changeName}`,
    `Reviewer instruction: ${bundle.reviewerInstruction}`,
    "Respond with: summary, feedback triage, proposed artifact updates, unresolved questions, and next actions.",
    "",
    "## Included Feedback",
    bundle.feedback.length === 0
      ? "No open or selected feedback was included."
      : bundle.feedback
          .map(
            (thread) =>
              `- ${thread.id} [${thread.status}, ${thread.anchorState}] ${thread.artifactPath}: ${thread.body}`,
          )
          .join("\n"),
    "",
    "## Analyzer Warnings",
    bundle.warnings.length === 0
      ? "No analyzer warnings are associated with this change."
      : bundle.warnings
          .map((warning) => `- ${warning.code} ${warning.path}: ${warning.message}`)
          .join("\n"),
    "",
    "## Artifact Snapshots",
    ...bundle.artifacts.map(
      (artifact) =>
        `### ${artifact.path}\nsha256: ${artifact.sha256}\n\n\`\`\`markdown\n${artifact.markdown}\n\`\`\``,
    ),
  ].join("\n");
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}
