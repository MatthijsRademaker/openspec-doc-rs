import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  createComment,
  eventPath,
  fetchScope,
  replyToComment,
  setCommentStatus,
  submitVerdict,
} from '@/lib/scope-review'

afterEach(() => vi.unstubAllGlobals())

function response(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  })
}

describe('scope review API', () => {
  it('encodes scope and comment identifiers in API paths', async () => {
    const fetch = vi.fn().mockImplementation(() => Promise.resolve(response({})))
    vi.stubGlobal('fetch', fetch)

    await fetchScope('change', 'name with spaces')
    await replyToComment('change', 'name with spaces', 'comment/id', 'Done.')
    await setCommentStatus('change', 'name with spaces', 'comment/id', 'resolved')
    await submitVerdict('change', 'name with spaces', 'comment-resolution')

    expect(fetch.mock.calls.map(([path]) => path)).toEqual([
      '/api/changes/name%20with%20spaces',
      '/api/changes/name%20with%20spaces/comments/comment%2Fid/replies',
      '/api/changes/name%20with%20spaces/comments/comment%2Fid/status',
      '/api/changes/name%20with%20spaces/verdict',
    ])
    expect(eventPath('session', 'session id')).toBe('/api/sessions/session%20id/events')
  })

  it('sends block source position when creating an anchored comment', async () => {
    const fetch = vi.fn().mockResolvedValue(response({}))
    vi.stubGlobal('fetch', fetch)

    await createComment('session', 's1', {
      kind: 'anchored',
      artifactPath: 'note.md',
      selectedText: 'Repeated.',
      searchFrom: 82,
      body: 'Second occurrence.',
    })

    const request = fetch.mock.calls[0]?.[1] as RequestInit
    expect(request.body).toBe(
      JSON.stringify({
        kind: 'anchored',
        artifactPath: 'note.md',
        selectedText: 'Repeated.',
        searchFrom: 82,
        body: 'Second occurrence.',
      }),
    )
  })

  it('surfaces server refusal reason instead of fake success', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(response({ error: 'selection crossed markup' }, 400)),
    )

    await expect(
      createComment('change', 'add-a', {
        kind: 'anchored',
        artifactPath: 'proposal.md',
        selectedText: 'rendered text',
        searchFrom: 0,
        body: 'Why?',
      }),
    ).rejects.toThrow('selection crossed markup')
  })
})
