import { render, screen } from '@testing-library/vue'
import { afterEach, describe, expect, it, vi } from 'vitest'

import type { Index } from '@/lib/scopes'
import IndexView from '@/views/IndexView.vue'

function stubIndex(body: Index) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue(
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    ),
  )
}

const ONE_CHANGE: Index = {
  sessions: [],
  changes: [
    {
      key: 'add-dashboard-development-harness',
      title: null,
      modifiedAt: null,
      openComments: 1,
      verdict: null,
      mostRecentlyActive: false,
    },
  ],
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('IndexView', () => {
  it('shows a loading state while the index is in flight', () => {
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))

    render(IndexView)

    expect(screen.getByText('Loading…')).toBeTruthy()
  })

  it('renders an empty index as empty, not as an error', async () => {
    stubIndex({ sessions: [], changes: [] })

    render(IndexView)

    expect(await screen.findAllByText('None discovered.')).toHaveLength(2)
    expect(screen.getByText('Sessions')).toBeTruthy()
    expect(screen.getByText('Changes')).toBeTruthy()
  })

  it('renders the scopes the server returned', async () => {
    stubIndex(ONE_CHANGE)

    render(IndexView)

    const link = await screen.findByRole('link', { name: 'add-dashboard-development-harness' })
    expect(link.getAttribute('href')).toBe('/changes/add-dashboard-development-harness')
  })

  it('says the index failed to load, and why', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('fetch failed')))

    render(IndexView)

    expect(await screen.findByText(/The index could not be loaded: fetch failed/)).toBeTruthy()
    expect(screen.queryByText('Loading…')).toBeNull()
  })
})
