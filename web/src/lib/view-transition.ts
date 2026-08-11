type StartViewTransition = (update: () => void | Promise<void>) => {
  updateCallbackDone: Promise<void>
}

export function prefersReducedMotion(): boolean {
  return (
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  )
}

/** Runs same update in every environment; native transition is presentation-only. */
export async function withViewTransition(update: () => void | Promise<void>): Promise<void> {
  const start = (document as unknown as { startViewTransition?: StartViewTransition })
    .startViewTransition
  if (!start || prefersReducedMotion()) {
    await update()
    return
  }

  const transition = start.call(document, update)
  await transition.updateCallbackDone
}
