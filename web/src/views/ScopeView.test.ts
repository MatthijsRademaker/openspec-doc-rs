import { fireEvent, render, screen, waitFor } from '@testing-library/vue'
import { nextTick, reactive } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { EVENT_DWELL_MS, EVENT_TRANSITION_MS } from '@/lib/event-channel'
import type { ApprovalState, Artifact, ScopeDetail } from '@/lib/scope-review'
import ScopeView from '@/views/ScopeView.vue'

const route = reactive<{
  name: string
  params: Record<string, string>
  query: Record<string, string | string[] | undefined>
}>({
  name: 'session',
  params: { id: 'session-with-a-long-exact-identifier' },
  query: {},
})

const replace = vi.fn(
  async (location: { query?: Record<string, string | string[] | undefined> }) => {
    route.query = location.query ?? {}
  },
)
const push = vi.fn(async (location: { query?: Record<string, string | string[] | undefined> }) => {
  route.query = location.query ?? {}
})

vi.mock('vue-router', () => ({
  useRoute: () => route,
  useRouter: () => ({ replace, push }),
  RouterLink: {
    props: ['to'],
    template: '<a :href="to"><slot /></a>',
  },
}))

const SESSION_PATH = '.openspec-doc/scratch/_session/session-with-a-long-exact-identifier.md'
const SESSION_DISPLAY_PATH = 'session-with-a-long-exact-identifier.md'
const CHANGE_ROOT = 'openspec/changes/change-with-a-long-exact-identifier'
const PROPOSAL_PATH = `${CHANGE_ROOT}/proposal.md`
const DESIGN_PATH = `${CHANGE_ROOT}/design.md`
const TASKS_PATH = `${CHANGE_ROOT}/tasks.md`
const HTML_SPEC_PATH = `${CHANGE_ROOT}/specs/dashboard-html-views/spec.md`
const VISUAL_SPEC_PATH = `${CHANGE_ROOT}/specs/dashboard-visual-system/spec.md`

function artifact(path: string, prefix: string, text: string): Artifact {
  return {
    path,
    blocks: [
      {
        id: `${prefix}-heading`,
        html: `<h1>${text}</h1>`,
        source: `# ${text}`,
        range: { start: 0, end: text.length + 2 },
      },
      {
        id: `${prefix}-body`,
        html: `<p>${text} body.</p>`,
        source: `${text} body.`,
        range: { start: text.length + 4, end: text.length * 2 + 10 },
      },
    ],
  }
}

