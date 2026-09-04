import type { Index, Scope, Verdict } from '@/lib/scopes'
import type {
  Anchor,
  AnchorState,
  ApprovalAct,
  ApprovalOutcome,
  ApprovalState,
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

export const MOCK_CHANGE_KEY = 'add-dashboard-lifecycle'
export const MOCK_SESSION_ID = '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74'

const CHANGE_ROOT = `openspec/changes/${MOCK_CHANGE_KEY}`
const CHANGE_PROPOSAL_PATH = `${CHANGE_ROOT}/proposal.md`
const CHANGE_DESIGN_PATH = `${CHANGE_ROOT}/design.md`
const CHANGE_TASKS_PATH = `${CHANGE_ROOT}/tasks.md`
const CHANGE_LIFECYCLE_SPEC_PATH = `${CHANGE_ROOT}/specs/dashboard-lifecycle/spec.md`
const CHANGE_SERVER_SPEC_PATH = `${CHANGE_ROOT}/specs/dashboard-server/spec.md`
const CHANGE_SCRATCH_PATH = `.openspec-doc/scratch/${MOCK_CHANGE_KEY}.md`
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

function makeAnchor(
  path: string,
  selectedText: string,
  startOffset: number,
  headingPath: string[] = [],
): Anchor {
  return {
    artifactPath: path,
    selectedText,
    headingPath,
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

/** What a change nobody has approved reports. Never inferred from settled
 *  feedback: a change nobody looked at has no open comments either. */
function notApproved(): ApprovalState {
  return {
    state: 'not-approved',
    reason: 'no approval has been recorded for this change',
    changedArtifacts: [],
  }
}

function makeVerdict(id: string, verdict: Verdict, createdAt: string): VerdictRecord {
  return { id, verdict, notes: '', createdAt }
}

interface SupportingChangeInput {
  key: string
  title: string
  why: string
  change: string
  decision: string
  tasks: Array<{ done: boolean; label: string }>
  specs: Array<{ capability: string; requirement: string; scenario: string }>
  verdict?: Verdict
}

function makeSupportingChange(input: SupportingChangeInput): ScopeDetail {
  const root = `openspec/changes/${input.key}`
  const proposal = makeBlocks(`${input.key}-proposal`, [
    { html: '<h2>Why</h2>', source: '## Why' },
    { html: `<p>${input.why}</p>`, source: input.why },
    { html: '<h2>What Changes</h2>', source: '## What Changes' },
    { html: `<ul><li>${input.change}</li></ul>`, source: `- ${input.change}` },
    { html: '<h2>Impact</h2>', source: '## Impact' },
    {
      html: `<p>The change touches the narrowest existing owner and updates its executable contract.</p>`,
      source:
        'The change touches the narrowest existing owner and updates its executable contract.',
    },
  ])
  const design = makeBlocks(`${input.key}-design`, [
    { html: '<h2>Context</h2>', source: '## Context' },
    { html: `<p>${input.why}</p>`, source: input.why },
    { html: '<h2>Goals / Non-Goals</h2>', source: '## Goals / Non-Goals' },
    {
      html: `<ul><li>Deliver the stated behavior without inventing adjacent lifecycle state.</li></ul>`,
      source: '- Deliver the stated behavior without inventing adjacent lifecycle state.',
    },
    { html: '<h2>Decisions</h2>', source: '## Decisions' },
    { html: `<p>${input.decision}</p>`, source: input.decision },
    { html: '<h2>Risks / Trade-offs</h2>', source: '## Risks / Trade-offs' },
    {
      html: '<ul><li>The operational boundary must stay explicit when the happy path fails.</li></ul>',
      source: '- The operational boundary must stay explicit when the happy path fails.',
    },
  ])
  const tasks = makeBlocks(`${input.key}-tasks`, [
    { html: '<h2>1. Implementation</h2>', source: '## 1. Implementation' },
    ...input.tasks.map(({ done, label }) => ({
      html: `<ul><li><input disabled="" type="checkbox"${done ? ' checked=""' : ''}> ${label}</li></ul>`,
      source: `- [${done ? 'x' : ' '}] ${label}`,
    })),
    { html: '<h2>2. Verification</h2>', source: '## 2. Verification' },
    {
      html: '<ul><li><input disabled="" type="checkbox"> Exercise the real boundary manually.</li></ul>',
      source: '- [ ] Exercise the real boundary manually.',
    },
  ])
  const specs = input.specs.map(({ capability, requirement, scenario }) =>
    makeArtifact(
      `${root}/specs/${capability}/spec.md`,
      makeBlocks(`${input.key}-${capability}`, [
        { html: '<h2>ADDED Requirements</h2>', source: '## ADDED Requirements' },
        { html: `<h3>Requirement: ${requirement}</h3>`, source: `### Requirement: ${requirement}` },
        {
          html: `<p>The system SHALL ${input.change.charAt(0).toLowerCase()}${input.change.slice(1)}.</p>`,
          source: `The system SHALL ${input.change.charAt(0).toLowerCase()}${input.change.slice(1)}.`,
        },
        { html: `<h4>Scenario: ${scenario}</h4>`, source: `#### Scenario: ${scenario}` },
        {
          html: '<ul><li><strong>WHEN</strong> the capability is invoked</li></ul>',
          source: '- **WHEN** the capability is invoked',
        },
        {
          html: '<ul><li><strong>THEN</strong> the stated behavior is observable</li></ul>',
          source: '- **THEN** the stated behavior is observable',
        },
      ]),
    ),
  )
  const verdicts = input.verdict
    ? [makeVerdict(`${input.key}-verdict`, input.verdict, '2026-08-08T14:00:00Z')]
    : []

  return {
    kind: 'change',
    key: input.key,
    title: input.title,
    artifacts: [
      makeArtifact(`${root}/proposal.md`, proposal),
      makeArtifact(`${root}/design.md`, design),
      makeArtifact(`${root}/tasks.md`, tasks),
      ...specs,
    ],
    comments: [],
    commentCounts: { open: 0, addressed: 0, resolved: 0 },
    approval: notApproved(),
    verdicts,
    standingVerdict: input.verdict
      ? {
          id: `${input.key}-verdict`,
          verdict: input.verdict,
          createdAt: '2026-08-08T14:00:00Z',
          directiveDelivered: true,
          directivePending: false,
        }
      : null,
  }
}

function makeSupportingSession(
  key: string,
  title: string | null,
  paragraphs: string[],
  verdict?: Verdict,
): ScopeDetail {
  const blocks = makeBlocks(`session-${key}`, [
    ...(title ? [{ html: `<h1>${title}</h1>`, source: `# ${title}` }] : []),
    { html: '<h2>What is known</h2>', source: '## What is known' },
    ...paragraphs.map((paragraph) => ({ html: `<p>${paragraph}</p>`, source: paragraph })),
    { html: '<h2>Next question</h2>', source: '## Next question' },
    {
      html: '<p>Which constraint must be settled before this exploration becomes a change?</p>',
      source: 'Which constraint must be settled before this exploration becomes a change?',
    },
  ])
  const verdicts = verdict
    ? [makeVerdict(`session-${key}-verdict`, verdict, '2026-08-08T13:00:00Z')]
    : []

  return {
    kind: 'session',
    key,
    title,
    artifacts: [makeArtifact(`.openspec-doc/scratch/_session/${key}.md`, blocks)],
    comments: [],
    commentCounts: { open: 0, addressed: 0, resolved: 0 },
    approval: null,
    verdicts,
    standingVerdict: verdict
      ? {
          id: `session-${key}-verdict`,
          verdict,
          createdAt: '2026-08-08T13:00:00Z',
          directiveDelivered: verdict === 'keep-exploring',
          directivePending: verdict !== 'keep-exploring',
        }
      : null,
  }
}

const proposalBlocks = makeBlocks('change-proposal', [
  { html: '<h2>Why</h2>', source: '## Why' },
  {
    html: '<p>The review loop has a manual precondition nobody remembers: the owner must have <code>openspec-doc serve</code> running before any of it works.</p>',
    source:
      'The review loop has a manual precondition nobody remembers: the owner must have `openspec-doc serve` running before any of it works.',
  },
  { html: '<h2>What Changes</h2>', source: '## What Changes' },
  {
    html: '<ul><li>The dashboard is started by the hooks at the first turn boundary with review material.</li></ul>',
    source:
      '- The dashboard is started by the hooks at the first turn boundary with review material.',
  },
  {
    html: '<ul><li>A running dashboard is found by probing a bounded localhost port range for project identity.</li></ul>',
    source:
      '- A running dashboard is found by probing a bounded localhost port range for project identity.',
  },
  {
    html: '<ul><li>A hook-started dashboard exits only after both review pages and hook heartbeats are idle.</li></ul>',
    source:
      '- A hook-started dashboard exits only after both review pages and hook heartbeats are idle.',
  },
  { html: '<h2>Capabilities</h2>', source: '## Capabilities' },
  {
    html: '<p><code>dashboard-lifecycle</code> is new; <code>dashboard-server</code> gains identity, fixed-port discovery, and opt-in idle exit.</p>',
    source:
      '`dashboard-lifecycle` is new; `dashboard-server` gains identity, fixed-port discovery, and opt-in idle exit.',
  },
  { html: '<h2>Impact</h2>', source: '## Impact' },
  {
    html: '<p>The hook entry points, server identity route, discovery module, process detachment, and operator documentation move together.</p>',
    source:
      'The hook entry points, server identity route, discovery module, process detachment, and operator documentation move together.',
  },
])
const proposalTarget = requireBlock(proposalBlocks, 'change-proposal-block-1')
const designBlocks = makeBlocks('design', [
  { html: '<h2>Context</h2>', source: '## Context' },
  {
    html: '<p>Discovery must survive <code>git clean</code>, a reboot, and a killed process without manufacturing orphaned servers.</p>',
    source:
      'Discovery must survive `git clean`, a reboot, and a killed process without manufacturing orphaned servers.',
  },
  { html: '<h2>Goals / Non-Goals</h2>', source: '## Goals / Non-Goals' },
  {
    html: '<ul><li>Start once, reuse by canonical project root, and fail loudly when startup does not converge.</li></ul>',
    source:
      '- Start once, reuse by canonical project root, and fail loudly when startup does not converge.',
  },
  { html: '<h2>Decisions</h2>', source: '## Decisions' },
  {
    html: '<p>Discovery is stateless: probe <code>4321</code>–<code>4330</code> and compare the canonical root returned by <code>/identity</code>.</p>',
    source:
      'Discovery is stateless: probe `4321`–`4330` and compare the canonical root returned by `/identity`.',
  },
  {
    html: '<p>The server is detached into its own process group and writes startup output to <code>.openspec-doc/serve.log</code>.</p>',
    source:
      'The server is detached into its own process group and writes startup output to `.openspec-doc/serve.log`.',
  },
  { html: '<h2>Risks / Trade-offs</h2>', source: '## Risks / Trade-offs' },
  {
    html: '<ul><li>A service outside the bounded range is intentionally invisible to stateless discovery.</li></ul>',
    source:
      '- A service outside the bounded range is intentionally invisible to stateless discovery.',
  },
])
const designTarget = requireBlock(designBlocks, 'design-block-5')
const taskBlocks = makeBlocks('tasks', [
  { html: '<h2>1. Identity and discovery</h2>', source: '## 1. Identity and discovery' },
  {
    html: '<ul><li><input checked="" disabled="" type="checkbox"> Add the identity route with canonical root and pid.</li></ul>',
    source: '- [x] Add the identity route with canonical root and pid.',
  },
  {
    html: '<ul><li><input disabled="" type="checkbox"> Probe the bounded port range with strict timeouts.</li></ul>',
    source: '- [ ] Probe the bounded port range with strict timeouts.',
  },
  { html: '<h2>2. Hook ownership</h2>', source: '## 2. Hook ownership' },
  {
    html: '<ul><li><input disabled="" type="checkbox"> Register only sessions that have review material.</li></ul>',
    source: '- [ ] Register only sessions that have review material.',
  },
  {
    html: '<ul><li><input disabled="" type="checkbox"> Open the session page once, after its note exists.</li></ul>',
    source: '- [ ] Open the session page once, after its note exists.',
  },
  { html: '<h2>3. Live verification</h2>', source: '## 3. Live verification' },
  {
    html: '<ul><li><input disabled="" type="checkbox"> Kill a dashboard mid-session and confirm the next turn restores it.</li></ul>',
    source: '- [ ] Kill a dashboard mid-session and confirm the next turn restores it.',
  },
])
const taskTarget = requireBlock(taskBlocks, 'tasks-block-4')
const lifecycleSpecBlocks = makeBlocks('lifecycle-spec', [
  { html: '<h2>ADDED Requirements</h2>', source: '## ADDED Requirements' },
  {
    html: '<h3>Requirement: A running dashboard is discovered by probing</h3>',
    source: '### Requirement: A running dashboard is discovered by probing',
  },
  {
    html: '<p>The system SHALL determine whether a dashboard serves a project root by probing a bounded local port range.</p>',
    source:
      'The system SHALL determine whether a dashboard serves a project root by probing a bounded local port range.',
  },
  {
    html: '<h4>Scenario: Existing dashboard is reused</h4>',
    source: '#### Scenario: Existing dashboard is reused',
  },
  {
    html: '<ul><li><strong>WHEN</strong> a dashboard for the same canonical root answers</li></ul>',
    source: '- **WHEN** a dashboard for the same canonical root answers',
  },
  {
    html: '<ul><li><strong>THEN</strong> the system reuses it and starts no duplicate</li></ul>',
    source: '- **THEN** the system reuses it and starts no duplicate',
  },
])
const specTarget = requireBlock(lifecycleSpecBlocks, 'lifecycle-spec-block-2')
const serverSpecBlocks = makeBlocks('server-spec', [
  { html: '<h2>MODIFIED Requirements</h2>', source: '## MODIFIED Requirements' },
  {
    html: '<h3>Requirement: Dashboard server identity is queryable</h3>',
    source: '### Requirement: Dashboard server identity is queryable',
  },
  {
    html: '<p>The system SHALL expose the canonical project root and process id from a same-origin identity route.</p>',
    source:
      'The system SHALL expose the canonical project root and process id from a same-origin identity route.',
  },
  {
    html: '<h4>Scenario: Identity names the serving process</h4>',
    source: '#### Scenario: Identity names the serving process',
  },
])
const scratchBlocks = makeBlocks('change-scratch', [
  {
    html: '<h1>Exploration: starting the dashboard from the explore slash command</h1>',
    source: '# Exploration: starting the dashboard from the explore slash command',
  },
  { html: '<h2>The ask</h2>', source: '## The ask' },
  {
    html: '<p><code>/opsx:explore</code> should leave the reviewer with a dashboard already running, from either supported agent.</p>',
    source:
      '`/opsx:explore` should leave the reviewer with a dashboard already running, from either supported agent.',
  },
  {
    html: '<h2>What changed during exploration</h2>',
    source: '## What changed during exploration',
  },
  {
    html: '<p>The initial state-file recommendation was rejected because deleting local runtime state would leave a live server undiscoverable.</p>',
    source:
      'The initial state-file recommendation was rejected because deleting local runtime state would leave a live server undiscoverable.',
  },
  {
    html: '<p>The retained decision is stateless discovery through a bounded identity probe.</p>',
    source: 'The retained decision is stateless discovery through a bounded identity probe.',
  },
])

const sessionBlocks = makeBlocks('session', [
  {
    html: '<h1>Approval gate: stale reviews and reviewer intent</h1>',
    source: '# Approval gate: stale reviews and reviewer intent',
  },
  { html: '<h2>Observed gap</h2>', source: '## Observed gap' },
  {
    html: '<p>No open comments does not mean a change was reviewed; an untouched change also has no open comments.</p>',
    source:
      'No open comments does not mean a change was reviewed; an untouched change also has no open comments.',
  },
  { html: '<h2>Open questions</h2>', source: '## Open questions' },
  {
    html: '<ul><li>Which artifacts invalidate an approval when they change?</li></ul>',
    source: '- Which artifacts invalidate an approval when they change?',
  },
  {
    html: '<ul><li>Should completed task checkboxes count as implementation evidence?</li></ul>',
    source: '- Should completed task checkboxes count as implementation evidence?',
  },
  { html: '<h2>Working conclusion</h2>', source: '## Working conclusion' },
  {
    html: '<p>Approval must be explicit, must reject unresolved feedback, and must become stale when reviewed artifacts change.</p>',
    source:
      'Approval must be explicit, must reject unresolved feedback, and must become stale when reviewed artifacts change.',
  },
])
const sessionTarget = requireBlock(sessionBlocks, 'session-block-7')

const changeScope: ScopeDetail = {
  kind: 'change',
  key: MOCK_CHANGE_KEY,
  title: 'Exploration: starting the dashboard from the explore slash command',
  artifacts: [
    makeArtifact(CHANGE_PROPOSAL_PATH, proposalBlocks),
    makeArtifact(CHANGE_DESIGN_PATH, designBlocks),
    makeArtifact(CHANGE_TASKS_PATH, taskBlocks),
    makeArtifact(CHANGE_LIFECYCLE_SPEC_PATH, lifecycleSpecBlocks),
    makeArtifact(CHANGE_SERVER_SPEC_PATH, serverSpecBlocks),
    makeArtifact(CHANGE_SCRATCH_PATH, scratchBlocks),
  ],
  comments: [
    makeThread(
      makeComment(
        'mock-proposal-open',
        'This says the manual startup is forgotten, but the change should also name the silent failure it creates.',
        '2026-08-08T09:40:00Z',
        makeAnchor(CHANGE_PROPOSAL_PATH, proposalTarget.source, proposalTarget.range.start, [
          'Why',
        ]),
      ),
      'open',
      'exact',
      proposalTarget.id,
      [
        {
          id: 'mock-proposal-agent-reply',
          commentId: 'mock-proposal-open',
          author: 'agent',
          body: 'The Why section now states that the loop appears healthy while no reviewer is connected.',
          createdAt: '2026-08-08T09:48:00Z',
        },
      ],
    ),
    makeThread(
      makeComment(
        'mock-design-addressed',
        'Explain why the identity probe is safer than a pid file after cleanup.',
        '2026-08-08T10:02:00Z',
        makeAnchor(CHANGE_DESIGN_PATH, designTarget.source, designTarget.range.start, [
          'Decisions',
        ]),
      ),
      'addressed',
      'fuzzy',
      designTarget.id,
      [
        {
          id: 'mock-design-agent-reply',
          commentId: 'mock-design-addressed',
          author: 'agent',
          body: 'The decision now covers deleted state, stale pids, and canonical-root matching.',
          createdAt: '2026-08-08T10:10:00Z',
        },
      ],
    ),
    makeThread(
      makeComment(
        'mock-task-open',
        'This task needs a second-session case; reuse and first-page opening interact here.',
        '2026-08-08T10:22:00Z',
        makeAnchor(CHANGE_TASKS_PATH, taskTarget.source, taskTarget.range.start, [
          '2. Hook ownership',
        ]),
      ),
      'open',
      'exact',
      taskTarget.id,
    ),
    makeThread(
      makeComment(
        'mock-spec-resolved',
        'The requirement now distinguishes a foreign root from an unrelated listener.',
        '2026-08-08T10:35:00Z',
        makeAnchor(CHANGE_LIFECYCLE_SPEC_PATH, specTarget.source, specTarget.range.start, [
          'ADDED Requirements',
          'A running dashboard is discovered by probing',
        ]),
      ),
      'resolved',
      'exact',
      specTarget.id,
    ),
    makeThread(
      makeComment(
        'mock-change-orphaned',
        'Keep this concern visible even though the original paragraph was rewritten.',
        '2026-08-08T10:48:00Z',
        makeAnchor(CHANGE_PROPOSAL_PATH, 'The hook opens a tab immediately.', 1400, [
          'What Changes',
        ]),
      ),
      'resolved',
      'orphaned',
      null,
    ),
    makeThread(
      makeComment(
        'mock-change-addressed',
        'The proposal and design now agree on turn-boundary ownership.',
        '2026-08-08T11:03:00Z',
      ),
      'addressed',
      'unanchored',
      null,
    ),
  ],
  commentCounts: { open: 2, addressed: 2, resolved: 2 },
  approval: notApproved(),
  verdicts: [makeVerdict('mock-change-verdict', 'comment-resolution', '2026-08-08T11:15:00Z')],
  standingVerdict: {
    id: 'mock-change-verdict',
    verdict: 'comment-resolution',
    createdAt: '2026-08-08T11:15:00Z',
    directiveDelivered: false,
    directivePending: true,
  },
}

const sessionScope: ScopeDetail = {
  kind: 'session',
  key: MOCK_SESSION_ID,
  title: 'Approval gate: stale reviews and reviewer intent',
  artifacts: [makeArtifact(SESSION_ARTIFACT_PATH, sessionBlocks)],
  comments: [
    makeThread(
      makeComment(
        'mock-session-open',
        'The conclusion is sound, but settle whether tasks.md belongs in the approval fingerprint.',
        '2026-08-08T08:40:00Z',
        makeAnchor(SESSION_ARTIFACT_PATH, sessionTarget.source, sessionTarget.range.start, [
          'Working conclusion',
        ]),
      ),
      'open',
      'fuzzy',
      sessionTarget.id,
    ),
  ],
  commentCounts: { open: 1, addressed: 0, resolved: 0 },
  approval: null,
  verdicts: [makeVerdict('mock-session-verdict', 'keep-exploring', '2026-08-08T08:55:00Z')],
  standingVerdict: {
    id: 'mock-session-verdict',
    verdict: 'keep-exploring',
    createdAt: '2026-08-08T08:55:00Z',
    directiveDelivered: false,
    directivePending: true,
  },
}

const approvalChange = makeSupportingChange({
  key: 'add-change-approval-gate',
  title: 'Add an explicit approval gate before implementation',
  why: 'A change can move from proposal to implementation without a reviewer ever stating that it is ready.',
  change: 'Add explicit approval, stale-approval detection, withdrawal, and an apply precheck',
  decision:
    'Approval is an append-only verdict bound to a fingerprint of proposal, design, and spec deltas; tasks.md is excluded because checkbox progress changes during implementation.',
  tasks: [
    { done: false, label: 'Add approval and withdrawal verdict records' },
    { done: false, label: 'Evaluate approved, stale, and not-approved states' },
    { done: false, label: 'Expose an apply precheck with meaningful exit codes' },
  ],
  specs: [
    {
      capability: 'change-approval',
      requirement: 'Approval is explicit and artifact-bound',
      scenario: 'An edited proposal makes approval stale',
    },
    {
      capability: 'dashboard-html-views',
      requirement: 'Change pages expose approval state',
      scenario: 'Outstanding feedback refuses approval',
    },
  ],
})

const serveControlChange = makeSupportingChange({
  key: 'add-serve-process-control',
  title: 'List and stop detached dashboard processes',
  why: 'Hook-started dashboards survive their parent process, but operators cannot currently enumerate or stop them safely.',
  change: 'Add targeted serve list and serve kill commands backed by the existing identity probe',
  decision:
    'A bare kill command is rejected; a project root, port, pid, or explicit --all is required, and success is verified by probing again.',
  tasks: [
    { done: false, label: 'Extract reusable discovery over the fixed port range' },
    { done: false, label: 'Implement serve list with root, port, and pid' },
    { done: false, label: 'Implement targeted kill and re-probe' },
  ],
  specs: [
    {
      capability: 'dashboard-lifecycle',
      requirement: 'Running dashboards are operable',
      scenario: 'An operator stops one dashboard by project root',
    },
  ],
})

const diagnosticsChange = makeSupportingChange({
  key: 'add-setup-diagnostics',
  title: 'Diagnose broken project hook setup',
  why: 'A fresh clone can have documented hooks and no registered local configuration, producing an empty dashboard that looks healthy.',
  change:
    'Add a doctor command that executes configured hooks against a throwaway root and reports their origin',
  decision:
    'Hook commands are executed rather than merely parsed; only the matcher remains a labelled string comparison because direct execution bypasses dispatch.',
  tasks: [
    { done: false, label: 'Read local and project settings with source attribution' },
    { done: false, label: 'Execute each configured hook with a synthetic payload' },
    { done: false, label: 'Report binary path and version mismatches' },
  ],
  specs: [
    {
      capability: 'setup-diagnostics',
      requirement: 'Project setup failures are actionable',
      scenario: 'A missing prompt hook exits non-zero',
    },
    {
      capability: 'cli-surface',
      requirement: 'Doctor is discoverable from help',
      scenario: 'Top-level help lists doctor',
    },
  ],
})

const vueFoundationChange = makeSupportingChange({
  key: 'add-vue-dashboard-foundation',
  title: 'Embed the first Vue dashboard surface',
  why: 'The dashboard interface must move to Vue without making cargo install depend on a JavaScript toolchain.',
  change:
    'Build the index as a Vue application, embed committed assets, and reject stale frontend output in CI',
  decision:
    'Built assets remain committed and embedded; CI rebuilds them to make staleness executable instead of relying on contributor memory.',
  tasks: [
    { done: true, label: 'Create the Vue and Vite application' },
    { done: true, label: 'Serve the index JSON contract' },
    { done: true, label: 'Embed dist in the Rust binary' },
  ],
  specs: [
    {
      capability: 'dashboard-server',
      requirement: 'Frontend assets are embedded and current',
      scenario: 'A stale dist fails verification',
    },
    {
      capability: 'dashboard-html-views',
      requirement: 'The index renders every discovered scope',
      scenario: 'A fresh binary serves the Vue index offline',
    },
  ],
  verdict: 'comment-resolution',
})

const harnessChange = makeSupportingChange({
  key: 'add-dashboard-development-harness',
  title: 'Add an executable dashboard development harness',
  why: 'The Vue migration cannot be reviewed honestly without format, type, component, browser, and local API feedback.',
  change:
    'Standardize Bun, add frontend quality gates, wire the Vite proxy, and document repository-specific browser verification',
  decision:
    'Use Bun as the only frontend package-manager path and require the embedded browser suite for interaction-heavy changes.',
  tasks: [
    { done: true, label: 'Pin Bun and remove the npm path' },
    { done: true, label: 'Add format, lint, type, component, and browser checks' },
    { done: true, label: 'Document local API and embedded verification' },
  ],
  specs: [
    {
      capability: 'dashboard-development-harness',
      requirement: 'Dashboard changes have executable feedback',
      scenario: 'The full frontend check runs from a clean clone',
    },
  ],
  verdict: 'comment-resolution',
})

const directiveSession = makeSupportingSession(
  '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d75',
  'Prompt-time directive delivery and duplicate suppression',
  [
    'Stop-time delivery prevents the agent from going idle, but prompt-time delivery is the ordinary path after a reviewer verdict.',
    'Exactly one of the two hooks must consume a standing directive; both reading the same record would duplicate work.',
  ],
  'move-to-proposal',
)

const processSession = makeSupportingSession(
  '0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d76',
  'Dashboard process ownership across concurrent sessions',
  [
    'Two sessions can reach a turn boundary simultaneously and race to start the same project dashboard.',
    'The losing process is harmless if both callers verify that one dashboard for the canonical root is serving.',
  ],
  'keep-exploring',
)

const untitledSession = makeSupportingSession('0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d77', null, [
  'The session note exists but deliberately has no level-one heading, so the index must fall back to its exact identifier.',
])

function scopeSummary(scope: ScopeDetail, modifiedAt: string, mostRecentlyActive = false): Scope {
  return {
    key: scope.key,
    title: scope.title,
    modifiedAt,
    openComments: scope.commentCounts.open,
    verdict: scope.verdicts[scope.verdicts.length - 1]?.verdict ?? null,
    mostRecentlyActive,
  }
}

const initialIndex: Index = {
  sessions: [
    scopeSummary(sessionScope, '2026-08-08T08:55:00Z'),
    scopeSummary(directiveSession, '2026-08-07T16:30:00Z'),
    scopeSummary(processSession, '2026-08-07T14:10:00Z'),
    scopeSummary(untitledSession, '2026-08-06T18:05:00Z'),
  ],
  changes: [
    scopeSummary(changeScope, '2026-08-08T11:15:00Z', true),
    scopeSummary(approvalChange, '2026-08-08T10:40:00Z'),
    scopeSummary(serveControlChange, '2026-08-08T09:25:00Z'),
    scopeSummary(diagnosticsChange, '2026-08-07T17:40:00Z'),
    scopeSummary(vueFoundationChange, '2026-08-07T13:20:00Z'),
    scopeSummary(harnessChange, '2026-08-07T11:55:00Z'),
  ],
}

const initialState: MockState = {
  index: initialIndex,
  scopes: {
    [scopeAddress('session', MOCK_SESSION_ID)]: sessionScope,
    [scopeAddress('session', directiveSession.key)]: directiveSession,
    [scopeAddress('session', processSession.key)]: processSession,
    [scopeAddress('session', untitledSession.key)]: untitledSession,
    [scopeAddress('change', MOCK_CHANGE_KEY)]: changeScope,
    [scopeAddress('change', approvalChange.key)]: approvalChange,
    [scopeAddress('change', serveControlChange.key)]: serveControlChange,
    [scopeAddress('change', diagnosticsChange.key)]: diagnosticsChange,
    [scopeAddress('change', vueFoundationChange.key)]: vueFoundationChange,
    [scopeAddress('change', harnessChange.key)]: harnessChange,
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

/**
 * The mock counterpart of the server's approval acts. The precondition and the
 * bulk sweep are modelled rather than stubbed, so the mock lane refuses an
 * approval over outstanding feedback the way the real one does.
 */
export function submitMockApproval(key: string, act: ApprovalAct): ApprovalOutcome | string {
  const scope = requireMockScope('change', key)

  if (act === 'withdraw') {
    if (!scope.approval || scope.approval.state === 'not-approved') {
      return `there is no approval on ${key} to withdraw: ${scope.approval?.reason ?? 'none'}`
    }
    scope.approval = {
      state: 'not-approved',
      reason: `approval was withdrawn on ${new Date().toISOString()}`,
      changedArtifacts: [],
    }
    touchSummary('change', key, new Date().toISOString())
    return { resolved: 0, approval: scope.approval }
  }

  let resolved = 0
  if (act === 'resolve-all-and-approve') {
    for (const thread of scope.comments) {
      if (thread.status === 'resolved') continue
      thread.status = 'resolved'
      resolved += 1
    }
    refreshCounts(scope)
  }

  const { open, addressed } = scope.commentCounts
  if (open || addressed) {
    return `${key} has ${open} open and ${addressed} addressed comment(s) outstanding, so it cannot be approved yet`
  }

  scope.approval = {
    state: 'approved',
    reason: `approved on ${new Date().toISOString()}, and no reviewed artifact has changed since`,
    changedArtifacts: [],
  }
  touchSummary('change', key, new Date().toISOString())
  return { resolved, approval: scope.approval }
}

/**
 * Replaces one block's content and keeps every block id: the mock lane has no anchor resolver,
 * so a fresh id would silently detach the threads anchored to that block and the conversation
 * would empty out instead of showing a rewritten passage under its own comment. Block 1 is the
 * first prose block of every fixture artifact and the one the anchored fixtures point at.
 */
export function rewriteMockArtifact(
  kind: ScopeKind,
  key: string,
  path?: string,
): Artifact | undefined {
  const scope = requireMockScope(kind, key)
  const artifact = path
    ? scope.artifacts.find((candidate) => candidate.path === path)
    : scope.artifacts[0]
  const block = artifact?.blocks[1] ?? artifact?.blocks[0]
  if (!artifact || !block) return undefined

  const source = `Agent rewrote this passage in the mock lane (revision ${++sequence}).`
  block.source = source
  block.html = `<p>${source}</p>`
  block.range = { start: block.range.start, end: block.range.start + source.length }
  // What the reviewer approved is no longer what is on disk, which is the whole
  // point of binding an approval to artifact content.
  if (scope.approval?.state === 'approved') {
    scope.approval = {
      state: 'stale',
      reason: `approved earlier, but ${artifact.path} has changed since`,
      changedArtifacts: [artifact.path],
    }
  }
  touchSummary(kind, key, new Date().toISOString())
  return artifact
}

export function resetMockState(): void {
  state = structuredClone(initialState)
  sequence = 0
}

export { scopeAddress }
