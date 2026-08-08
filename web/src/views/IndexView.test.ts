import { render, screen } from '@testing-library/vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Index, Scope } from '@/lib/scopes'
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

function scope(key: string, overrides: Partial<Scope> = {}): Scope {
  return {
    key,
    title: null,
    modifiedAt: null,
    openComments: 0,
    verdict: null,
    mostRecentlyActive: false,
    ...overrides,
  }
}

const PRESSURED_INDEX: Index = {
  sessions: [
    scope('0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74'),
    scope('0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d75', {
      title: 'Titled exploration session',
      modifiedAt: '2026-08-06T10:00:00Z',
    }),
    scope('0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d76', {
      title: 'Promoted observatory change',
      openComments: 2,
      verdict: 'move-to-proposal',
    }),
    scope('0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d77', { mostRecentlyActive: true }),
  ],
  changes: [
    scope('implement-observatory-design-system-with-a-realistically-long-identifier', {
      openComments: 1,
      verdict: 'comment-resolution',
      mostRecentlyActive: true,
    }),
    scope('preserve-offline-observation-assets', {
      title: 'Preserve offline observation assets',
      modifiedAt: '2026-08-06T11:30:00Z',
    }),
  ],
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('IndexView', () => {
  it('shows a distinct loading instrument while the index is in flight', () => {
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))

    const { container } = render(IndexView)

    expect(screen.getByText('Observing available scopes…')).toBeTruthy()
    expect(screen.getByText('Index signal / observing')).toBeTruthy()
    expect(container.querySelector('.index-observation__image')).toBeTruthy()
  })

  it('renders empty registers as successful emptiness, not failure', async () => {
    stubIndex({ sessions: [], changes: [] })

    render(IndexView)

    expect(await screen.findByText('No sessions discovered.')).toBeTruthy()
    expect(screen.getByText('No changes discovered.')).toBeTruthy()
    expect(screen.getByRole('heading', { name: 'Changes', level: 1 })).toBeTruthy()
    expect(screen.getByRole('heading', { name: 'Change register', level: 2 })).toBeTruthy()
    expect(screen.getByRole('heading', { name: 'Sessions', level: 2 })).toBeTruthy()
    expect(screen.queryByText('Index unavailable')).toBeNull()
  })

  it('keeps Changes primary and before Sessions when sessions outnumber changes', async () => {
    stubIndex(PRESSURED_INDEX)

    render(IndexView)

    const sessions = await screen.findByRole('heading', { name: 'Sessions', level: 2 })
    const changes = screen.getByRole('heading', { name: 'Changes', level: 1 })
    expect(changes.compareDocumentPosition(sessions) & Node.DOCUMENT_POSITION_FOLLOWING).not.toBe(0)
    expect(screen.getAllByRole('link')).toHaveLength(6)
  })

  it('renders every scope field returned by the server', async () => {
    stubIndex(PRESSURED_INDEX)

    render(IndexView)

    const key = PRESSURED_INDEX.changes[0]?.key ?? ''
    const link = await screen.findByRole('link', { name: key })
    expect(link.getAttribute('href')).toBe(`/changes/${key}`)
    expect(screen.getByText('1 open')).toBeTruthy()
    expect(screen.getByText('comment-resolution')).toBeTruthy()
    expect(screen.getAllByText('most recently active')).toHaveLength(2)
    expect(screen.getAllByText('—').length).toBeGreaterThan(0)
    expect(screen.getAllByText('0').length).toBeGreaterThan(0)
  })

  it('keeps observation and plate images decorative', async () => {
    stubIndex(PRESSURED_INDEX)

    const { container } = render(IndexView)
    await screen.findByRole('heading', { name: 'Sessions' })

    const images = container.querySelectorAll('.index-observation__image, .index-plate__image')
    expect(images).toHaveLength(4)
    for (const image of images) {
      expect(image.getAttribute('alt')).toBe('')
      expect(image.getAttribute('aria-hidden')).toBe('true')
    }
    expect(screen.queryByRole('img')).toBeNull()
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
