import type { Index, Scope, Verdict } from '@/lib/scopes'
import type {
  Anchor,
  AnchorState,
  Artifact,
  Block,
  CommentRecord,
  CommentStatus,
  NewComment,
  Reply,
  ScopeDetail,
  ScopeKind,
  Thread,
  VerdictRecord,
} from '@/lib/scope-review'

export const MOCK_CHANGE_KEY =
  'implement-observatory-design-system-with-a-realistically-long-identifier'
export const MOCK_SESSION_ID = '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74'

const CHANGE_ROOT = `openspec/changes/${MOCK_CHANGE_KEY}`
const CHANGE_ARTIFACT_PATH = `${CHANGE_ROOT}/proposal.md`
const CHANGE_DESIGN_PATH = `${CHANGE_ROOT}/design.md`
const CHANGE_TASKS_PATH = `${CHANGE_ROOT}/tasks.md`
const CHANGE_HTML_SPEC_PATH = `${CHANGE_ROOT}/specs/dashboard-html-views/spec.md`
const CHANGE_VISUAL_SPEC_PATH = `${CHANGE_ROOT}/specs/dashboard-visual-system/spec.md`
const SESSION_ARTIFACT_PATH = `.openspec-doc/scratch/_session/${MOCK_SESSION_ID}.md`

interface MockState {
  index: Index
  scopes: Record<string, ScopeDetail>
}

function scopeAddress(kind: ScopeKind, key: string): string {
  return `${kind}:${key}`
}

function makeBlocks(prefix: string, entries: Array<{ html: string; source: string }>): Block[] {
  let cursor = 0
  return entries.map(({ html, source }, index) => {
    const block: Block = {
      id: `${prefix}-block-${index}`,
      html,
      source,
      range: { start: cursor, end: cursor + source.length },
    }
    cursor += source.length + 2
    return block
  })
}

function makeArtifact(path: string, blocks: Block[]): Artifact {
  return { path, blocks }
}

function requireBlock(blocks: Block[], id: string): Block {
  const block = blocks.find((candidate) => candidate.id === id)
  if (!block) throw new Error(`Mock fixture is missing block ${id}`)
  return block
}

function makeAnchor(path: string, selectedText: string, startOffset: number): Anchor {
  return {
    artifactPath: path,
    selectedText,
    headingPath: ['Observatory Design System Fixture'],
    beforeText: '',
    afterText: '',
    startOffset,
    endOffset: startOffset + selectedText.length,
  }
}

function makeComment(id: string, body: string, createdAt: string, anchor?: Anchor): CommentRecord {
  return anchor ? { id, body, createdAt, anchor } : { id, body, createdAt }
}

function makeThread(
  comment: CommentRecord,
  status: CommentStatus,
  anchorState: AnchorState,
  blockId: string | null,
  replies: Reply[] = [],
): Thread {
  return { comment, status, replies, anchorState, blockId }
}

function makeVerdict(id: string, verdict: Verdict, createdAt: string): VerdictRecord {
  return { id, verdict, notes: '', createdAt }
}

const changeBlocks = makeBlocks('change', [
  {
    html: '<h1>Observatory Design System Fixture</h1>',
    source: '# Observatory Design System Fixture',
  },
  {
    html: '<p>Browser lane must see this proposal.</p>',
    source: 'Browser lane must see this proposal.',
  },
  {
    html: '<p>Repeated review target.</p>',
    source: 'Repeated review target.',
  },
  {
    html: '<p>A bridge between repeated blocks.</p>',
    source: 'A bridge between repeated blocks.',
  },
  {
    html: '<p>Repeated review target.</p>',
    source: 'Repeated review target.',
  },
  {
    html: '<p>Selection with <strong>inline markup</strong> crosses source.</p>',
    source: 'Selection with inline markup crosses source.',
  },
])
const changeTarget = requireBlock(changeBlocks, 'change-block-4')
const designBlocks = makeBlocks('design', [
  { html: '<h1>Fixture design</h1>', source: '# Fixture design' },
  {
    html: '<p>Conversation follows exact selected artifact.</p>',
    source: 'Conversation follows exact selected artifact.',
  },
])
const designTarget = requireBlock(designBlocks, 'design-block-1')
const taskBlocks = makeBlocks('tasks', [
  { html: '<h1>Fixture tasks</h1>', source: '# Fixture tasks' },
  { html: '<ul><li>Render selected artifact</li></ul>', source: '- Render selected artifact' },
])
const htmlSpecBlocks = makeBlocks('html-spec', [
  { html: '<h1>HTML view fixture</h1>', source: '# HTML view fixture' },
  { html: '<p>Nested paths remain exact.</p>', source: 'Nested paths remain exact.' },
])
const visualSpecBlocks = makeBlocks('visual-spec', [
  { html: '<h1>Visual system fixture</h1>', source: '# Visual system fixture' },
  { html: '<p>Artwork yields before prose.</p>', source: 'Artwork yields before prose.' },
])

