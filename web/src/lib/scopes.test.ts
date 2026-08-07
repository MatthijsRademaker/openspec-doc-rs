import { afterEach, describe, expect, it, vi } from 'vitest'

import { fetchIndex, type Index } from '@/lib/scopes'

const INDEX: Index = {
  sessions: [],
  changes: [
    {
      key: 'add-dashboard-development-harness',
      title: null,
      modifiedAt: '2026-08-06T12:00:00Z',
      openComments: 2,
      verdict: 'comment-resolution',
      mostRecentlyActive: true,
    },
  ],
}

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { 'content-type': 'application/json' },
    ...init,
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('fetchIndex', () => {
  it('returns the parsed index on success', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(INDEX)))

    await expect(fetchIndex()).resolves.toEqual(INDEX)
  })

  it('throws the response status on failure rather than faking an empty index', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(
          new Response('nope', { status: 503, statusText: 'Service Unavailable' }),
        ),
    )

    await expect(fetchIndex()).rejects.toThrow(/503 Service Unavailable/)
  })

  it('rejects on a malformed body rather than faking an empty index', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('not json', { status: 200 })))

    await expect(fetchIndex()).rejects.toThrow()
  })

  it('propagates a network failure', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('fetch failed')))

    await expect(fetchIndex()).rejects.toThrow('fetch failed')
  })
})
