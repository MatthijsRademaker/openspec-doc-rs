import { describe, expect, it } from 'vitest'
import { router } from '@/router'
import IndexView from '@/views/IndexView.vue'
import ScopeView from '@/views/ScopeView.vue'

describe('router', () => {
  it('maps index, session, and change paths to implemented views', () => {
    const routes = router.getRoutes()

    expect(routes.map((route) => route.path).sort()).toEqual([
      '/',
      '/changes/:name',
      '/sessions/:id',
    ])
    expect(router.resolve('/').matched[0]?.components?.default).toBe(IndexView)
    expect(router.resolve('/sessions/some-session').matched[0]?.components?.default).toBe(ScopeView)
    expect(router.resolve('/changes/some-change').matched[0]?.components?.default).toBe(ScopeView)
  })
})
