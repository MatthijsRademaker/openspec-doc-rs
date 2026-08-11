import { effectScope, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { createLatestEventChannel, EVENT_DWELL_MS } from '@/lib/event-channel'

afterEach(() => {
  vi.useRealTimers()
})

describe('latest event channel', () => {
  it('replaces same-channel events and restarts dwell cleanup', async () => {
    vi.useFakeTimers()
    const scope = effectScope()
    const channel = scope.run(() => createLatestEventChannel<string>())
    if (!channel) throw new Error('effect scope did not create event channel')

    channel.signal('first')
    vi.advanceTimersByTime(EVENT_DWELL_MS - 1)
    channel.signal('latest')
    vi.advanceTimersByTime(1)
    await nextTick()
    expect(channel.event.value).toBe('latest')

    vi.advanceTimersByTime(EVENT_DWELL_MS)
    await nextTick()
    expect(channel.event.value).toBeUndefined()
    expect(vi.getTimerCount()).toBe(0)
    scope.stop()
  })

  it('keeps channels independent', () => {
    vi.useFakeTimers()
    const scope = effectScope()
    const channels = scope.run(() => ({
      document: createLatestEventChannel<string>(),
      reviewer: createLatestEventChannel<string>(),
    }))
    if (!channels) throw new Error('effect scope did not create event channels')

    channels.document.signal('artifact-content')
    channels.reviewer.signal('thread-1')
    channels.document.clear()

    expect(channels.document.event.value).toBeUndefined()
    expect(channels.reviewer.event.value).toBe('thread-1')
    scope.stop()
  })

  it('clears pending events and timers on route-scope disposal', async () => {
    vi.useFakeTimers()
    const scope = effectScope()
    const channel = scope.run(() => createLatestEventChannel<string>())
    if (!channel) throw new Error('effect scope did not create event channel')

    channel.signal('in-flight')
    expect(vi.getTimerCount()).toBe(1)
    scope.stop()
    await nextTick()

    expect(channel.event.value).toBeUndefined()
    expect(vi.getTimerCount()).toBe(0)
  })
})