const SESSION_SCOPE: ScopeDetail = {
  kind: 'session',
  key: 'session-with-a-long-exact-identifier',
  title: 'Exploring anchored review',
  artifacts: [artifact(SESSION_PATH, 'session', 'Exploration')],
  comments: [
    {
      comment: {
        id: 'anchored',
        anchor: {
          artifactPath: SESSION_PATH,
          selectedText: 'Exploration body.',
          headingPath: ['Exploration'],
          beforeText: '',
          afterText: '',
          startOffset: 15,
          endOffset: 32,
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
      blockId: 'session-body',
    },
    {
      comment: {
        id: 'orphaned',
        anchor: {
          artifactPath: SESSION_PATH,
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
  // An exploration is not a change and has nothing to approve.
  approval: null,
}

const CHANGE_ARTIFACTS = [
  {
    path: PROPOSAL_PATH,
    blocks: [
      {
        id: 'proposal-heading',
        html: '<h1>Proposal coordinate</h1>',
        source: '# Proposal coordinate',
        range: { start: 0, end: 21 },
      },
      {
        id: 'proposal-repeat-1',
        html: '<p>Repeated review target.</p>',
        source: 'Repeated review target.',
        range: { start: 23, end: 46 },
      },
      {
        id: 'proposal-repeat-2',
        html: '<p>Repeated review target.</p>',
        source: 'Repeated review target.',
        range: { start: 48, end: 71 },
      },
    ],
  },
  artifact(DESIGN_PATH, 'design', 'Design coordinate'),
  artifact(TASKS_PATH, 'tasks', 'Tasks coordinate'),
  artifact(HTML_SPEC_PATH, 'html-spec', 'HTML capability coordinate'),
  artifact(VISUAL_SPEC_PATH, 'visual-spec', 'Visual capability coordinate'),
]

const ORPHANED_THREAD = SESSION_SCOPE.comments[1]
if (!ORPHANED_THREAD) throw new Error('session fixture must contain orphaned thread')

const NOT_APPROVED: ApprovalState = {
  state: 'not-approved',
  reason: 'no approval has been recorded for this change',
  changedArtifacts: [],
}

const CHANGE_SCOPE: ScopeDetail = {
  ...SESSION_SCOPE,
  approval: NOT_APPROVED,
  kind: 'change',
  key: 'change-with-a-long-exact-identifier',
  title: 'Selected artifact review',
  artifacts: CHANGE_ARTIFACTS,
  comments: [
    {
      comment: {
        id: 'proposal-thread',
        anchor: {
          artifactPath: PROPOSAL_PATH,
          selectedText: 'Repeated review target.',
          headingPath: ['Proposal coordinate'],
          beforeText: '',
          afterText: '',
          startOffset: 48,
          endOffset: 71,
        },
        body: 'Proposal thread body.',
        createdAt: '2026-08-07T10:00:00Z',
      },
      status: 'open',
      replies: [],
      anchorState: 'exact',
      blockId: 'proposal-repeat-2',
    },
    {
      comment: {
        id: 'design-thread',
        anchor: {
          artifactPath: DESIGN_PATH,
          selectedText: 'Design coordinate body.',
          headingPath: ['Design coordinate'],
          beforeText: '',
          afterText: '',
          startOffset: 21,
          endOffset: 44,
        },
        body: 'Design thread body.',
        createdAt: '2026-08-07T10:01:00Z',
      },
      status: 'addressed',
      replies: [],
      anchorState: 'fuzzy',
      blockId: 'design-body',
    },
    ORPHANED_THREAD,
  ],
  commentCounts: { open: 1, addressed: 1, resolved: 1 },
  verdicts: [
    {
      id: 'change-verdict',
      verdict: 'comment-resolution',
      notes: '',
      createdAt: '2026-08-07T11:00:00Z',
    },
  ],
  standingVerdict: {
    id: 'change-verdict',
    verdict: 'comment-resolution',
    createdAt: '2026-08-07T11:00:00Z',
    directiveDelivered: true,
    directivePending: false,
  },
}

function response(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  })
}

function stubScope(scope: ScopeDetail = SESSION_SCOPE) {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response(scope)))
}

class StubEventSource {
  static instances: StubEventSource[] = []
  onmessage: ((event: MessageEvent<string>) => void) | null = null
  onerror: (() => void) | null = null
  readonly url: string

  constructor(url: string) {
    this.url = url
    StubEventSource.instances.push(this)
  }

  close() {}

  emit(data: string) {
    this.onmessage?.({ data } as MessageEvent<string>)
  }
}

function stubLiveScope(initial: ScopeDetail = SESSION_SCOPE) {
  // A fresh Response per call: a shared one is unusable once its body has been read, and a
  // mutation followed by a review-state refresh reads twice.
  const fetchMock = vi.fn(() => Promise.resolve(response(initial)))
  vi.stubGlobal('fetch', fetchMock)
  vi.stubGlobal('EventSource', StubEventSource)
  return fetchMock
}

function useChange(query: Record<string, string | string[] | undefined> = {}) {
  route.name = 'change'
  route.params = { name: CHANGE_SCOPE.key }
  route.query = query
}

Object.defineProperty(HTMLElement.prototype, 'scrollIntoView', {
  configurable: true,
  value: vi.fn(),
})

const originalStartViewTransition = document.startViewTransition

afterEach(() => {
  route.name = 'session'
  route.params = { id: SESSION_SCOPE.key }
  route.query = {}
  replace.mockClear()
  push.mockClear()
  StubEventSource.instances = []
  Object.defineProperty(window, 'scrollY', { configurable: true, value: 0 })
  Object.defineProperty(document, 'startViewTransition', {
    configurable: true,
    value: originalStartViewTransition,
  })
  vi.unstubAllGlobals()
})

describe('ScopeView selected-artifact workbench', () => {
  it('reports exact route coordinate while scope data is unresolved', () => {
    useChange({ artifact: PROPOSAL_PATH })
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))

    const { container } = render(ScopeView)

    expect(screen.getByRole('heading', { name: 'Acquiring route coordinate' })).toBeTruthy()
    expect(screen.getByText(CHANGE_SCOPE.key)).toBeTruthy()
    expect(container.querySelector('[data-motion-event="acquire"]')).toBeTruthy()
    expect(screen.queryByText(CHANGE_SCOPE.title ?? '')).toBeNull()
  })

  it('canonicalizes unqualified change to first server artifact with replace', async () => {
    useChange()
    stubScope(CHANGE_SCOPE)
    render(ScopeView)

    expect(await screen.findByRole('heading', { name: 'Proposal coordinate' })).toBeTruthy()
    await waitFor(() =>
      expect(replace).toHaveBeenCalledWith({ query: { artifact: PROPOSAL_PATH } }),
    )
    expect(push).not.toHaveBeenCalled()
    expect(screen.queryByRole('heading', { name: 'Design coordinate' })).toBeNull()
    expect(document.querySelectorAll('.artifact-navigator__path')).toHaveLength(5)
  })

  it('loads exact nested artifact query directly without canonical replacement', async () => {
    useChange({ artifact: HTML_SPEC_PATH })
    stubScope(CHANGE_SCOPE)
    render(ScopeView)

    expect(await screen.findByRole('heading', { name: 'HTML capability coordinate' })).toBeTruthy()
    expect(screen.queryByRole('heading', { name: 'Proposal coordinate' })).toBeNull()
    expect(replace).not.toHaveBeenCalled()
    expect(
      document.querySelector('.artifact-navigator__path[aria-current="page"]')?.textContent,
    ).toContain('specs/dashboard-html-views/spec.md')
  })

  it('pushes exact selections so Router history can restore each query', async () => {
    useChange({ artifact: PROPOSAL_PATH })
    stubScope(CHANGE_SCOPE)
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Proposal coordinate' })

    await fireEvent.click(screen.getByRole('button', { name: /design\.md/ }))
    expect(push).toHaveBeenLastCalledWith({ query: { artifact: DESIGN_PATH } })
    expect(await screen.findByRole('heading', { name: 'Design coordinate' })).toBeTruthy()
    expect(document.activeElement).toBe(document.getElementById('selected-artifact-title'))
    expect(document.querySelector('.artifact-document--acquired')).toBeTruthy()
    expect(document.querySelector('.artifact-document__coordinate-lock')).toBeTruthy()
    expect(document.querySelector('.artifact-document')?.getAttribute('data-motion-event')).toBe(
      'acquire',
    )

    await fireEvent.click(screen.getByRole('button', { name: /tasks\.md/ }))
    expect(push).toHaveBeenLastCalledWith({ query: { artifact: TASKS_PATH } })
    expect(await screen.findByRole('heading', { name: 'Tasks coordinate' })).toBeTruthy()

    route.query = { artifact: DESIGN_PATH }
    expect(await screen.findByRole('heading', { name: 'Design coordinate' })).toBeTruthy()
  })

  it('flushes clicked coordinate before native transition captures source pixels', async () => {
    useChange({ artifact: PROPOSAL_PATH })
    stubScope(CHANGE_SCOPE)
    let capturedSource: { path: string; event: string | null; lock: string | null } | undefined
    const start = vi.fn((update: () => void | Promise<void>) => {
      const source = document.querySelector('.artifact-navigator__path--acquiring')
      capturedSource = {
        path: source?.querySelector('.artifact-navigator__exact')?.textContent?.trim() ?? '',
        event: source?.getAttribute('data-motion-event') ?? null,
        lock: source?.querySelector('.artifact-navigator__lock')?.textContent?.trim() ?? null,
      }
      const updateCallbackDone = (async () => {
        await update()
      })()
      return { updateCallbackDone }
    })
    Object.defineProperty(document, 'startViewTransition', { configurable: true, value: start })
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Proposal coordinate' })

    await fireEvent.click(screen.getByRole('button', { name: /design\.md/ }))

    await waitFor(() => expect(start).toHaveBeenCalledOnce())
    expect(capturedSource).toEqual({ path: 'design.md', event: 'acquire', lock: '◇' })
    expect(await screen.findByRole('heading', { name: 'Design coordinate' })).toBeTruthy()
    expect(document.querySelector('.artifact-document__coordinate-lock')).toBeTruthy()
  })

  it('keeps invalid exact query, scope identity, and every available path visible', async () => {
    const missing = `${CHANGE_ROOT}/specs/missing/spec.md`
    useChange({ artifact: missing })
    stubScope(CHANGE_SCOPE)
    render(ScopeView)

    expect(
      await screen.findByRole('heading', { name: 'Selected artifact unavailable' }),
    ).toBeTruthy()
    expect(screen.getByText(missing)).toBeTruthy()
    expect(screen.getByRole('heading', { name: CHANGE_SCOPE.title ?? '' })).toBeTruthy()
    expect(screen.getAllByRole('button', { name: /\.md/ })).toHaveLength(5)
    expect(screen.queryByRole('heading', { name: 'Proposal coordinate' })).toBeNull()
    expect(replace).not.toHaveBeenCalled()
  })

  it('preserves selected query when live update removes that artifact', async () => {
    useChange({ artifact: DESIGN_PATH })
    const fetchMock = stubLiveScope(CHANGE_SCOPE)
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Design coordinate' })

    fetchMock.mockResolvedValueOnce(
      response({
        ...CHANGE_SCOPE,
        artifacts: CHANGE_SCOPE.artifacts.filter((candidate) => candidate.path !== DESIGN_PATH),
      }),
    )
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: true, reviewStateChanged: false }),
    )

    expect(
      await screen.findByRole('heading', { name: 'Selected artifact unavailable' }),
    ).toBeTruthy()
    expect(route.query.artifact).toBe(DESIGN_PATH)
    expect(screen.getByRole('button', { name: /proposal\.md/ })).toBeTruthy()
    expect(replace).not.toHaveBeenCalled()
  })

  it('auto-selects sole session scratch artifact without clickable artifact destinations', async () => {
    stubScope()
    const { container } = render(ScopeView)

    expect(await screen.findByRole('heading', { name: 'Exploration' })).toBeTruthy()
    expect(screen.getAllByText(SESSION_DISPLAY_PATH).length).toBeGreaterThan(0)
    expect(container.querySelector('.artifact-navigator--identity')).toBeTruthy()
    expect(screen.queryByRole('navigation', { name: 'Artifacts' })).toBeNull()
    expect(replace).not.toHaveBeenCalled()
    expect(push).not.toHaveBeenCalled()
  })

  it('shows only selected-artifact conversation while retaining global counts', async () => {
    useChange({ artifact: PROPOSAL_PATH })
    stubScope(CHANGE_SCOPE)
    render(ScopeView)

    expect(await screen.findByText('Proposal thread body.')).toBeTruthy()
    expect(screen.queryByText('Design thread body.')).toBeNull()
    expect(screen.getByText('1 open')).toBeTruthy()
    expect(screen.getByText('1 addressed')).toBeTruthy()
    expect(screen.getByText('1 resolved')).toBeTruthy()

    const collapse = screen.getByRole('button', { name: 'Collapse anchored threads' })
    await fireEvent.click(collapse)
    expect(screen.getByRole('button', { name: 'Expand anchored threads' })).toBeTruthy()
    expect(document.querySelector('.scope-conversation--reconfiguring')).toBeTruthy()
    expect(document.querySelector('.scope-conversation')?.getAttribute('data-motion-event')).toBe(
      'reconfigure',
    )
    expect(document.getElementById('scope-conversation-body')?.style.display).toBe('none')
    await fireEvent.click(screen.getByRole('button', { name: 'Expand anchored threads' }))
    expect(document.getElementById('scope-conversation-body')?.style.display).not.toBe('none')

    await fireEvent.click(screen.getByRole('button', { name: /design\.md/ }))
    expect(await screen.findByText('Design thread body.')).toBeTruthy()
    expect(screen.queryByText('Proposal thread body.')).toBeNull()
  })

  it('links marker and thread to exact repeated source block', async () => {
    useChange({ artifact: PROPOSAL_PATH })
    stubScope(CHANGE_SCOPE)
    const { container } = render(ScopeView)
    await screen.findByText('Proposal thread body.')

    const repeated = container.querySelectorAll('.review-block[data-block-id^="proposal-repeat"]')
    expect(repeated).toHaveLength(2)
    expect(repeated[0]?.querySelector('.review-block__marker')).toBeNull()
    const marker = repeated[1]?.querySelector('.review-block__marker')
    expect(marker).toBeTruthy()
    await fireEvent.click(marker as HTMLButtonElement)
    await waitFor(() => expect(repeated[1]?.classList.contains('review-block--active')).toBe(true))
    // The pane the reviewer did not click is the one that used to change by almost nothing.
    const railThread = document.querySelector('.artifact-conversation__thread--active')
    expect(railThread?.getAttribute('id')).toBe('artifact-thread-proposal-thread')
    expect(railThread?.classList).toContain(
      'artifact-conversation__thread--triangulation-destination',
    )
    expect(repeated[1]?.classList).toContain('review-block--triangulation-origin')
    expect(railThread?.querySelector('.artifact-conversation__crosshair-lock')).toBeTruthy()
    expect(container.querySelector('.scope-layout__direction-trace')).toBeTruthy()

    await fireEvent.click(
      screen.getByRole('button', { name: 'Locate source for comment proposal-thread' }),
    )
    expect(document.activeElement).toBe(repeated[1])
    expect(repeated[1]?.classList).toContain('review-block--triangulation-destination')
    expect(railThread?.classList).toContain('artifact-conversation__thread--triangulation-origin')
  })

  it('clears transient triangulation while preserving settled active exactness', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true })
    useChange({ artifact: PROPOSAL_PATH })
    stubScope(CHANGE_SCOPE)
    const { container } = render(ScopeView)
    await screen.findByText('Proposal thread body.')

    await fireEvent.click(container.querySelector('.review-block__marker') as HTMLButtonElement)
    expect(container.querySelector('[data-motion-event="triangulate"]')).toBeTruthy()

    vi.advanceTimersByTime(EVENT_TRANSITION_MS + 1)
    await nextTick()
    expect(container.querySelector('[data-motion-event="triangulate"]')).toBeNull()
    expect(container.querySelector('.scope-layout__direction-trace')).toBeNull()
    expect(container.querySelector('.review-block--active')).toBeTruthy()
    expect(container.querySelector('.artifact-conversation__thread--active')).toBeTruthy()
    vi.useRealTimers()
  })

  it('uses immediate scroll while preserving paired locks under reduced motion', async () => {
    useChange({ artifact: PROPOSAL_PATH })
    vi.stubGlobal(
      'matchMedia',
      vi.fn(() => ({ matches: true })),
    )
    stubScope(CHANGE_SCOPE)
    const { container } = render(ScopeView)
    await screen.findByText('Proposal thread body.')
    const scrollIntoView = vi.mocked(HTMLElement.prototype.scrollIntoView)
    scrollIntoView.mockClear()

    await fireEvent.click(container.querySelector('.review-block__marker') as HTMLButtonElement)

    expect(scrollIntoView).toHaveBeenCalledWith({ block: 'center', behavior: 'auto' })
    expect(container.querySelector('.review-block--active')).toBeTruthy()
    expect(container.querySelector('.artifact-conversation__thread--active')).toBeTruthy()
  })

  it('locks the document block when activation comes from the conversation pane', async () => {
    useChange({ artifact: PROPOSAL_PATH })
    stubScope(CHANGE_SCOPE)
    const { container } = render(ScopeView)
    await screen.findByText('Proposal thread body.')
    expect(container.querySelector('.review-block--active')).toBeNull()

    await fireEvent.click(
      screen.getByRole('button', { name: 'Locate source for comment proposal-thread' }),
    )

    // Activation is symmetric: the pane the reviewer did not click has to change either way.
    await waitFor(() => expect(container.querySelectorAll('.review-block--active')).toHaveLength(1))
    expect(container.querySelector('.review-block--active')?.getAttribute('data-block-id')).toBe(
      'proposal-repeat-2',
    )
    expect(document.querySelector('.artifact-conversation__thread--active')).toBeTruthy()
  })

  it('keeps orphaned comments, standing verdict, delivery, history, and session actions', async () => {
    stubScope()
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    expect(screen.getAllByText('keep-exploring').length).toBeGreaterThan(0)
    expect(screen.getByText('Awaiting agent')).toBeTruthy()
    expect(screen.getByText('Verdict history / 1')).toBeTruthy()
    await fireEvent.click(
      screen.getByRole('button', { name: 'Open scope feedback, 1 unplaced note' }),
    )
    const dialog = screen.getByRole('dialog', { name: 'Review the whole exploration' })
    expect(dialog.getAttribute('data-motion-event')).toBe('reconfigure')
    expect(dialog.closest('.scope-conversation')).toBeNull()
    expect(document.activeElement).toBe(dialog)
    expect(screen.getByText('This concern must remain reachable.')).toBeTruthy()
    expect(screen.getByText('Anchor lost')).toBeTruthy()
    expect(screen.getByText('Text rewritten away.')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Keep exploring with this feedback' })).toBeTruthy()
    await fireEvent.keyDown(dialog, { key: 'Escape' })
    expect(screen.queryByRole('dialog', { name: 'Review the whole exploration' })).toBeNull()
    await waitFor(() =>
      expect(document.activeElement).toBe(
        screen.getByRole('button', { name: 'Open scope feedback, 1 unplaced note' }),
      ),
    )
    expect(screen.getByRole('button', { name: 'Move to proposal' })).toBeTruthy()
  })

  it('renders successful no-artifact scope without fabricating query state', async () => {
    useChange()
    stubScope({
      ...CHANGE_SCOPE,
      artifacts: [],
      comments: [],
      commentCounts: { open: 0, addressed: 0, resolved: 0 },
    })
    render(ScopeView)

    expect(await screen.findByRole('heading', { name: 'No artifacts written yet' })).toBeTruthy()
    expect(screen.getByText(/Document stage will appear/)).toBeTruthy()
    expect(replace).not.toHaveBeenCalled()
  })

  it('surfaces scope fetch failure', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response({ error: 'fixture exploded' }, 500)))
    render(ScopeView)

    expect(await screen.findByRole('heading', { name: 'Scope unavailable' })).toBeTruthy()
    expect(screen.getByRole('alert').textContent).toContain('fixture exploded')
  })

  it('surfaces action failure without replacing loaded scope', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(response(SESSION_SCOPE))
      .mockResolvedValueOnce(response({ error: 'comment refused' }, 409))
    vi.stubGlobal('fetch', fetchMock)
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    await fireEvent.click(screen.getByRole('button', { name: /Comment on .* block 2/ }))
    await fireEvent.update(screen.getByLabelText('Comment on block'), 'Keep exact failure.')
    await fireEvent.click(screen.getByRole('button', { name: 'Record comment' }))

    expect((await screen.findByRole('alert')).textContent).toContain('comment refused')
    expect(screen.getByRole('heading', { name: 'Exploration' })).toBeTruthy()
    expect(screen.getByLabelText('Comment on block')).toHaveProperty('value', 'Keep exact failure.')
    expect(container.querySelector('[data-motion-event="receive"]')).toBeNull()
  })

  it('names unresolved comment transmission and lands success at created thread', async () => {
    let resolveMutation: (response: Response) => void = () => {
      throw new Error('mutation resolver was not initialized')
    }
    const pendingMutation = new Promise<Response>((resolve) => {
      resolveMutation = resolve
    })
    const createdThread = {
      ...SESSION_SCOPE.comments[0],
      comment: {
        ...SESSION_SCOPE.comments[0]?.comment,
        id: 'created-thread',
        body: 'New anchored concern.',
      },
      replies: [],
    }
    const updated = {
      ...SESSION_SCOPE,
      comments: [...SESSION_SCOPE.comments, createdThread],
      commentCounts: { open: 2, addressed: 0, resolved: 1 },
    }
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(response(SESSION_SCOPE))
      .mockReturnValueOnce(pendingMutation)
      .mockResolvedValueOnce(response(updated))
    vi.stubGlobal('fetch', fetchMock)
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    await fireEvent.click(screen.getByRole('button', { name: /Comment on .* block 2/ }))
    await fireEvent.update(screen.getByLabelText('Comment on block'), 'New anchored concern.')
    await fireEvent.click(screen.getByRole('button', { name: 'Record comment' }))

    expect(screen.getByText('Transmitting comment to review record').getAttribute('role')).toBe(
      'status',
    )
    expect(screen.getByLabelText('Comment on block')).toHaveProperty(
      'value',
      'New anchored concern.',
    )
    resolveMutation(response({ id: 'created-thread' }, 201))

    await waitFor(() =>
      expect(
        container
          .querySelector('#artifact-thread-created-thread')
          ?.classList.contains('artifact-conversation__thread--received'),
      ).toBe(true),
    )
    expect(screen.queryByLabelText('Comment on block')).toBeNull()
  })

  it('lands successful status and verdict outcomes at semantic destinations', async () => {
    const resolvedScope: ScopeDetail = {
      ...SESSION_SCOPE,
      comments: SESSION_SCOPE.comments.map((thread) =>
        thread.comment.id === 'anchored' ? { ...thread, status: 'resolved' as const } : thread,
      ),
      commentCounts: { open: 0, addressed: 0, resolved: 2 },
    }
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(response(SESSION_SCOPE))
      .mockResolvedValueOnce(response({ status: 'resolved' }))
      .mockResolvedValueOnce(response(resolvedScope))
    vi.stubGlobal('fetch', fetchMock)
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    await fireEvent.click(screen.getByRole('button', { name: 'Resolve' }))
    await waitFor(() =>
      expect(container.querySelector('.comment-thread--received-status')).toBeTruthy(),
    )
    expect(container.querySelector('.status-mark[data-motion-event="resolve"]')).toBeTruthy()

    const verdictScope: ScopeDetail = {
      ...resolvedScope,
      verdicts: [
        ...resolvedScope.verdicts,
        {
          id: 'verdict-2',
          verdict: 'move-to-proposal',
          notes: '',
          createdAt: '2026-08-07T12:00:00Z',
        },
      ],
      standingVerdict: {
        id: 'verdict-2',
        verdict: 'move-to-proposal',
        createdAt: '2026-08-07T12:00:00Z',
        directiveDelivered: false,
        directivePending: true,
      },
    }
    fetchMock
      .mockResolvedValueOnce(response({ id: 'verdict-2' }, 201))
      .mockResolvedValueOnce(response(verdictScope))
    await fireEvent.click(screen.getByRole('button', { name: 'Move to proposal' }))
    await waitFor(() =>
      expect(container.querySelector('.scope-header__standing--received')).toBeTruthy(),
    )
    expect(screen.getAllByText('move-to-proposal').length).toBeGreaterThan(0)
  })

  it('applies clean live update and preserves selected-artifact reading model', async () => {
    const updated = {
      ...SESSION_SCOPE,
      artifacts: [artifact(SESSION_PATH, 'session-new', 'New document text')],
      comments: [],
    }
    const fetchMock = stubLiveScope()
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    fetchMock.mockResolvedValueOnce(response(updated))
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: true, reviewStateChanged: false }),
    )

    expect(await screen.findByRole('heading', { name: 'New document text' })).toBeTruthy()
    expect(screen.queryByRole('heading', { name: 'Exploration' })).toBeNull()
  })

  it('defers artifact snapshot while composer is dirty and applies latest on dismissal', async () => {
    const first = {
      ...SESSION_SCOPE,
      artifacts: [artifact(SESSION_PATH, 'first', 'First rewrite')],
    }
    const latest = {
      ...SESSION_SCOPE,
      artifacts: [artifact(SESSION_PATH, 'latest', 'Latest rewrite')],
    }
    const fetchMock = stubLiveScope()
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })
    await fireEvent.click(screen.getByRole('button', { name: /Comment on .* block 2/ }))
    await fireEvent.update(screen.getByLabelText('Comment on block'), 'Keep this sentence.')

    fetchMock.mockResolvedValueOnce(response(first)).mockResolvedValueOnce(response(latest))
    const source = StubEventSource.instances[0]
    source?.emit(JSON.stringify({ artifactsChanged: true, reviewStateChanged: false }))
    source?.emit(JSON.stringify({ artifactsChanged: true, reviewStateChanged: false }))

    expect(await screen.findByText('Artifact changed.')).toBeTruthy()
    expect(screen.queryByRole('heading', { name: 'Latest rewrite' })).toBeNull()
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(await screen.findByRole('heading', { name: 'Latest rewrite' })).toBeTruthy()
  })

  it('reports an arriving artifact replacement at the document without moving the reader', async () => {
    const updated = {
      ...SESSION_SCOPE,
      artifacts: [artifact(SESSION_PATH, 'session-new', 'New document text')],
    }
    const scrollTo = vi.fn()
    Object.defineProperty(window, 'scrollY', { configurable: true, value: 420 })
    const fetchMock = stubLiveScope()
    vi.stubGlobal('scrollTo', scrollTo)
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })
    expect(container.querySelector('.artifact-document--replaced')).toBeNull()

    fetchMock.mockResolvedValueOnce(response(updated))
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: true, reviewStateChanged: false }),
    )

    await screen.findByRole('heading', { name: 'New document text' })
    await waitFor(() =>
      expect(container.querySelector('.artifact-document--replaced')).toBeTruthy(),
    )
    expect(screen.getByText('Document content replaced')).toBeTruthy()
    expect(scrollTo).toHaveBeenCalledWith(0, 420)
    await fireEvent.click(screen.getByRole('button', { name: /Comment on .* block 2/ }))
    expect(screen.getByLabelText('Comment on block')).toBeTruthy()
  })

  it('reports a deferred replacement when it applies, not when it was detected', async () => {
    const updated = {
      ...SESSION_SCOPE,
      artifacts: [artifact(SESSION_PATH, 'deferred', 'Deferred rewrite')],
    }
    const fetchMock = stubLiveScope()
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })
    await fireEvent.click(screen.getByRole('button', { name: /Comment on .* block 2/ }))
    await fireEvent.update(screen.getByLabelText('Comment on block'), 'Keep this sentence.')

    fetchMock.mockResolvedValueOnce(response(updated))
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: true, reviewStateChanged: false }),
    )

    expect(await screen.findByText('Artifact changed.')).toBeTruthy()
    expect(container.querySelector('.artifact-document--replaced')).toBeNull()

    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(await screen.findByRole('heading', { name: 'Deferred rewrite' })).toBeTruthy()
    expect(container.querySelector('.artifact-document--replaced')).toBeTruthy()
  })

  it('confirms the reviewer own reply at the thread it extended', async () => {
    stubLiveScope()
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    await fireEvent.click(screen.getByRole('button', { name: 'Reply' }))
    await fireEvent.update(screen.getByLabelText('Reply to thread'), 'Reviewer follow-up.')
    await fireEvent.click(screen.getByRole('button', { name: 'Record reply' }))
    await waitFor(() =>
      expect(container.querySelector('.artifact-conversation__thread--received')).toBeTruthy(),
    )
    expect(
      container.querySelector('.artifact-conversation__thread--received')?.getAttribute('id'),
    ).toBe('artifact-thread-anchored')
    expect(container.querySelector('.artifact-document--replaced')).toBeNull()
  })

  it('clears an arrival report on its own with no dismissal control', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true })
    const updated = {
      ...SESSION_SCOPE,
      artifacts: [artifact(SESSION_PATH, 'decaying', 'Decaying mark')],
    }
    const fetchMock = stubLiveScope()
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    fetchMock.mockResolvedValueOnce(response(updated))
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: true, reviewStateChanged: false }),
    )
    await screen.findByRole('heading', { name: 'Decaying mark' })
    await waitFor(() =>
      expect(container.querySelector('.artifact-document--replaced')).toBeTruthy(),
    )
    expect(container.querySelector('[aria-label="Dismiss"]')).toBeNull()

    vi.advanceTimersByTime(EVENT_DWELL_MS + 1)
    await nextTick()
    expect(container.querySelector('.artifact-document--replaced')).toBeNull()
    expect(screen.queryByText('Document content replaced')).toBeNull()
    vi.useRealTimers()
  })

  it('reports nothing when a live update carries no artifact change', async () => {
    const fetchMock = stubLiveScope()
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    fetchMock.mockResolvedValueOnce(
      response({ ...SESSION_SCOPE, commentCounts: { open: 2, addressed: 0, resolved: 1 } }),
    )
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: false, reviewStateChanged: true }),
    )

    expect(await screen.findByText('2 open')).toBeTruthy()
    expect(container.querySelector('.artifact-document--replaced')).toBeNull()
    expect(container.querySelector('.artifact-conversation__thread--received')).toBeNull()
  })

  it('marks known remote review destination and decays without document claims', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true })
    const updated: ScopeDetail = {
      ...SESSION_SCOPE,
      comments: SESSION_SCOPE.comments.map((thread) =>
        thread.comment.id === 'anchored' ? { ...thread, status: 'resolved' as const } : thread,
      ),
      commentCounts: { open: 0, addressed: 0, resolved: 2 },
    }
    const fetchMock = stubLiveScope()
    const { container } = render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    fetchMock.mockResolvedValueOnce(response(updated))
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: false, reviewStateChanged: true }),
    )

    await screen.findByText('2 resolved')
    await waitFor(() =>
      expect(container.querySelector('.comment-thread--received-status')).toBeTruthy(),
    )
    expect(container.querySelector('.artifact-document--replaced')).toBeNull()
    expect(screen.queryByText('Document content replaced')).toBeNull()

    vi.advanceTimersByTime(EVENT_DWELL_MS + 1)
    await nextTick()
    expect(container.querySelector('.comment-thread--received-status')).toBeNull()
    expect(container.querySelector('[data-motion-event="resolve"]')).toBeNull()
    vi.useRealTimers()
  })

  it('refreshes review state immediately while artifact composer remains dirty', async () => {
    const fetchMock = stubLiveScope()
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })
    await fireEvent.click(screen.getByRole('button', { name: /Comment on .* block 2/ }))
    await fireEvent.update(screen.getByLabelText('Comment on block'), 'Keep this sentence.')

    fetchMock.mockResolvedValueOnce(
      response({ ...SESSION_SCOPE, commentCounts: { open: 2, addressed: 0, resolved: 1 } }),
    )
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: false, reviewStateChanged: true }),
    )

    expect(await screen.findByText('2 open')).toBeTruthy()
    expect(screen.getByLabelText('Comment on block')).toHaveProperty('value', 'Keep this sentence.')
  })
})

