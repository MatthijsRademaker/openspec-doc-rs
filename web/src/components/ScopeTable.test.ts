import { render, screen } from '@testing-library/vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import ScopeTable from '@/components/ScopeTable.vue'
import type { Scope } from '@/lib/scopes'

const NOW = new Date('2026-08-06T12:00:00Z')

function scope(overrides: Partial<Scope> = {}): Scope {
  return {
    key: 'add-dashboard-development-harness',
    title: null,
    modifiedAt: null,
    openComments: 0,
    verdict: null,
    mostRecentlyActive: false,
    ...overrides,
  }
}

describe('ScopeTable', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(NOW)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('says so when there is nothing to list', () => {
    render(ScopeTable, { props: { scopes: [], prefix: 'changes' } })

    expect(screen.getByText('None discovered.')).toBeTruthy()
  })

  it('links a scope by its key while showing its title', () => {
    render(ScopeTable, {
      props: { scopes: [scope({ title: 'Development harness' })], prefix: 'changes' },
    })

    const link = screen.getByRole('link', { name: 'Development harness' })
    expect(link.getAttribute('href')).toBe('/changes/add-dashboard-development-harness')
    expect(screen.getByText('add-dashboard-development-harness')).toBeTruthy()
  })

  it('falls back to the key when a scope has no title', () => {
    render(ScopeTable, { props: { scopes: [scope()], prefix: 'sessions' } })

    const links = screen.getAllByRole('link', { name: 'add-dashboard-development-harness' })
    expect(links[0]?.getAttribute('href')).toBe('/sessions/add-dashboard-development-harness')
  })

  it('renders activity, comment count, and verdict only when they exist', () => {
    render(ScopeTable, {
      props: {
        scopes: [
          scope({
            openComments: 3,
            verdict: 'comment-resolution',
            mostRecentlyActive: true,
            modifiedAt: '2026-08-06T10:00:00Z',
          }),
        ],
        prefix: 'changes',
      },
    })

    expect(screen.getByText('most recently active')).toBeTruthy()
    expect(screen.getByText('3')).toBeTruthy()
    expect(screen.getByText('comment-resolution')).toBeTruthy()
    expect(screen.getByText('2h ago')).toBeTruthy()
  })

  it('renders dashes rather than fake values for absent data', () => {
    render(ScopeTable, { props: { scopes: [scope()], prefix: 'changes' } })

    // No modifiedAt and no verdict are both '—'; zero open comments is a real 0.
    expect(screen.getAllByText('—')).toHaveLength(2)
    expect(screen.getByText('0')).toBeTruthy()
    expect(screen.queryByText('most recently active')).toBeNull()
  })
})
