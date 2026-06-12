import type { ResolvedReviewComment, ReviewDigest, ReviewDigestGroup } from "./review-companion.js";

export function createReviewDigest(
  changeName: string,
  comments: ResolvedReviewComment[],
  generatedAt = new Date().toISOString(),
): ReviewDigest {
  const openComments = comments.filter((comment) => comment.status === "open");
  const groups = new Map<string, ResolvedReviewComment[]>();

  for (const comment of openComments) {
    const key = inferGroupKey(comment);
    groups.set(key, [...(groups.get(key) ?? []), comment]);
  }

  return {
    changeName,
    generatedAt,
    openCommentCount: openComments.length,
    groups: [...groups.entries()].map(([title, groupComments]) =>
      createDigestGroup(title, groupComments),
    ),
    requiresExplicitApproval: true,
  };
}

function createDigestGroup(title: string, comments: ResolvedReviewComment[]): ReviewDigestGroup {
  const artifacts = [...new Set(comments.map((comment) => comment.artifactPath))];
  return {
    title,
    sourceCommentIds: comments.map((comment) => comment.id),
    summary: comments.map((comment) => comment.body).join(" "),
    artifacts,
    proposedUpdates: comments.map((comment) => ({
      artifactPath: comment.artifactPath,
      summary: `Consider updating ${comment.artifactPath} based on comment ${comment.id}: ${comment.body}`,
    })),
    unresolvedQuestions: comments
      .filter((comment) => /\?|clarify|unclear|what about/i.test(comment.body))
      .map((comment) => `Clarify intent for comment ${comment.id}: ${comment.body}`),
  };
}

function inferGroupKey(comment: ResolvedReviewComment): string {
  if (comment.anchor.headingPath && comment.anchor.headingPath.length > 0) {
    return `${comment.artifactPath} · ${comment.anchor.headingPath.at(-1)}`;
  }
  return comment.artifactPath;
}
