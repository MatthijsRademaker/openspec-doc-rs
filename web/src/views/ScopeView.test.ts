import { fireEvent, render, screen } from '@testing-library/vue'
import { reactive } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { ScopeDetail } from '@/lib/scope-review'
import ScopeView from '@/views/ScopeView.vue'

const route = reactive({ name: 'session', params: { id: 'session-with-a-long-exact-identifier' } })

vi.mock('vue-router', () => ({ useRoute: () => route }))

const SCOPE: ScopeDetail = {
  kind: 'session',
  key: 'session-with-a-long-exact-identifier',
  title: 'Exploring anchored review',
  artifacts: [
    {
      path: '.openspec-doc/scratch/_session/session-with-a-long-exact-identifier.md',
      blocks: [
        {
          id: 'block-0-12',
          html: '<h1>Exploration</h1>',
          source: '# Exploration',
          range: { start: 0, end: 13 },
        },
        {
          id: 'block-15-39',
          html: '<p>Review this exact block.</p>',
          source: 'Review this exact block.',
          range: { start: 15, end: 39 },
        },
      ],
    },
  ],
  comments: [
    {
      comment: {
        id: 'anchored',
        anchor: {
          artifactPath: '.openspec-doc/scratch/_session/session-with-a-long-exact-identifier.md',
          selectedText: 'Review this exact block.',
          headingPath: ['Exploration'],
          beforeText: '',
          afterText: '',
          startOffset: 15,
          endOffset: 39,
        },
        body: 'Make ownership explicit.',
        createdAt: '2026-08-07T10:00:00Z',
      },
      status: 'open',
      replies: [
        {
          id: 'reply-1',
          commentId: 'anchored',
          author: 'agent',
          body: 'Ownership now names agent and reviewer.',
          createdAt: '2026-08-07T10:05:00Z',
        },
      ],
      anchorState: 'fuzzy',
      blockId: 'block-15-39',
    },
    {
      comment: {
        id: 'orphaned',
        anchor: {
          artifactPath: '.openspec-doc/scratch/_session/session-with-a-long-exact-identifier.md',
          selectedText: 'Text rewritten away.',
          headingPath: ['Exploration'],
          beforeText: '',
          afterText: '',
          startOffset: 45,
          endOffset: 65,
        },
        body: 'This concern must remain reachable.',
        createdAt: '2026-08-07T09:00:00Z',
      },
      status: 'resolved',
      replies: [],
      anchorState: 'orphaned',
      blockId: null,
    },
  ],
  commentCounts: { open: 1, addressed: 0, resolved: 1 },
  verdicts: [
    {
      id: 'verdict-1',
      verdict: 'keep-exploring',
      notes: '',
      createdAt: '2026-08-07T11:00:00Z',
    },
  ],
  standingVerdict: {
    id: 'verdict-1',
    verdict: 'keep-exploring',
    createdAt: '2026-08-07T11:00:00Z',
    directiveDelivered: false,
    directivePending: true,
  },
}

function stubScope(scope: ScopeDetail = SCOPE) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue(
      new Response(JSON.stringify(scope), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    ),
  )
}

afterEach(() => vi.unstubAllGlobals())

describe('ScopeView', () => {
  it('keeps document primary and expands anchored conversation beside its block', async () => {
    stubScope()
    render(ScopeView)

    expect(await screen.findByRole('heading', { name: 'Exploring anchored review' })).toBeTruthy()
    expect(screen.getByText('Review this exact block.')).toBeTruthy()
    expect(screen.getByText('1 open')).toBeTruthy()
    expect(screen.getByText('Awaiting agent')).toBeTruthy()

    await fireEvent.click(screen.getByRole('button', { name: '1 open comment' }))

    expect(screen.getByText('Make ownership explicit.')).toBeTruthy()
    expect(screen.getByText('Ownership now names agent and reviewer.')).toBeTruthy()
    expect(screen.getByText('Reviewer')).toBeTruthy()
    expect(screen.getByText('Agent')).toBeTruthy()
    expect(screen.getByText(/Anchor moved/)).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Reply' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Resolve' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: /addressed/i })).toBeNull()
  })

  it('keeps orphaned comments reachable behind plus with original quote', async () => {
    stubScope()
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploring anchored review' })

    await fireEvent.click(screen.getByRole('button', { name: /Comment \/ 1 without block/i }))

    expect(screen.getByText('This concern must remain reachable.')).toBeTruthy()
    expect(screen.getByText('Anchor lost')).toBeTruthy()
    expect(screen.getByText('Text rewritten away.')).toBeTruthy()
    expect(screen.getByLabelText('Optional scope comment')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Keep exploring' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Move to proposal' })).toBeTruthy()
  })

  it('renders successful empty artifact state instead of hiding scope', async () => {
    stubScope({
      ...SCOPE,
      artifacts: [],
      comments: [],
      commentCounts: { open: 0, addressed: 0, resolved: 0 },
    })
    render(ScopeView)

    expect(await screen.findByRole('heading', { name: 'No artifacts written yet' })).toBeTruthy()
    expect(screen.getByText(/Document spine will appear/)).toBeTruthy()
  })
})
