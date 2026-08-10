/**
 * A trigger for the one live-update path no reviewer action reaches. It lives here rather than in
 * a Vue component so the whole mock lane stays inside `src/mocks/`, which the `import.meta.env.DEV`
 * gate in `main.ts` drops from the production bundle.
 */
function rewriteEndpoint(): string {
  const [, collection, key] = window.location.pathname.split('/')
  if ((collection !== 'changes' && collection !== 'sessions') || !key) {
    throw new Error(`Mock artifact rewrite needs a scope route, got ${window.location.pathname}`)
  }
  return `/api/${collection}/${key}/mock/artifact-rewrite`
}

async function rewriteSelectedArtifact(): Promise<void> {
  const endpoint = rewriteEndpoint()
  const artifactPath = new URLSearchParams(window.location.search).get('artifact')
  // Mock lane only: the endpoint is same-origin and served by the Service Worker.
  // nosemgrep: typescript.react.security.react-insecure-request.react-insecure-request
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ artifactPath }),
  })
  if (!response.ok) throw new Error(`Mock artifact rewrite failed: HTTP ${response.status}`)
}

export function mountMockLaneControls(): void {
  const trigger = document.createElement('button')
  trigger.type = 'button'
  trigger.textContent = 'Mock · rewrite artifact'
  trigger.style.cssText = [
    'position: fixed',
    'z-index: 40',
    'bottom: 0.5rem',
    'left: 0.5rem',
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
  trigger.addEventListener('click', () => {
    void rewriteSelectedArtifact()
  })
  document.body.append(trigger)
}
