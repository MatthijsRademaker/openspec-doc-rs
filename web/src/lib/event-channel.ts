import { onScopeDispose, ref, type Ref } from 'vue'

/**
 * Arrival reports outlive their 160ms transition so reviewers can perceive them. They never block
 * content or interaction and clear themselves, keeping them events rather than durable state.
 */
export const EVENT_TRANSITION_MS = 160
/**
 * Covers the longest animation the acquired class gates, which is `--motion-duration-resolve`, not
 * the shorter `--motion-duration-acquire` beside it. A channel whose dwell is shorter than the
 * animation it gates removes the class mid-gesture and the animation is cut off at the dwell, so
 * the CSS duration reads as a value the interface never actually shows. These move together.
 */
export const EVENT_ACQUIRE_MS = 600
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
