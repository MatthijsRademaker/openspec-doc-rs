import { defineComponent } from 'vue'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  installRouteTransitions,
  NAVIGATION_HOLD_CEILING_MS,
  returningScopeKey,
  takeHeldScope,
} from '@/lib/route-transition'

const Stub = defineComponent({ template: '<p>route</p>' })
const originalStartViewTransition = document.startViewTransition
const fixtureDetail = { key: 'observatory-change', artifacts: [] }

function reducedMotion(matches: boolean): void {
  vi.stubGlobal(
    'matchMedia',
    vi.fn(() => ({ matches })),
  )
}

/** Runs the update the browser would run, and reports what the page looked like while it ran. */
function stubViewTransition(observe?: () => void) {
  const start = vi.fn((update: () => void | Promise<void>) => {
    const updateCallbackDone = Promise.resolve()
      .then(() => update())
      .then(() => {
        observe?.()
      })
    return { updateCallbackDone, finished: updateCallbackDone }
  })
  Object.defineProperty(document, 'startViewTransition', { configurable: true, value: start })
  return start
}

function makeRouter(): Router {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'index', component: Stub },
      { path: '/sessions/:id', name: 'session', component: Stub },
      { path: '/changes/:name', name: 'change', component: Stub },
    ],
  })
  installRouteTransitions(router)
  return router
}

beforeEach(() => {
  reducedMotion(false)
})

afterEach(() => {
  Object.defineProperty(document, 'startViewTransition', {
    configurable: true,
    value: originalStartViewTransition,
  })
  takeHeldScope('change', fixtureDetail.key)
  returningScopeKey.value = undefined
  vi.unstubAllGlobals()
})

describe('router-owned route transitions', () => {
  it('holds the navigation and hands the loaded scope to the destination', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify(fixtureDetail), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        }),
      ),
    )
    const start = stubViewTransition()
    const router = makeRouter()
    await router.push('/')

    await router.push(`/changes/${fixtureDetail.key}`)

    expect(start).toHaveBeenCalledOnce()
    expect(router.currentRoute.value.name).toBe('change')
    // The hold bought the gesture, not a second round trip.
    expect(takeHeldScope('change', fixtureDetail.key)).toEqual(fixtureDetail)
    expect(fetch).toHaveBeenCalledTimes(1)
  })

  /* A load that fails must never present as a click that did nothing: the route commits and the
     scope's own failure state reports the error. */
  it('commits the route when the prefetch fails', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('fetch failed')))
    stubViewTransition()
    const router = makeRouter()
    await router.push('/')

    await router.push(`/changes/${fixtureDetail.key}`)

    expect(router.currentRoute.value.path).toBe(`/changes/${fixtureDetail.key}`)
    expect(takeHeldScope('change', fixtureDetail.key)).toBeUndefined()
  })

  it('commits the route when the hold ceiling elapses, and discards the late arrival', async () => {
    let settle: ((response: Response) => void) | undefined
    vi.stubGlobal(
      'fetch',
      vi.fn(
        () =>
          new Promise<Response>((resolve) => {
            settle = resolve
          }),
      ),
    )
    stubViewTransition()
    const router = makeRouter()
    await router.push('/')

    const before = Date.now()
    await router.push(`/changes/${fixtureDetail.key}`)

    expect(Date.now() - before).toBeGreaterThanOrEqual(NAVIGATION_HOLD_CEILING_MS - 5)
    expect(router.currentRoute.value.path).toBe(`/changes/${fixtureDetail.key}`)
    settle?.(
      new Response(JSON.stringify(fixtureDetail), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    )
    await Promise.resolve()
    expect(takeHeldScope('change', fixtureDetail.key)).toBeUndefined()
  })

  it('carries the coordinate back to the index entry being returned to', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify({ sessions: [], changes: [] }), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        }),
      ),
    )
    let marked: string | undefined
    stubViewTransition()
    const router = makeRouter()
    await router.push(`/changes/${fixtureDetail.key}`)
    router.afterEach(() => {
      marked = returningScopeKey.value
    })

    await router.push('/')

    expect(marked).toBe(fixtureDetail.key)
    // Exactly one element may claim the name, so the marker does not outlive its gesture.
    await vi.waitFor(() => {
      expect(returningScopeKey.value).toBeUndefined()
    })
  })

  it('skips the hold and the transition entirely under reduced motion', async () => {
    reducedMotion(true)
    vi.stubGlobal('fetch', vi.fn())
    const start = stubViewTransition()
    const router = makeRouter()
    await router.push('/')

    await router.push(`/changes/${fixtureDetail.key}`)

    expect(start).not.toHaveBeenCalled()
    expect(fetch).not.toHaveBeenCalled()
    expect(router.currentRoute.value.name).toBe('change')
  })

  /* Artifact selection changes a query inside one route and owns its own paired-coordinate
     gesture. Wrapping it here would run two transitions over one navigation. */
  it('leaves artifact-query navigation on its own path', async () => {
    vi.stubGlobal('fetch', vi.fn())
    const start = stubViewTransition()
    const router = makeRouter()
    await router.push(`/changes/${fixtureDetail.key}`)

    await router.push(`/changes/${fixtureDetail.key}?artifact=proposal.md`)

    expect(start).not.toHaveBeenCalled()
    expect(fetch).not.toHaveBeenCalled()
  })
})
