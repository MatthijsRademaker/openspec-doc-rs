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
      key: 'implement-observatory-design-system-with-a-realistically-long-identifier',
      title: null,
      modifiedAt: null,
      openComments: 1,
      verdict: 'comment-resolution',
      mostRecentlyActive: true,
    },
  ],
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('IndexView', () => {
  it('shows a distinct loading instrument while the index is in flight', () => {
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))

    render(IndexView)

    expect(screen.getByText('Observing available scopes…')).toBeTruthy()
    expect(screen.getByText('Index signal / observing')).toBeTruthy()
  })

  it('renders empty registers as successful emptiness, not failure', async () => {
    stubIndex({ sessions: [], changes: [] })

    render(IndexView)

    expect(await screen.findByText('No sessions discovered.')).toBeTruthy()
    expect(screen.getByText('No changes discovered.')).toBeTruthy()
    expect(screen.getByRole('heading', { name: 'Sessions' })).toBeTruthy()
    expect(screen.getByRole('heading', { name: 'Changes' })).toBeTruthy()
    expect(screen.queryByText('Index unavailable')).toBeNull()
  })

  it('renders every scope field returned by the server', async () => {
    stubIndex(ONE_CHANGE)

    render(IndexView)

    const key = ONE_CHANGE.changes[0]?.key ?? ''
    const link = await screen.findByRole('link', { name: key })
    expect(link.getAttribute('href')).toBe(`/changes/${key}`)
    expect(screen.getByText('1 open')).toBeTruthy()
    expect(screen.getByText('comment-resolution')).toBeTruthy()
    expect(screen.getByText('most recently active')).toBeTruthy()
  })

  it('renders a failure alert with its cause, never as emptiness', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('fetch failed')))

    render(IndexView)

    expect((await screen.findByRole('alert')).textContent).toContain(
      'The index could not be loaded: fetch failed',
    )
    expect(screen.getByText('Index unavailable')).toBeTruthy()
    expect(screen.queryByText('No sessions discovered.')).toBeNull()
  })

  it('ignores obsolete stored theme preferences and exposes no theme control', () => {
    window.localStorage.setItem('openspec-doc-theme', 'light')
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))

    render(IndexView)

    expect(document.documentElement.classList.contains('light')).toBe(false)
    expect(document.documentElement.classList.contains('dark')).toBe(false)
    expect(screen.queryByRole('button', { name: /theme/i })).toBeNull()
  })
})
