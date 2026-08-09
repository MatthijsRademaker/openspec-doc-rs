import { describe, expect, it } from 'vitest'
import { MOCK_CHANGE_KEY, MOCK_SESSION_ID } from './data'

const origin = 'http://127.0.0.1'

describe('MSW API handlers', () => {
  it('serves index and scope fixtures through Node integration', async () => {
    // pi-lens-ignore: typescript.react.security.react-insecure-request.react-insecure-request
    const indexResponse = await fetch(`${origin}/api/index`)
    const index = (await indexResponse.json()) as {
      sessions: Array<{ key: string }>
      changes: Array<{ key: string }>
    }

    expect(indexResponse.ok).toBe(true)
    expect(index.sessions[0]?.key).toBe(MOCK_SESSION_ID)
    expect(index.changes[0]?.key).toBe(MOCK_CHANGE_KEY)

    const scopeResponse = await fetch(`${origin}/api/changes/${MOCK_CHANGE_KEY}`)
    const scope = (await scopeResponse.json()) as {
      kind: string
      artifacts: unknown[]
      comments: unknown[]
    }

    expect(scopeResponse.ok).toBe(true)
    expect(scope.kind).toBe('change')
    expect(scope.artifacts.length).toBeGreaterThan(0)
    expect(scope.artifacts).toHaveLength(5)
    expect(scope.comments.length).toBe(4)
  })

  it('applies comment mutations and exposes updated scope state', async () => {
    const path = `${origin}/api/sessions/${MOCK_SESSION_ID}`
    const createResponse = await fetch(`${path}/comments`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ kind: 'unanchored', body: 'Mock comment recorded.' }),
    })
    const comment = (await createResponse.json()) as { body: string }

    expect(createResponse.status).toBe(201)
    expect(comment.body).toBe('Mock comment recorded.')

    const scopeResponse = await fetch(path)
    const scope = (await scopeResponse.json()) as {
      comments: Array<{ comment: { body: string } }>
      commentCounts: { open: number }
    }

    expect(scope.comments.some((thread) => thread.comment.body === 'Mock comment recorded.')).toBe(
      true,
    )
    expect(scope.commentCounts.open).toBe(2)
  })
})
