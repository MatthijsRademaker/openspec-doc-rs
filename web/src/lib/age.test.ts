import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { age } from '@/lib/age'

const NOW = new Date('2026-08-06T12:00:00Z')

function ago(amount: number, unit: 's' | 'm' | 'h' | 'd'): string {
  const ms = { s: 1_000, m: 60_000, h: 3_600_000, d: 86_400_000 }[unit]
  return new Date(NOW.getTime() - amount * ms).toISOString()
}

describe('age', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(NOW)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('calls anything under a minute "just now"', () => {
    expect(age(ago(0, 's'))).toBe('just now')
    expect(age(ago(59, 's'))).toBe('just now')
  })

  it('calls a future timestamp "just now" rather than rendering a negative age', () => {
    expect(age(new Date(NOW.getTime() + 60_000).toISOString())).toBe('just now')
  })

  it('switches to minutes at one minute', () => {
    expect(age(ago(60, 's'))).toBe('1m ago')
    expect(age(ago(59, 'm'))).toBe('59m ago')
  })

  it('switches to hours at one hour', () => {
    expect(age(ago(60, 'm'))).toBe('1h ago')
    expect(age(ago(23, 'h'))).toBe('23h ago')
  })

  it('switches to days at one day', () => {
    expect(age(ago(24, 'h'))).toBe('1d ago')
    expect(age(ago(40, 'd'))).toBe('40d ago')
  })
})
