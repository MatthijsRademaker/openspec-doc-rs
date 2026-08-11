import { onScopeDispose, ref, type Ref } from 'vue'

/**
 * Arrival reports outlive their 160ms transition so reviewers can perceive them. They never block
 * content or interaction and clear themselves, keeping them events rather than durable state.
 */
export const EVENT_TRANSITION_MS = 160
export const EVENT_DWELL_MS = 2200

export interface LatestEventChannel<T> {
  event: Ref<T | undefined>
  signal: (event: T) => void
  clear: () => void
}

/** Owns one latest-event channel. New events replace older ones and disposal leaves no timer/state. */
export function createLatestEventChannel<T>(dwellMs = EVENT_DWELL_MS): LatestEventChannel<T> {
  const event = ref<T>() as Ref<T | undefined>
  let timer: ReturnType<typeof setTimeout> | undefined

  function clear(): void {
    if (timer) clearTimeout(timer)
    timer = undefined
    event.value = undefined
  }

  function signal(next: T): void {
    clear()
    event.value = next
    timer = setTimeout(clear, dwellMs)
  }

  onScopeDispose(clear)
  return { event, signal, clear }
}
