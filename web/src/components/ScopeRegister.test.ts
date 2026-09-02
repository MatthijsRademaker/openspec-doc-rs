import { fireEvent, render, screen, waitFor } from '@testing-library/vue'
import { defineComponent } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import ScopeRegister from '@/components/ScopeRegister.vue'
import type { Scope } from '@/lib/scopes'

const NOW = new Date('2026-08-06T12:00:00Z')
const RouteTarget = defineComponent({ template: '<p>target</p>' })

function renderRegister(props: InstanceType<typeof ScopeRegister>['$props']) {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: RouteTarget },
      { path: '/changes/:name', component: RouteTarget },
      { path: '/sessions/:id', component: RouteTarget },
    ],
  })
  return {
    router,
    ...render(ScopeRegister, { props, global: { plugins: [router] } }),
  }
}

function scope(overrides: Partial<Scope> = {}): Scope {
  return {
    key: 'implement-observatory-design-system-with-a-realistically-long-identifier',
    title: null,
    modifiedAt: null,
    openComments: 0,
    verdict: null,
    mostRecentlyActive: false,
    ...overrides,
  }
}

describe('ScopeRegister', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(NOW)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  /* Pointing at a coordinate, or tabbing onto it, is the gesture before choosing it. The read the
     choice would start begins there so the hold has nothing left to wait on. */
  it('starts the scope read on hover and on keyboard focus', async () => {
    const read = vi.fn(
      async (url: RequestInfo | URL) =>
        new Response(JSON.stringify({ key: String(url), artifacts: [] }), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        }),
    )
    vi.stubGlobal('fetch', read)
    const hovered = scope({ key: 'speculated-by-pointer' })
    const focused = scope({ key: 'speculated-by-keyboard' })
    renderRegister({ scopes: [hovered, focused], prefix: 'sessions', title: 'Sessions' })

    await fireEvent.mouseEnter(screen.getByRole('link', { name: hovered.key }))
    await fireEvent.focus(screen.getByRole('link', { name: focused.key }))

    expect(read.mock.calls.map(([url]) => url)).toEqual([
      `/api/sessions/${hovered.key}`,
      `/api/sessions/${focused.key}`,
    ])
    vi.unstubAllGlobals()
  })

  it('renders a named empty instrument', () => {
    renderRegister({ scopes: [], prefix: 'changes', title: 'Changes' })

    expect(screen.getByRole('heading', { name: 'Changes' })).toBeTruthy()
    expect(screen.getByText('No changes discovered.')).toBeTruthy()
    expect(screen.getByText('0 scopes')).toBeTruthy()
  })

  it('keeps titled and promoted identities separate from exact keys', () => {
    const item = scope({ title: 'Promoted observatory change' })
    renderRegister({ scopes: [item], prefix: 'changes', title: 'Changes' })

    const link = screen.getByRole('link', { name: 'Promoted observatory change' })
    expect(link.getAttribute('href')).toBe(`/changes/${item.key}`)
    expect(screen.getByText(item.key)).toBeTruthy()
    expect(link.classList.contains('scope-entry__link--identifier')).toBe(false)
  })

  it('renders one operational identity for an untitled session', () => {
    const item = scope()
    renderRegister({ scopes: [item], prefix: 'sessions', title: 'Sessions' })

    const link = screen.getByRole('link', { name: item.key })
    expect(link.getAttribute('href')).toBe(`/sessions/${item.key}`)
    expect(link.classList).toContain('scope-entry__link--identifier')
    expect(screen.getAllByText(item.key)).toHaveLength(1)
  })

  it('keeps modified time, comment count, verdict, and recent marker visible', () => {
    renderRegister({
      scopes: [
        scope({
          title: 'Observatory design system',
          modifiedAt: '2026-08-06T10:00:00Z',
          openComments: 3,
          verdict: 'comment-resolution',
          mostRecentlyActive: true,
        }),
      ],
      prefix: 'changes',
      title: 'Changes',
    })

    expect(screen.getByText('2h ago')).toBeTruthy()
    expect(screen.getByText('3 open')).toBeTruthy()
    expect(screen.getByText('comment-resolution')).toBeTruthy()
    expect(screen.getByText('most recently active')).toBeTruthy()
    expect(screen.getByText('◇')).toBeTruthy()
    expect(screen.getByText('◆')).toBeTruthy()
  })

  it('routes primary activation and preserves modified-click behavior', async () => {
    vi.useRealTimers()
    const item = scope()
    const { router, container } = renderRegister({
      scopes: [item],
      prefix: 'changes',
      title: 'Changes',
    })
    const link = screen.getByRole('link', { name: item.key })

    // jsdom cannot open a background tab; cancel browser default while preserving modifier data.
    link.addEventListener('click', (event) => event.preventDefault(), {
      capture: true,
      once: true,
    })
    await fireEvent.click(link, { ctrlKey: true })
    expect(router.currentRoute.value.path).toBe('/')
    expect(container.querySelector('.scope-entry--acquiring')).toBeNull()

    await fireEvent.click(link)
    await waitFor(() => expect(router.currentRoute.value.path).toBe(`/changes/${item.key}`))
  })

  it('renders explicit zero and dashes instead of fabricated values', () => {
    renderRegister({ scopes: [scope()], prefix: 'changes', title: 'Changes' })

    expect(screen.getAllByText('—')).toHaveLength(2)
    expect(screen.getByText('0')).toBeTruthy()
    expect(screen.queryByText('most recently active')).toBeNull()
  })
})
