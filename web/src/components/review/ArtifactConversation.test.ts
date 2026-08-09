import { fireEvent, render, screen } from '@testing-library/vue'
import { describe, expect, it } from 'vitest'
import ArtifactConversation from '@/components/review/ArtifactConversation.vue'
import type { Artifact, Thread } from '@/lib/scope-review'

const artifact: Artifact = {
  path: 'openspec/changes/example/proposal.md',
  blocks: [
    { id: 'first', html: '<p>First</p>', source: 'First', range: { start: 0, end: 5 } },
    { id: 'second', html: '<p>Second</p>', source: 'Second', range: { start: 7, end: 13 } },
  ],
}

function thread(id: string, blockId: string, createdAt: string): Thread {
  return {
    comment: {
      id,
      anchor: {
        artifactPath: artifact.path,
        selectedText: blockId,
        headingPath: [],
        beforeText: '',
        afterText: '',
        startOffset: 0,
        endOffset: blockId.length,
      },
      body: `${id} body`,
      createdAt,
    },
    status: 'open',
    replies: [],
    anchorState: 'exact',
    blockId,
  }
}

describe('ArtifactConversation', () => {
  it('orders by source block then creation order and emits exact anchor activation', async () => {
    const threads = [
      thread('second-block', 'second', '2026-01-01T00:00:00Z'),
      thread('first-later', 'first', '2026-01-01T00:02:00Z'),
      thread('first-earlier', 'first', '2026-01-01T00:01:00Z'),
    ]
    const { emitted } = render(ArtifactConversation, { props: { artifact, threads } })

    expect(
      Array.from(document.querySelectorAll('.artifact-conversation__thread'), (element) =>
        element.getAttribute('data-thread-id'),
      ),
    ).toEqual(['first-earlier', 'first-later', 'second-block'])

    await fireEvent.click(
      screen.getByRole('button', { name: 'Locate source for comment first-earlier' }),
    )
    expect(emitted().activate?.[0]).toEqual(['first-earlier'])
  })

  it('renders honest empty instrument without fabricated state', () => {
    const { container } = render(ArtifactConversation, { props: { artifact, threads: [] } })

    expect(screen.getByText('No threads resolve to selected artifact.')).toBeTruthy()
    expect(container.textContent).not.toMatch(/activity|agent online|validation|complete/i)
  })
})