describe('approval gate', () => {
  /** A change scope in `state`, with the comment counts `counts` says it has. */
  function changeIn(
    approval: ApprovalState,
    counts = { open: 0, addressed: 0, resolved: 2 },
  ): ScopeDetail {
    return {
      ...CHANGE_SCOPE,
      approval,
      comments: [],
      commentCounts: counts,
    }
  }

  it('renders the approval state and its reason on a change page', async () => {
    useChange()
    stubScope(changeIn(NOT_APPROVED))

    render(ScopeView)
    await waitFor(() => screen.getByRole('region', { name: 'Approval state' }))

    expect(screen.getByText('Not approved')).toBeTruthy()
    expect(screen.getByText(NOT_APPROVED.reason)).toBeTruthy()
  })

  /** A stale approval covers content that is no longer on disk, and saying so
   *  without naming the artifact leaves the reviewer nothing to look at. */
  it('renders a stale approval as stale and names what changed', async () => {
    useChange()
    stubScope(
      changeIn({
        state: 'stale',
        reason: 'approved earlier, but proposal.md has changed since',
        changedArtifacts: ['proposal.md'],
      }),
    )

    render(ScopeView)
    await waitFor(() => screen.getByText('Stale'))

    expect(screen.getByText(/Changed since approval: proposal\.md/)).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Withdraw approval' })).toBeNull()
    expect(screen.getByRole('button', { name: 'Approve this change' })).toBeTruthy()
  })

  it('offers withdrawal on an approved change and no second approve control', async () => {
    useChange()
    stubScope(
      changeIn({
        state: 'approved',
        reason: 'approved on 2026-09-02T09:00:00Z, and no reviewed artifact has changed since',
        changedArtifacts: [],
      }),
    )

    render(ScopeView)
    await waitFor(() => screen.getByText('Approved'))

    expect(screen.getByRole('button', { name: 'Withdraw approval' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Approve this change' })).toBeNull()
  })

  /** Showing both would ask the reviewer to choose between a button that works
   *  and one that is refused, on a page that already knows which is which. */
  it('replaces the plain approve control with the sweep while feedback is outstanding', async () => {
    useChange()
    stubScope(changeIn(NOT_APPROVED, { open: 2, addressed: 1, resolved: 4 }))

    render(ScopeView)
    await waitFor(() => screen.getByRole('region', { name: 'Approval state' }))

    expect(screen.queryByRole('button', { name: 'Approve this change' })).toBeNull()
    expect(
      screen.getByRole('button', { name: 'Resolve 3 and approve (2 open, 1 addressed)' }),
    ).toBeTruthy()
  })

  it('submits the sweep as one request and shows the state it left behind', async () => {
    useChange()
    const outstanding = changeIn(NOT_APPROVED, { open: 2, addressed: 1, resolved: 0 })
    const settled = changeIn({
      state: 'approved',
      reason: 'approved on 2026-09-02T09:00:00Z, and no reviewed artifact has changed since',
      changedArtifacts: [],
    })
    const fetchMock = vi.fn((_input: unknown, init?: RequestInit) => {
      if (init?.method === 'POST') return Promise.resolve(response({ resolved: 3 }))
      return Promise.resolve(
        response(fetchMock.mock.calls.some((call) => call[1]) ? settled : outstanding),
      )
    })
    vi.stubGlobal('fetch', fetchMock)
    vi.stubGlobal('EventSource', StubEventSource)

    render(ScopeView)
    const sweep = await waitFor(() =>
      screen.getByRole('button', { name: 'Resolve 3 and approve (2 open, 1 addressed)' }),
    )
    await fireEvent.click(sweep)

    await waitFor(() => screen.getByText('Approved'))
    const posts = fetchMock.mock.calls.filter((call) => call[1]?.method === 'POST')
    expect(posts).toHaveLength(1)
    expect(posts[0]?.[0]).toContain('/approval')
    expect(posts[0]?.[1]?.body).toBe(JSON.stringify({ act: 'resolve-all-and-approve' }))
  })

  /** An approval that was not recorded and said nothing about why reads as the
   *  button not having worked. */
  it('renders the refusal reason when an approval is refused', async () => {
    useChange()
    const refusal =
      'change-with-a-long-exact-identifier has 1 open and 0 addressed comment(s) outstanding'
    const fetchMock = vi.fn((_input: unknown, init?: RequestInit) =>
      init?.method === 'POST'
        ? Promise.resolve(response({ error: refusal }, 400))
        : Promise.resolve(response(changeIn(NOT_APPROVED))),
    )
    vi.stubGlobal('fetch', fetchMock)
    vi.stubGlobal('EventSource', StubEventSource)

    render(ScopeView)
    await fireEvent.click(
      await waitFor(() => screen.getByRole('button', { name: 'Approve this change' })),
    )

    await waitFor(() => screen.getByText(refusal))
    expect(screen.getByText('Not approved')).toBeTruthy()
  })

  /** The comments are resolved and the change is not approved. Rendering only
   *  half of that would leave the reviewer looking for feedback that is gone. */
  it('renders both halves of a partial failure', async () => {
    useChange()
    const partial =
      'resolved 3 comment(s) on change-with-a-long-exact-identifier and then failed to approve it: ' +
      'the comments are resolved and the change is not approved'
    const fetchMock = vi.fn((_input: unknown, init?: RequestInit) =>
      init?.method === 'POST'
        ? Promise.resolve(response({ error: partial }, 400))
        : Promise.resolve(response(changeIn(NOT_APPROVED, { open: 3, addressed: 0, resolved: 0 }))),
    )
    vi.stubGlobal('fetch', fetchMock)
    vi.stubGlobal('EventSource', StubEventSource)

    render(ScopeView)
    await fireEvent.click(
      await waitFor(() => screen.getByRole('button', { name: 'Resolve 3 and approve (3 open)' })),
    )

    await waitFor(() => screen.getByText(partial))
  })

  /** Staleness is a fact about the artifacts, so an artifact-only update is
   *  exactly what flips an approval with no review state having changed. */
  it('takes a new approval state from a live update without a reload', async () => {
    useChange()
    const approved = changeIn({
      state: 'approved',
      reason: 'approved on 2026-09-02T09:00:00Z, and no reviewed artifact has changed since',
      changedArtifacts: [],
    })
    const stale = changeIn({
      state: 'stale',
      reason: 'approved earlier, but proposal.md has changed since',
      changedArtifacts: ['proposal.md'],
    })
    let current = approved
    const fetchMock = vi.fn(() => Promise.resolve(response(current)))
    vi.stubGlobal('fetch', fetchMock)
    vi.stubGlobal('EventSource', StubEventSource)

    render(ScopeView)
    await waitFor(() => screen.getByText('Approved'))
    // A change page canonicalizes its artifact query before it subscribes, so
    // the rendered document arrives a tick ahead of the event stream.
    await waitFor(() => expect(StubEventSource.instances).toHaveLength(1))

    current = stale
    StubEventSource.instances[0]?.emit(
      JSON.stringify({ artifactsChanged: true, reviewStateChanged: false }),
    )

    await waitFor(() => screen.getByText('Stale'))
    expect(screen.getByText(/Changed since approval: proposal\.md/)).toBeTruthy()
  })

  it('offers no approval instrument on a session', async () => {
    stubScope(SESSION_SCOPE)

    render(ScopeView)
    await waitFor(() => screen.getByRole('heading', { name: SESSION_SCOPE.title ?? '' }))

    expect(screen.queryByRole('region', { name: 'Approval state' })).toBeNull()
  })
})
