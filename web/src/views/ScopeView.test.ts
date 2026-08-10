import { fireEvent, render, screen, waitFor } from '@testing-library/vue'
import { reactive } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Artifact, ScopeDetail } from '@/lib/scope-review'
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

const CHANGE_SCOPE: ScopeDetail = {
  ...SESSION_SCOPE,
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
  const fetchMock = vi.fn().mockResolvedValue(response(initial))
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

afterEach(() => {
  route.name = 'session'
  route.params = { id: SESSION_SCOPE.key }
  route.query = {}
  replace.mockClear()
  push.mockClear()
  StubEventSource.instances = []
  vi.unstubAllGlobals()
})

describe('ScopeView selected-artifact workbench', () => {
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

    await fireEvent.click(screen.getByRole('button', { name: /tasks\.md/ }))
    expect(push).toHaveBeenLastCalledWith({ query: { artifact: TASKS_PATH } })
    expect(await screen.findByRole('heading', { name: 'Tasks coordinate' })).toBeTruthy()

    route.query = { artifact: DESIGN_PATH }
    expect(await screen.findByRole('heading', { name: 'Design coordinate' })).toBeTruthy()
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
    expect(document.querySelector('.artifact-conversation__thread--active')).toBeTruthy()

    await fireEvent.click(
      screen.getByRole('button', { name: 'Locate source for comment proposal-thread' }),
    )
    expect(document.activeElement).toBe(repeated[1])
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
    expect(dialog.closest('.scope-conversation')).toBeNull()
    expect(document.activeElement).toBe(screen.getByLabelText('New whole-exploration note'))
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
    render(ScopeView)
    await screen.findByRole('heading', { name: 'Exploration' })

    await fireEvent.click(screen.getByRole('button', { name: /Comment on .* block 2/ }))
    await fireEvent.update(screen.getByLabelText('Comment on block'), 'Keep exact failure.')
    await fireEvent.click(screen.getByRole('button', { name: 'Record comment' }))

    expect((await screen.findByRole('alert')).textContent).toContain('comment refused')
    expect(screen.getByRole('heading', { name: 'Exploration' })).toBeTruthy()
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
