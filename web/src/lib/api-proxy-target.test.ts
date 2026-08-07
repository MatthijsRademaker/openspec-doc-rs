import { describe, expect, it } from 'vitest'

import {
  API_PROXY_TARGET_ENV,
  DEFAULT_API_PROXY_TARGET,
  resolveApiProxyTarget,
} from '@/lib/api-proxy-target'

describe('resolveApiProxyTarget', () => {
  it('defaults to the documented development server', () => {
    expect(resolveApiProxyTarget({})).toBe(DEFAULT_API_PROXY_TARGET)
    expect(DEFAULT_API_PROXY_TARGET).toBe('http://127.0.0.1:8791')
  })

  it('honours an explicit override', () => {
    expect(resolveApiProxyTarget({ [API_PROXY_TARGET_ENV]: 'http://localhost:9000' })).toBe(
      'http://localhost:9000',
    )
  })

  it('refuses an empty override instead of silently defaulting', () => {
    expect(() => resolveApiProxyTarget({ [API_PROXY_TARGET_ENV]: '  ' })).toThrow(/empty/)
  })

  it('refuses a value without an http(s) scheme', () => {
    // `new URL` accepts `localhost:9000` as the non-special scheme `localhost:`,
    // so what rejects it is the protocol guard, not URL parsing.
    expect(() => resolveApiProxyTarget({ [API_PROXY_TARGET_ENV]: 'localhost:9000' })).toThrow(
      /http/,
    )
  })

  it('refuses a non-http URL', () => {
    expect(() => resolveApiProxyTarget({ [API_PROXY_TARGET_ENV]: 'file:///tmp/server' })).toThrow(
      /http/,
    )
  })
})