const sessionBlocks = makeBlocks('session', [
  {
    html: '<h1>Agent-guided observatory exploration session</h1>',
    source: '# Agent-guided observatory exploration session',
  },
  {
    html: '<p>Review atmosphere must yield before content.</p>',
    source: 'Review atmosphere must yield before content.',
  },
])
const sessionTarget = requireBlock(sessionBlocks, 'session-block-1')

const changeScope: ScopeDetail = {
  kind: 'change',
  key: MOCK_CHANGE_KEY,
  title: 'Observatory Design System Fixture',
  artifacts: [
    makeArtifact(CHANGE_ARTIFACT_PATH, changeBlocks),
    makeArtifact(CHANGE_DESIGN_PATH, designBlocks),
    makeArtifact(CHANGE_TASKS_PATH, taskBlocks),
    makeArtifact(CHANGE_HTML_SPEC_PATH, htmlSpecBlocks),
    makeArtifact(CHANGE_VISUAL_SPEC_PATH, visualSpecBlocks),
  ],
  comments: [
    makeThread(
      makeComment(
        'mock-change-open',
        'Keep repeated occurrence mapping exact.',
        '2026-08-07T10:00:00Z',
        makeAnchor(CHANGE_ARTIFACT_PATH, changeTarget.source, changeTarget.range.start),
      ),
      'open',
      'exact',
      changeTarget.id,
      [
        {
          id: 'mock-change-agent-reply',
          commentId: 'mock-change-open',
          author: 'agent',
          body: 'Second occurrence is now explicit.',
          createdAt: '2026-08-07T10:01:00Z',
        },
      ],
    ),
    makeThread(
      makeComment(
        'mock-design-addressed',
        'Keep design conversation artifact-scoped.',
        '2026-08-07T10:01:30Z',
        makeAnchor(CHANGE_DESIGN_PATH, designTarget.source, designTarget.range.start),
      ),
      'addressed',
      'exact',
      designTarget.id,
    ),
    makeThread(
      makeComment(
        'mock-change-orphaned',
        'Lost anchors must stay reachable.',
        '2026-08-07T10:02:00Z',
        makeAnchor(CHANGE_ARTIFACT_PATH, 'Original text rewritten away.', 900),
      ),
      'resolved',
      'orphaned',
      null,
    ),
    makeThread(
      makeComment('mock-change-addressed', 'Whole-scope agent claim.', '2026-08-07T10:03:00Z'),
      'addressed',
      'unanchored',
      null,
    ),
  ],
  commentCounts: { open: 1, addressed: 2, resolved: 1 },
  verdicts: [makeVerdict('mock-change-verdict', 'comment-resolution', '2026-08-07T10:04:00Z')],
  standingVerdict: {
    id: 'mock-change-verdict',
    verdict: 'comment-resolution',
    createdAt: '2026-08-07T10:04:00Z',
    directiveDelivered: false,
    directivePending: false,
  },
}

const sessionScope: ScopeDetail = {
  kind: 'session',
  key: MOCK_SESSION_ID,
  title: 'Agent-guided observatory exploration session',
  artifacts: [makeArtifact(SESSION_ARTIFACT_PATH, sessionBlocks)],
  comments: [
    makeThread(
      makeComment(
        'mock-session-open',
        'Review this exact exploration block.',
        '2026-08-07T09:00:00Z',
        makeAnchor(SESSION_ARTIFACT_PATH, sessionTarget.source, sessionTarget.range.start),
      ),
      'open',
      'fuzzy',
      sessionTarget.id,
    ),
  ],
  commentCounts: { open: 1, addressed: 0, resolved: 0 },
  verdicts: [makeVerdict('mock-session-verdict', 'keep-exploring', '2026-08-07T09:01:00Z')],
  standingVerdict: {
    id: 'mock-session-verdict',
    verdict: 'keep-exploring',
    createdAt: '2026-08-07T09:01:00Z',
    directiveDelivered: false,
    directivePending: true,
  },
}

const initialIndex: Index = {
  sessions: [
    {
      key: MOCK_SESSION_ID,
      title: sessionScope.title,
      modifiedAt: '2026-08-07T09:01:00Z',
      openComments: 1,
      verdict: 'keep-exploring',
      mostRecentlyActive: false,
    },
  ],
  changes: [
    {
      key: MOCK_CHANGE_KEY,
      title: changeScope.title,
      modifiedAt: '2026-08-07T10:04:00Z',
      openComments: 1,
      verdict: 'comment-resolution',
      mostRecentlyActive: true,
    },
  ],
}

