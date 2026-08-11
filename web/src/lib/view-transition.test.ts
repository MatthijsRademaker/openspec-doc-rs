import { afterEach, describe, expect, it, vi } from 'vitest'
import { withViewTransition } from '@/lib/view-transition'

const originalStartViewTransition = document.startViewTransition

afterEach(() => {
  Object.defineProperty(document, 'startViewTransition', {
    configurable: true,
    value: originalStartViewTransition,
  })
  vi.unstubAllGlobals()
})

function reducedMotion(matches: boolean): void {
  vi.stubGlobal(
    'matchMedia',
    vi.fn(() => ({ matches })),
  )
}

describe('view transition navigation', () => {
  it('runs update unchanged when native transitions are unsupported', async () => {
    Object.defineProperty(document, 'startViewTransition', {
      configurable: true,
      value: undefined,
    })
    const update = vi.fn()

    await withViewTransition(update)

    expect(update).toHaveBeenCalledOnce()
  })

  it('uses native transition when motion is allowed', async () => {
    reducedMotion(false)
    const start = vi.fn((update: () => void | Promise<void>) => {
      const updateCallbackDone = Promise.resolve(update()).then(() => undefined)
      return { updateCallbackDone }
    })
    Object.defineProperty(document, 'startViewTransition', { configurable: true, value: start })
    const update = vi.fn()

    await withViewTransition(update)

    expect(start).toHaveBeenCalledOnce()
    expect(update).toHaveBeenCalledOnce()
  })

  it('skips native transition under reduced motion', async () => {
    reducedMotion(true)
    const start = vi.fn()
    Object.defineProperty(document, 'startViewTransition', { configurable: true, value: start })
    const update = vi.fn()

    await withViewTransition(update)

    expect(start).not.toHaveBeenCalled()
    expect(update).toHaveBeenCalledOnce()
  })
})
