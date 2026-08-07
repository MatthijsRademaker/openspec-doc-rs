/**
 * Where the Vite dev server sends `/api` requests. Development is same-origin,
 * like production: the browser talks to Vite, Vite forwards to the Rust server
 * a contributor started with `openspec-doc serve --port 8791 --no-open`.
 */

export const DEFAULT_API_PROXY_TARGET = 'http://127.0.0.1:8791'
export const API_PROXY_TARGET_ENV = 'OPENSPEC_DOC_API_PROXY_TARGET'

export function resolveApiProxyTarget(env: Record<string, string | undefined>): string {
  const override = env[API_PROXY_TARGET_ENV]
  if (override === undefined) {
    return DEFAULT_API_PROXY_TARGET
  }
  // An empty override is a mistyped shell export, not a request for the
  // default: silently falling back would point the proxy at a server the
  // contributor did not start.
  if (override.trim() === '') {
    throw new Error(`${API_PROXY_TARGET_ENV} is set but empty; unset it or name a server`)
  }

  let url: URL
  try {
    url = new URL(override)
  } catch {
    throw new Error(`${API_PROXY_TARGET_ENV} is not a URL: ${override}`)
  }
  if (url.protocol !== 'http:' && url.protocol !== 'https:') {
    throw new Error(`${API_PROXY_TARGET_ENV} must be an http(s) URL: ${override}`)
  }

  return override
}
