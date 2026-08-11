/** Mock-only origins for remote artifact and review-state events. Production drops this module. */
function mockEndpoint(event: 'artifact-rewrite' | 'review-state'): string {
  const [, collection, key] = window.location.pathname.split('/')
  if ((collection !== 'changes' && collection !== 'sessions') || !key) {
    throw new Error(`Mock ${event} needs a scope route, got ${window.location.pathname}`)
  }
  return `/api/${collection}/${key}/mock/${event}`
}

async function triggerMockEvent(
  event: 'artifact-rewrite' | 'review-state',
  body?: unknown,
): Promise<void> {
  // Mock lane only: endpoint is same-origin and served by Service Worker.
  // nosemgrep: typescript.react.security.react-insecure-request.react-insecure-request
  const response = await fetch(mockEndpoint(event), {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body ?? {}),
  })
  if (!response.ok) throw new Error(`Mock ${event} failed: HTTP ${response.status}`)
}

function mockTrigger(label: string, left: string, action: () => Promise<void>): HTMLButtonElement {
  const trigger = document.createElement('button')
  trigger.type = 'button'
  trigger.textContent = label
  trigger.style.cssText = [
    'position: fixed',
    'z-index: 40',
    'bottom: 0.5rem',
    `left: ${left}`,
    'padding: 0.35rem 0.6rem',
    'border: 1px solid var(--delivery)',
    'background: var(--surface)',
    'color: var(--delivery)',
    'font-family: "IBM Plex Mono", ui-monospace, monospace',
    'font-size: 0.62rem',
    'letter-spacing: 0.06em',
    'text-transform: uppercase',
    'cursor: pointer',
  ].join(';')
  trigger.addEventListener('click', () => void action())
  return trigger
}

export function mountMockLaneControls(): void {
  document.body.append(
    mockTrigger('Mock · rewrite artifact', '0.5rem', () =>
      triggerMockEvent('artifact-rewrite', {
        artifactPath: new URLSearchParams(window.location.search).get('artifact'),
      }),
    ),
    mockTrigger('Mock · review state', '12rem', () => triggerMockEvent('review-state')),
  )
}
