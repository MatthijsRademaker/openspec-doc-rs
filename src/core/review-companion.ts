export type ReviewCommentStatus = "open" | "addressed" | "resolved";

export type ReviewAnchorState = "exact" | "fuzzy" | "orphaned" | "missing";

export interface ReviewAnchorContext {
  artifactPath: string;
  artifactKind?: string;
  selectedText: string;
  headingPath?: string[];
  beforeText?: string;
  afterText?: string;
  startOffset?: number;
  endOffset?: number;
}

export interface AnchoredReviewComment {
  id: string;
  changeName: string;
  artifactPath: string;
  artifactKind?: string;
  anchor: ReviewAnchorContext;
  body: string;
  author?: string;
  source: "browser" | "pi" | "import";
  status: ReviewCommentStatus;
  createdAt: string;
  updatedAt: string;
}

export interface ReviewThreadReply {
  id: string;
  commentId: string;
  changeName: string;
  body: string;
  author?: string;
  source: "browser" | "pi" | "import";
  createdAt: string;
}

export interface ReviewStatusUpdate {
  id: string;
  commentId: string;
  changeName: string;
  status: ReviewCommentStatus;
  author?: string;
  source: "browser" | "pi" | "import";
  createdAt: string;
}

export interface ReviewCommentEdit {
  id: string;
  commentId: string;
  changeName: string;
  body: string;
  author?: string;
  source: "browser" | "pi" | "import";
  createdAt: string;
}

export interface ResolvedReviewComment extends AnchoredReviewComment {
  replies: ReviewThreadReply[];
  statusHistory: ReviewStatusUpdate[];
  editHistory: ReviewCommentEdit[];
  anchorState: ReviewAnchorState;
  resolvedOffset?: number;
}

export interface ReviewDigestGroup {
  title: string;
  sourceCommentIds: string[];
  summary: string;
  artifacts: string[];
  proposedUpdates: ReviewDigestProposedUpdate[];
  unresolvedQuestions: string[];
}

export interface ReviewDigestProposedUpdate {
  artifactPath: string;
  summary: string;
  suggestedText?: string;
}

export interface ReviewDigest {
  changeName: string;
  generatedAt: string;
  openCommentCount: number;
  groups: ReviewDigestGroup[];
  requiresExplicitApproval: true;
}
