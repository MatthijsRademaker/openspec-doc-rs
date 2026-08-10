import { http, HttpResponse } from 'msw'
import type { NewComment, ScopeKind } from '@/lib/scope-review'
import type { Verdict } from '@/lib/scopes'
import {
  addMockComment,
  addMockReply,
  getMockScope,
  getMockState,
  rewriteMockArtifact,
  scopeAddress,
  setMockCommentStatus,
  submitMockVerdict,
} from './data'

const verdicts: readonly Verdict[] = ['keep-exploring', 'move-to-proposal', 'comment-resolution']

/** The Rust server's event payload. A bare marker would fail the client's payload parse. */
interface LiveUpdate {
  artifactsChanged: boolean
  reviewStateChanged: boolean
}

const REVIEW_STATE_CHANGED: LiveUpdate = { artifactsChanged: false, reviewStateChanged: true }
const ARTIFACTS_CHANGED: LiveUpdate = { artifactsChanged: true, reviewStateChanged: false }

type EventController = ReadableStreamDefaultController<Uint8Array>

const eventStreams = new Map<string, Set<EventController>>()
const encoder = new TextEncoder()

function pathParam(value: unknown): string {
  if (typeof value === 'string') return value
  if (Array.isArray(value) && typeof value[0] === 'string') return value[0]
  return ''
}

