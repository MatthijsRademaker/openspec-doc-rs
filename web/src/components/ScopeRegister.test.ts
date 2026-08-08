import { render, screen } from '@testing-library/vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import ScopeRegister from '@/components/ScopeRegister.vue'
import type { Scope } from '@/lib/scopes'

const NOW = new Date('2026-08-06T12:00:00Z')

function scope(overrides: Partial<Scope> = {}): Scope {
  return {
    key: 'implement-observatory-design-system-with-a-realistically-long-identifier',
    title: null,
    modifiedAt: null,
    openComments: 0,
    verdict: null,
    mostRecentlyActive: false,
    ...overrides,
  }
}

describe('ScopeRegister', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(NOW)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('renders a named empty instrument', () => {
    render(ScopeRegister, { props: { scopes: [], prefix: 'changes', title: 'Changes' } })

    expect(screen.getByRole('heading', { name: 'Changes' })).toBeTruthy()
    expect(screen.getByText('No changes discovered.')).toBeTruthy()
    expect(screen.getByText('0 scopes')).toBeTruthy()
  })

  it('keeps titled and promoted identities separate from exact keys', () => {
    const item = scope({ title: 'Promoted observatory change' })
    render(ScopeRegister, { props: { scopes: [item], prefix: 'changes', title: 'Changes' } })

    const link = screen.getByRole('link', { name: 'Promoted observatory change' })
    expect(link.getAttribute('href')).toBe(`/changes/${item.key}`)
    expect(screen.getByText(item.key)).toBeTruthy()
    expect(link.classList.contains('scope-entry__link--identifier')).toBe(false)
  })

  it('renders one operational identity for an untitled session', () => {
    const item = scope()
    render(ScopeRegister, { props: { scopes: [item], prefix: 'sessions', title: 'Sessions' } })

    const link = screen.getByRole('link', { name: item.key })
    expect(link.getAttribute('href')).toBe(`/sessions/${item.key}`)
    expect(link.classList).toContain('scope-entry__link--identifier')
    expect(screen.getAllByText(item.key)).toHaveLength(1)
  })

  it('keeps modified time, comment count, verdict, and recent marker visible', () => {
    render(ScopeRegister, {
      props: {
        scopes: [
          scope({
            title: 'Observatory design system',
            modifiedAt: '2026-08-06T10:00:00Z',
            openComments: 3,
            verdict: 'comment-resolution',
            mostRecentlyActive: true,
          }),
        ],
        prefix: 'changes',
        title: 'Changes',
      },
    })

    expect(screen.getByText('2h ago')).toBeTruthy()
    expect(screen.getByText('3 open')).toBeTruthy()
    expect(screen.getByText('comment-resolution')).toBeTruthy()
    expect(screen.getByText('most recently active')).toBeTruthy()
    expect(screen.getByText('◇')).toBeTruthy()
    expect(screen.getByText('◆')).toBeTruthy()
  })

  it('renders explicit zero and dashes instead of fabricated values', () => {
    render(ScopeRegister, {
      props: { scopes: [scope()], prefix: 'changes', title: 'Changes' },
    })

    expect(screen.getAllByText('—')).toHaveLength(2)
    expect(screen.getByText('0')).toBeTruthy()
    expect(screen.queryByText('most recently active')).toBeNull()
  })
})
