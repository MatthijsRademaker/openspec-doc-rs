import { render, screen } from '@testing-library/vue'
import { describe, expect, it } from 'vitest'
import CommentThread from '@/components/review/CommentThread.vue'
import type { Thread } from '@/lib/scope-review'

function thread(status: Thread['status']): Thread {
  return {
    comment: {
      id: 'comment-1',
      body: 'Review this.',
      createdAt: '2026-08-07T10:00:00Z',
    },
    status,
    replies: [
      {
        id: 'reply-1',
        commentId: 'comment-1',
        author: 'agent',
        body: 'Changed.',
        createdAt: '2026-08-07T10:05:00Z',
      },
      {
        id: 'reply-2',
        commentId: 'comment-1',
        author: 'reviewer',
        body: 'Needs another pass.',
        createdAt: '2026-08-07T10:10:00Z',
      },
    ],
    anchorState: 'unanchored',
    blockId: null,
  }
}

describe('CommentThread', () => {
  it('attributes each persisted reply to its writer', () => {
    render(CommentThread, { props: { thread: thread('open') } })

    expect(screen.getAllByText('Reviewer')).toHaveLength(2)
    expect(screen.getByText('Agent')).toBeTruthy()
    expect(screen.getByText('Changed.').closest('.comment-message--agent')).toBeTruthy()
    expect(
      screen.getByText('Needs another pass.').closest('.comment-message--reviewer'),
    ).toBeTruthy()
  })

  it('offers reply and resolve for open comments', () => {
    render(CommentThread, { props: { thread: thread('open') } })

    expect(screen.getByRole('button', { name: 'Reply' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Resolve' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Reopen' })).toBeNull()
  })

  it('offers reopen but not resolve for resolved comments', () => {
    render(CommentThread, { props: { thread: thread('resolved') } })

    expect(screen.getByRole('button', { name: 'Reopen' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Resolve' })).toBeNull()
  })

  it('renders addressed as agent claim awaiting reviewer judgement', () => {
    render(CommentThread, { props: { thread: thread('addressed') } })

    expect(screen.getByText(/Agent claims work complete/)).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Accept as resolved' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Reopen' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: /addressed/i })).toBeNull()
  })
})
