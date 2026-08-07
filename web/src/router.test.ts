import { describe, expect, it } from 'vitest'
import { router } from '@/router'
import IndexView from '@/views/IndexView.vue'

describe('router', () => {
  it('exposes exactly the index route', () => {
    const routes = router.getRoutes()

    expect(routes.map((route) => route.path)).toEqual(['/'])
    expect(routes[0]?.components?.default).toBe(IndexView)
  })

  it('does not fake session or change routes before their views exist', () => {
    // The Rust server still owns these paths; registering them here would render
    // an empty shell under a URL that has real data today.
    expect(router.resolve('/sessions/some-session').matched).toHaveLength(0)
    expect(router.resolve('/changes/some-change').matched).toHaveLength(0)
  })
})
