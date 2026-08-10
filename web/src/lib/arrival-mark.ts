import { onScopeDispose, ref, type Ref } from 'vue'

/**
 * Deliberately longer than the 100–200ms transition envelope. The envelope bounds how long a
 * transition takes to finish and how long content is withheld; this is a report about content that
 * arrived without the reviewer acting, and a report that has gone in 200ms is a report only for
 * someone who was already looking at it. It withholds nothing and clears itself, so it stays an
 * event rather than becoming a state the interface has to remember to clear.
 */
export const ARRIVAL_DWELL_MS = 2200

export interface ArrivalMark {
  target: Ref<string | undefined>
  mark: (target: string) => void
  clear: () => void
}

export function createArrivalMark(): ArrivalMark {
  const target = ref<string>()
  let timer: ReturnType<typeof setTimeout> | undefined

  function clear(): void {
    if (timer) clearTimeout(timer)
    timer = undefined
    target.value = undefined
  }

  function mark(next: string): void {
    clear()
    target.value = next
    timer = setTimeout(clear, ARRIVAL_DWELL_MS)
  }

  onScopeDispose(clear)
  return { target, mark, clear }
}
