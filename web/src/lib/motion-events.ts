import type { CommentStatus } from '@/lib/scope-review'

export interface TriangulationEvent {
  threadId: string
  origin: 'source' | 'thread'
  destination: 'source' | 'thread'
}

export type TransmissionEvent =
  | { kind: 'comment'; target: 'artifact-comment' }
  | { kind: 'reply'; threadId: string }
  | { kind: 'status'; threadId: string }
  | { kind: 'verdict'; target: 'standing-verdict' }

export type MutationOutcome =
  | { kind: 'created-thread'; threadId: string }
  | { kind: 'extended-thread'; threadId: string }
  | { kind: 'changed-status'; threadId: string; status: 'open' | 'resolved' }
  | { kind: 'standing-verdict' }

export type ReviewReceipt =
  | { kind: 'created-thread'; threadId: string; source: 'reviewer' | 'remote' }
  | { kind: 'extended-thread'; threadId: string; source: 'reviewer' | 'remote' }
  | {
      kind: 'changed-status'
      threadId: string
      status: CommentStatus
      source: 'reviewer' | 'remote'
    }
  | { kind: 'standing-verdict'; source: 'reviewer' | 'remote' }
  | { kind: 'delivery'; source: 'remote' }
