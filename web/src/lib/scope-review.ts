import type { Verdict } from '@/lib/scopes'

export type ScopeKind = 'session' | 'change'
export type CommentStatus = 'open' | 'addressed' | 'resolved'
export type AnchorState = 'exact' | 'fuzzy' | 'orphaned' | 'missing' | 'unanchored'

export interface SourceRange {
  start: number
  end: number
}

export interface Block {
  id: string
  html: string
  source: string
  range: SourceRange
}

export interface Artifact {
  path: string
  blocks: Block[]
}

export interface Anchor {
  artifactPath: string
  selectedText: string
  headingPath: string[]
  beforeText: string
  afterText: string
  startOffset: number
  endOffset: number
}

export interface CommentRecord {
  id: string
  anchor?: Anchor
  body: string
  createdAt: string
}

export interface Reply {
  id: string
  commentId: string
  author: 'reviewer' | 'agent'
  body: string
  createdAt: string
}

export interface Thread {
  comment: CommentRecord
  status: CommentStatus
  replies: Reply[]
  anchorState: AnchorState
  blockId: string | null
}

export interface CommentCounts {
  open: number
  addressed: number
  resolved: number
}

export interface VerdictRecord {
  id: string
  verdict: Verdict
  notes: string
  createdAt: string
}

export interface StandingVerdict {
  id: string
  verdict: Verdict
  createdAt: string
  directiveDelivered: boolean
  directivePending: boolean
}

export interface ScopeDetail {
  kind: ScopeKind
  key: string
  title: string | null
  artifacts: Artifact[]
  comments: Thread[]
  commentCounts: CommentCounts
  verdicts: VerdictRecord[]
  standingVerdict: StandingVerdict | null
}

export type NewComment =
  | {
      kind: 'anchored'
      artifactPath: string
      selectedText: string
      searchFrom: number
      body: string
    }
  | { kind: 'unanchored'; body: string }

function scopePath(kind: ScopeKind, key: string): string {
  const collection = kind === 'session' ? 'sessions' : 'changes'
  return `/api/${collection}/${encodeURIComponent(key)}`
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, init)
  if (!response.ok) {
    let reason = `${response.status} ${response.statusText}`
    try {
      const payload = (await response.json()) as { error?: unknown }
      if (typeof payload.error === 'string') reason = payload.error
    } catch {
      // Status remains useful when an intermediary returns a non-JSON failure.
    }
    throw new Error(reason)
  }

  return response.json() as Promise<T>
}

function post<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

export function fetchScope(kind: ScopeKind, key: string): Promise<ScopeDetail> {
  return request(scopePath(kind, key))
}

export function createComment(
  kind: ScopeKind,
  key: string,
  comment: NewComment,
): Promise<CommentRecord> {
  return post(`${scopePath(kind, key)}/comments`, comment)
}

export function replyToComment(
  kind: ScopeKind,
  key: string,
  commentId: string,
  body: string,
): Promise<Reply> {
  return post(`${scopePath(kind, key)}/comments/${encodeURIComponent(commentId)}/replies`, { body })
}

export function setCommentStatus(
  kind: ScopeKind,
  key: string,
  commentId: string,
  status: 'open' | 'resolved',
): Promise<void> {
  return post(`${scopePath(kind, key)}/comments/${encodeURIComponent(commentId)}/status`, {
    status,
  })
}

export function submitVerdict(
  kind: ScopeKind,
  key: string,
  verdict: Verdict,
): Promise<VerdictRecord> {
  return post(`${scopePath(kind, key)}/verdict`, { verdict })
}

export function eventPath(kind: ScopeKind, key: string): string {
  return `${scopePath(kind, key)}/events`
}