const initialState: MockState = {
  index: initialIndex,
  scopes: {
    [scopeAddress('session', MOCK_SESSION_ID)]: sessionScope,
    [scopeAddress('change', MOCK_CHANGE_KEY)]: changeScope,
  },
}

let state = structuredClone(initialState)
let sequence = 0

export function getMockState(): MockState {
  return state
}

export function getMockScope(kind: ScopeKind, key: string): ScopeDetail | undefined {
  return state.scopes[scopeAddress(kind, key)]
}

function requireMockScope(kind: ScopeKind, key: string): ScopeDetail {
  const scope = getMockScope(kind, key)
  if (!scope) throw new Error(`Mock fixture is missing ${kind} scope ${key}`)
  return scope
}

function refreshCounts(scope: ScopeDetail): void {
  const counts = { open: 0, addressed: 0, resolved: 0 }
  for (const thread of scope.comments) counts[thread.status] += 1
  scope.commentCounts = counts
}

function touchSummary(kind: ScopeKind, key: string, modifiedAt: string): void {
  const scope = requireMockScope(kind, key)
  const summaries: Scope[] = kind === 'session' ? state.index.sessions : state.index.changes
  const summary = summaries.find((candidate) => candidate.key === key)
  if (!summary) throw new Error(`Mock fixture is missing ${kind} summary ${key}`)
  summary.modifiedAt = modifiedAt
  summary.openComments = scope.commentCounts.open
  summary.verdict = scope.verdicts[scope.verdicts.length - 1]?.verdict ?? null
}

export function addMockComment(kind: ScopeKind, key: string, input: NewComment): CommentRecord {
  const scope = requireMockScope(kind, key)
  const createdAt = new Date().toISOString()
  const id = `mock-comment-${++sequence}`

  if (input.kind === 'unanchored') {
    const comment = makeComment(id, input.body, createdAt)
    scope.comments.push(makeThread(comment, 'open', 'unanchored', null))
    refreshCounts(scope)
    touchSummary(kind, key, createdAt)
    return comment
  }

  const artifact = scope.artifacts.find((candidate) => candidate.path === input.artifactPath)
  const block = artifact?.blocks.find(
    (candidate) =>
      candidate.range.start <= input.searchFrom && input.searchFrom < candidate.range.end,
  )
  const comment = makeComment(
    id,
    input.body,
    createdAt,
    makeAnchor(input.artifactPath, input.selectedText, input.searchFrom),
  )
  scope.comments.push(makeThread(comment, 'open', block ? 'exact' : 'orphaned', block?.id ?? null))
  refreshCounts(scope)
  touchSummary(kind, key, createdAt)
  return comment
}

export function addMockReply(
  kind: ScopeKind,
  key: string,
  commentId: string,
  body: string,
): Reply | undefined {
  const scope = requireMockScope(kind, key)
  const thread = scope.comments.find((candidate) => candidate.comment.id === commentId)
  if (!thread) return undefined
  const reply: Reply = {
    id: `mock-reply-${++sequence}`,
    commentId,
    author: 'reviewer',
    body,
    createdAt: new Date().toISOString(),
  }
  thread.replies.push(reply)
  touchSummary(kind, key, reply.createdAt)
  return reply
}

export interface MockStatusUpdate {
  id: string
  commentId: string
  status: 'open' | 'resolved'
  createdAt: string
}

export function setMockCommentStatus(
  kind: ScopeKind,
  key: string,
  commentId: string,
  status: 'open' | 'resolved',
): MockStatusUpdate | undefined {
  const scope = requireMockScope(kind, key)
  const thread = scope.comments.find((candidate) => candidate.comment.id === commentId)
  if (!thread) return undefined
  const update: MockStatusUpdate = {
    id: `mock-status-${++sequence}`,
    commentId,
    status,
    createdAt: new Date().toISOString(),
  }
  thread.status = status
  refreshCounts(scope)
  touchSummary(kind, key, update.createdAt)
  return update
}

export function submitMockVerdict(kind: ScopeKind, key: string, verdict: Verdict): VerdictRecord {
  const scope = requireMockScope(kind, key)
  const record = makeVerdict(`mock-verdict-${++sequence}`, verdict, new Date().toISOString())
  scope.verdicts.push(record)
  scope.standingVerdict = {
    id: record.id,
    verdict: record.verdict,
    createdAt: record.createdAt,
    directiveDelivered: false,
    directivePending: true,
  }
  touchSummary(kind, key, record.createdAt)
  return record
}

export function resetMockState(): void {
  state = structuredClone(initialState)
  sequence = 0
}

export { scopeAddress }
