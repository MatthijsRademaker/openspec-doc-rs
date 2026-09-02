import { nextTick, ref } from 'vue'
import type { RouteLocationNormalized, Router } from 'vue-router'
import { fetchScope, type ScopeDetail, type ScopeKind } from '@/lib/scope-review'
import { fetchIndex, type Index } from '@/lib/scopes'
import { prefersReducedMotion, viewTransitionStart } from '@/lib/view-transition'

/**
 * How long a navigation may be held so its gesture can terminate at a loaded destination instead
 * of at scaffolding.
 *
 * Measured rather than chosen: against the embedded binary serving a local project, `GET /api/index`
 * answers in 3–8ms and `GET /api/changes/<name>` in 7–12ms with the cold first request included.
 * 180ms is roughly fifteen times the slowest of those, so a local read always lands well inside it
 * and only a pathological one spends the ceiling — and a hold that does spend it still costs less
 * than the transition it was buying. If scope loads ever stop being reliably fast, the answer is to
 * abandon the hold and render the identity early, not to raise this.
 */
export const NAVIGATION_HOLD_CEILING_MS = 180

const SCOPE_ROUTES: Record<string, ScopeKind> = { session: 'session', change: 'change' }

interface ScopeTarget {
  kind: ScopeKind
  key: string
}

/**
 * The scope whose entry is the destination of a reverse gesture. Set while a scope-to-index
 * transition runs so the index entry being returned to carries the coordinate, cleared as soon as
 * it ends so exactly one element ever claims the name.
 */
export const returningScopeKey = ref<string>()

let heldIndex: Index | undefined
let heldScope: (ScopeTarget & { detail: ScopeDetail }) | undefined
let holdGeneration = 0

/** What the hold waited for is handed to the destination, so the wait buys no second round trip. */
export function takeHeldIndex(): Index | undefined {
  const held = heldIndex
  heldIndex = undefined
  return held
}

export function takeHeldScope(kind: ScopeKind, key: string): ScopeDetail | undefined {
  if (heldScope?.kind !== kind || heldScope.key !== key) return undefined

  const { detail } = heldScope
  heldScope = undefined
  return detail
}

/**
 * How long a speculative load stays spendable. Hover is what starts it and the click that follows
 * is what spends it, so the window only has to span the gap between two gestures of one hand.
 * Past it the entry is dropped and the hold fetches for itself: an agent rewrites this repository's
 * artifacts and review state while the reviewer reads them, so answering a click with a payload
 * read a minute ago would present a scope that has already moved.
 */
export const SPECULATION_TTL_MS = 10_000

interface Speculation {
  load: Promise<ScopeDetail>
  startedAt: number
}

const speculations = new Map<string, Speculation>()

function speculationId(kind: ScopeKind, key: string): string {
  return `${kind}:${key}`
}

function fresh(speculation: Speculation): boolean {
  return Date.now() - speculation.startedAt <= SPECULATION_TTL_MS
}

/**
 * Starts, on the gesture that precedes a click, the load that click would start. The hold spends
 * it, so a scope the reviewer pointed at before choosing it enters its transition without waiting
 * on a round trip first.
 *
 * Nothing here reports failure. A speculative load that fails is discarded, and the click that
 * follows fetches again — so the error reaches the reviewer through the destination's own failure
 * state rather than as an alarm about a page they never asked for.
 */
export function speculate(kind: ScopeKind, key: string): void {
  // Nothing else evicts: the reviewer sweeping a register of scopes must not leave a detail
  // payload per entry parked here for the life of the page.
  for (const [id, speculation] of speculations) {
    if (!fresh(speculation)) speculations.delete(id)
  }

  const id = speculationId(kind, key)
  if (speculations.has(id)) return

  const speculation: Speculation = { load: fetchScope(kind, key), startedAt: Date.now() }
  speculation.load.catch(() => {
    if (speculations.get(id) === speculation) speculations.delete(id)
  })
  speculations.set(id, speculation)
}

/** Spent once. A second click on the same scope reads the scope again rather than a stored answer. */
function takeSpeculation(kind: ScopeKind, key: string): Promise<ScopeDetail> | undefined {
  const id = speculationId(kind, key)
  const speculation = speculations.get(id)
  if (!speculation) return undefined

  speculations.delete(id)
  return fresh(speculation) ? speculation.load : undefined
}

function scopeTarget(route: RouteLocationNormalized): ScopeTarget | undefined {
  const kind = typeof route.name === 'string' ? SCOPE_ROUTES[route.name] : undefined
  if (!kind) return undefined

  const key = route.params.id ?? route.params.name
  return typeof key === 'string' && key ? { kind, key } : undefined
}

/** Index and scope are different pages; artifact selection changes a query within one of them. */
function isRouteChange(to: RouteLocationNormalized, from: RouteLocationNormalized): boolean {
  return from.name !== undefined && to.name !== from.name
}

async function hold(to: RouteLocationNormalized): Promise<void> {
  holdGeneration += 1
  const generation = holdGeneration
  const target = scopeTarget(to)
  const load = target
    ? (takeSpeculation(target.kind, target.key) ?? fetchScope(target.kind, target.key)).then(
        (detail) => {
          if (generation === holdGeneration) heldScope = { ...target, detail }
        },
      )
    : fetchIndex().then((index) => {
        if (generation === holdGeneration) heldIndex = index
      })

  let ceiling: ReturnType<typeof setTimeout> | undefined
  // A failed load ends the hold exactly as fast as a successful one, so the destination's own
  // failure state answers the click. A load that fails must never read as a click that did nothing.
  const elapsed = await Promise.race([
    load.then(() => false).catch(() => false),
    new Promise<boolean>((reach) => {
      ceiling = setTimeout(() => reach(true), NAVIGATION_HOLD_CEILING_MS)
    }),
  ])
  if (ceiling) clearTimeout(ceiling)
  // Past the ceiling the route commits without the gesture and the destination loads for itself;
  // invalidating the generation keeps a late arrival from being handed to some later navigation.
  if (elapsed) holdGeneration += 1
}

/** Resolves once the committed route has rendered, which is what the transition is waiting for. */
function rendered(router: Router): Promise<void> {
  return new Promise((resolve) => {
    const stop = router.afterEach(() => {
      stop()
      void nextTick().then(() => {
        resolve()
      })
    })
  })
}

/**
 * Index-to-scope and scope-to-index transitions are owned here rather than at the two link sites,
 * because a link site cannot see browser Back or Forward — wrapping links is what left history
 * navigation as an unanimated cut in both directions.
 */
export function installRouteTransitions(router: Router): void {
  router.beforeResolve(async (to, from) => {
    if (!isRouteChange(to, from)) return true

    returningScopeKey.value = undefined
    const start = viewTransitionStart()
    if (!start || prefersReducedMotion()) return true

    await hold(to)

    const departing = scopeTarget(from)
    if (departing && to.name === 'index') returningScopeKey.value = departing.key
    // The coordinate marker has to reach the DOM before the browser captures the outgoing page.
    await nextTick()

    return new Promise<true>((proceed) => {
      document.documentElement.dataset.viewTransition = 'route'
      const transition = start(() => {
        proceed(true)
        return rendered(router)
      })
      const release = () => {
        delete document.documentElement.dataset.viewTransition
        returningScopeKey.value = undefined
      }
      transition.finished.then(release, release)
    })
  })
}
