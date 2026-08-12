interface ViewTransition {
  updateCallbackDone: Promise<void>
  finished: Promise<void>
}

type StartViewTransition = (update: () => void | Promise<void>) => ViewTransition

export function prefersReducedMotion(): boolean {
  return (
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  )
}

/** The native entry point, bound, or nothing where the browser has none. */
export function viewTransitionStart(): StartViewTransition | undefined {
  const start = (document as unknown as { startViewTransition?: StartViewTransition })
    .startViewTransition
  return start ? start.bind(document) : undefined
}

/** Runs same update in every environment; native transition is presentation-only. */
export async function withViewTransition(update: () => void | Promise<void>): Promise<void> {
  const start = viewTransitionStart()
  if (!start || prefersReducedMotion()) {
    await update()
    return
  }

  await start(update).updateCallbackDone
}