function errorResponse(status: number, error: string): Response {
  return HttpResponse.json({ error }, { status })
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isNewComment(value: unknown): value is NewComment {
  if (!isRecord(value) || typeof value.body !== 'string') return false
  if (value.kind === 'unanchored') return true
  return (
    value.kind === 'anchored' &&
    typeof value.artifactPath === 'string' &&
    typeof value.selectedText === 'string' &&
    typeof value.searchFrom === 'number' &&
    Number.isInteger(value.searchFrom) &&
    value.searchFrom >= 0
  )
}

function isVerdict(value: unknown): value is Verdict {
  return typeof value === 'string' && verdicts.includes(value as Verdict)
}

function removeEventStream(address: string, controller: EventController): void {
  const streams = eventStreams.get(address)
  if (!streams) return
  streams.delete(controller)
  if (!streams.size) eventStreams.delete(address)
}

function eventResponse(address: string): Response {
  let controller: EventController | undefined
  const body = new ReadableStream<Uint8Array>({
    start(nextController) {
      controller = nextController
      const streams = eventStreams.get(address) ?? new Set<EventController>()
      streams.add(nextController)
      eventStreams.set(address, streams)
      nextController.enqueue(encoder.encode(': mock event stream\n\n'))
    },
    cancel() {
      if (controller) removeEventStream(address, controller)
    },
  })

  return new HttpResponse(body, {
    headers: {
      'cache-control': 'no-cache',
      connection: 'keep-alive',
      'content-type': 'text/event-stream',
    },
  })
}

function broadcast(address: string, update: LiveUpdate): void {
  const streams = eventStreams.get(address)
  if (!streams) return
  const payload = encoder.encode(`data: ${JSON.stringify(update)}\n\n`)

  for (const controller of streams) {
    if (controller.desiredSize === null) {
      streams.delete(controller)
      continue
    }
    controller.enqueue(payload)
  }
  if (!streams.size) eventStreams.delete(address)
}

function scopeResponse(kind: ScopeKind, key: string): Response {
  const scope = getMockScope(kind, key)
  return scope ? HttpResponse.json(scope) : errorResponse(404, `Unknown mock ${kind} scope: ${key}`)
}

function createScopeHandlers(
  kind: ScopeKind,
  collection: 'sessions' | 'changes',
  parameter: 'id' | 'name',
) {
  const basePath = `*/api/${collection}/:${parameter}`
  const readScope = (params: Record<string, unknown>) => {
    const key = pathParam(params[parameter])
    return { address: scopeAddress(kind, key), key }
  }

  return [
    http.get(basePath, ({ params }) => {
      const { key } = readScope(params)
      return scopeResponse(kind, key)
    }),
    http.get(`${basePath}/events`, ({ params }) => {
      const { address, key } = readScope(params)
      return getMockScope(kind, key)
        ? eventResponse(address)
        : errorResponse(404, `Unknown mock ${kind} scope: ${key}`)
    }),
    http.post(`${basePath}/comments`, async ({ params, request }) => {
      const { address, key } = readScope(params)
      if (!getMockScope(kind, key)) return errorResponse(404, `Unknown mock ${kind} scope: ${key}`)
      const body = (await request.json()) as unknown
      if (!isNewComment(body)) return errorResponse(400, 'Invalid mock comment body')
      const comment = addMockComment(kind, key, body)
      broadcast(address, REVIEW_STATE_CHANGED)
      return HttpResponse.json(comment, { status: 201 })
    }),
    http.post(`${basePath}/comments/:commentId/replies`, async ({ params, request }) => {
      const { address, key } = readScope(params)
      if (!getMockScope(kind, key)) return errorResponse(404, `Unknown mock ${kind} scope: ${key}`)
      const body = (await request.json()) as unknown
      if (!isRecord(body) || typeof body.body !== 'string' || !body.body.trim()) {
        return errorResponse(400, 'Invalid mock reply body')
      }
      const reply = addMockReply(kind, key, pathParam(params.commentId), body.body)
      if (!reply) return errorResponse(404, `Unknown mock comment: ${pathParam(params.commentId)}`)
      broadcast(address, REVIEW_STATE_CHANGED)
      return HttpResponse.json(reply, { status: 201 })
    }),
    http.post(`${basePath}/comments/:commentId/status`, async ({ params, request }) => {
      const { address, key } = readScope(params)
      if (!getMockScope(kind, key)) return errorResponse(404, `Unknown mock ${kind} scope: ${key}`)
      const body = (await request.json()) as unknown
      if (!isRecord(body) || (body.status !== 'open' && body.status !== 'resolved')) {
        return errorResponse(400, 'Invalid mock comment status')
      }
      const update = setMockCommentStatus(kind, key, pathParam(params.commentId), body.status)
      if (!update) return errorResponse(404, `Unknown mock comment: ${pathParam(params.commentId)}`)
      broadcast(address, REVIEW_STATE_CHANGED)
      return HttpResponse.json(update)
    }),
    http.post(`${basePath}/verdict`, async ({ params, request }) => {
      const { address, key } = readScope(params)
      if (!getMockScope(kind, key)) return errorResponse(404, `Unknown mock ${kind} scope: ${key}`)
      const body = (await request.json()) as unknown
      if (!isRecord(body) || !isVerdict(body.verdict)) {
        return errorResponse(400, 'Invalid mock verdict')
      }
      const record = submitMockVerdict(kind, key, body.verdict)
      broadcast(address, REVIEW_STATE_CHANGED)
      return HttpResponse.json(record, { status: 201 })
    }),
    // The one live-update path no reviewer mutation reaches: an agent rewriting an artifact on
    // disk while the reviewer reads it.
    http.post(`${basePath}/mock/artifact-rewrite`, async ({ params, request }) => {
      const { address, key } = readScope(params)
      if (!getMockScope(kind, key)) return errorResponse(404, `Unknown mock ${kind} scope: ${key}`)
      const body = (await request.json()) as unknown
      const path =
        isRecord(body) && typeof body.artifactPath === 'string' ? body.artifactPath : undefined
      const artifact = rewriteMockArtifact(kind, key, path)
      if (!artifact) return errorResponse(404, `Unknown mock artifact: ${path ?? '<first>'}`)
      broadcast(address, ARTIFACTS_CHANGED)
      return HttpResponse.json(artifact)
    }),
  ]
}

export const handlers = [
  http.get('*/api/index', () => HttpResponse.json(getMockState().index)),
  ...createScopeHandlers('session', 'sessions', 'id'),
  ...createScopeHandlers('change', 'changes', 'name'),
]
