<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import ArtifactConversation from '@/components/review/ArtifactConversation.vue'
import ArtifactDocument from '@/components/review/ArtifactDocument.vue'
import ArtifactNavigator from '@/components/review/ArtifactNavigator.vue'
import DecisionInstrument from '@/components/review/DecisionInstrument.vue'
import ScopeHeader from '@/components/review/ScopeHeader.vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import StatusMark from '@/components/StatusMark.vue'
import { Button } from '@/components/ui/button'
import { createLatestEventChannel, EVENT_TRANSITION_MS } from '@/lib/event-channel'
import {
  createComment,
  eventPath,
  fetchScope,
  type CommentCounts,
  type NewComment,
  replyToComment,
  type ScopeDetail,
  type ScopeKind,
  setCommentStatus,
  submitVerdict,
} from '@/lib/scope-review'
import type {
  MutationOutcome,
  ReviewReceipt,
  TransmissionEvent,
  TriangulationEvent,
} from '@/lib/motion-events'
import type { Verdict } from '@/lib/scopes'
import { prefersReducedMotion, withViewTransition } from '@/lib/view-transition'

interface ArtifactDocumentHandle {
  focusThread: (threadId: string) => void
}

const DOCUMENT_ARRIVAL = 'artifact-content'

const route = useRoute()
const router = useRouter()
const scope = ref<ScopeDetail>()
const failure = ref<string>()
const actionFailure = ref<string>()
const busy = ref(false)
const transmission = ref<TransmissionEvent>()
const pendingArtifact = ref<ScopeDetail>()
const pendingArtifactUpdate = ref(false)
const dirtyComposers = ref(new Set<string>())
const activeThreadId = ref<string>()
const artifactTransitionSource = ref<string>()
const artifactFocusTarget = ref<string>()
const artifactDocument = ref<ArtifactDocumentHandle>()
const conversationCollapsed = ref(false)
// Independent latest-event channels: remote document receipt never erases reviewer confirmation.
const documentArrival = createLatestEventChannel<string>()
const reviewReceipt = createLatestEventChannel<ReviewReceipt>()
const artifactAcquisition = createLatestEventChannel<string>(EVENT_TRANSITION_MS)
const triangulation = createLatestEventChannel<TriangulationEvent>(EVENT_TRANSITION_MS)
const reconfiguration = createLatestEventChannel<'conversation'>(EVENT_TRANSITION_MS)
let events: EventSource | undefined
let loadGeneration = 0

const hasDirtyComposer = computed(() => dirtyComposers.value.size > 0)
const documentReplaced = computed(() => documentArrival.event.value === DOCUMENT_ARRIVAL)

const kind = computed<ScopeKind>(() => (route.name === 'session' ? 'session' : 'change'))
const key = computed(() => String(route.params.id ?? route.params.name ?? ''))
const hasExplicitArtifactSelection = computed(
  () => kind.value === 'change' && 'artifact' in route.query,
)
const requestedArtifactPath = computed(() => {
  if (kind.value === 'session') return scope.value?.artifacts[0]?.path
  const query = route.query.artifact
  return typeof query === 'string' ? query : undefined
})
const selectedArtifact = computed(() => {
  if (!scope.value) return undefined
  if (kind.value === 'session') return scope.value.artifacts[0]
  if (!hasExplicitArtifactSelection.value) return scope.value.artifacts[0]
  return scope.value.artifacts.find((artifact) => artifact.path === requestedArtifactPath.value)
})
const selectionUnavailable = computed(
  () =>
    kind.value === 'change' &&
    hasExplicitArtifactSelection.value &&
    selectedArtifact.value === undefined,
)
const selectedThreads = computed(() => {
  if (!selectedArtifact.value) return []
  const blockIds = new Set(selectedArtifact.value.blocks.map((block) => block.id))
  return (
    scope.value?.comments.filter(
      (thread) =>
        thread.comment.anchor?.artifactPath === selectedArtifact.value?.path &&
        thread.blockId !== null &&
        blockIds.has(thread.blockId),
    ) ?? []
  )
})
const artifactThreadCounts = computed<Record<string, CommentCounts>>(() => {
  const counts: Record<string, CommentCounts> = {}
  for (const artifact of scope.value?.artifacts ?? []) {
    counts[artifact.path] = { open: 0, addressed: 0, resolved: 0 }
  }
  for (const thread of scope.value?.comments ?? []) {
    const path = thread.comment.anchor?.artifactPath
    if (!path || !thread.blockId || !counts[path]) continue
    counts[path][thread.status] += 1
  }
  return counts
})

watch(
  () => selectedArtifact.value?.path,
  (path) => {
    activeThreadId.value = undefined
    if (!path || artifactFocusTarget.value !== path) return

    artifactFocusTarget.value = undefined
    const heading = document.getElementById('selected-artifact-title')
    if (!heading) throw new Error(`Selected artifact heading missing for ${path}`)
    heading.focus({ preventScroll: true })
    heading.scrollIntoView({ block: 'start', behavior: navigationBehavior() })
  },
  { flush: 'post' },
)

function describe(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

interface LiveUpdate {
  artifactsChanged: boolean
  reviewStateChanged: boolean
}

function parseLiveUpdate(data: string): LiveUpdate {
  const payload: unknown = JSON.parse(data)
  if (
    typeof payload !== 'object' ||
    payload === null ||
    typeof (payload as Record<string, unknown>).artifactsChanged !== 'boolean' ||
    typeof (payload as Record<string, unknown>).reviewStateChanged !== 'boolean'
  ) {
    throw new Error('Live update payload has invalid change flags')
  }

  return payload as LiveUpdate
}

function mergeReviewState(target: ScopeDetail, source: ScopeDetail): ScopeDetail {
  return {
    ...target,
    comments: source.comments,
    commentCounts: source.commentCounts,
    verdicts: source.verdicts,
    standingVerdict: source.standingVerdict,
  }
}

function detectReviewReceipt(previous: ScopeDetail, next: ScopeDetail): ReviewReceipt | undefined {
  const previousThreads = new Map(previous.comments.map((thread) => [thread.comment.id, thread]))
  for (const thread of next.comments) {
    const prior = previousThreads.get(thread.comment.id)
    if (!prior) return { kind: 'created-thread', threadId: thread.comment.id, source: 'remote' }
    if (thread.replies.length > prior.replies.length) {
      return { kind: 'extended-thread', threadId: thread.comment.id, source: 'remote' }
    }
    if (thread.status !== prior.status) {
      return {
        kind: 'changed-status',
        threadId: thread.comment.id,
        status: thread.status,
        source: 'remote',
      }
    }
  }

  if (next.standingVerdict?.id !== previous.standingVerdict?.id) {
    return { kind: 'standing-verdict', source: 'remote' }
  }
  if (
    next.standingVerdict &&
    previous.standingVerdict &&
    (next.standingVerdict.directiveDelivered !== previous.standingVerdict.directiveDelivered ||
      next.standingVerdict.directivePending !== previous.standingVerdict.directivePending)
  ) {
    return { kind: 'delivery', source: 'remote' }
  }
  return undefined
}

function receiptForOutcome(outcome: MutationOutcome): ReviewReceipt {
  return { ...outcome, source: 'reviewer' }
}

async function replaceLiveState(
  detail: ScopeDetail,
  artifactsChanged: boolean,
  reviewStateChanged: boolean,
  markRemoteReceipt = true,
) {
  if (!scope.value) return

  const previous = scope.value
  const scrollTop = window.scrollY
  const scrollLeft = window.scrollX
  const next = artifactsChanged
    ? { ...scope.value, artifacts: detail.artifacts, comments: detail.comments }
    : scope.value
  scope.value = reviewStateChanged ? mergeReviewState(next, detail) : next
  if (reviewStateChanged && markRemoteReceipt) {
    const receipt = detectReviewReceipt(previous, scope.value)
    if (receipt) reviewReceipt.signal(receipt)
  }
  await nextTick()

  if (scrollTop || scrollLeft) window.scrollTo(scrollLeft, scrollTop)
  // Marked here rather than where the event arrived, so a deferred update reports at the moment
  // the reviewer's document actually changes.
  if (artifactsChanged) documentArrival.signal(DOCUMENT_ARRIVAL)
}

async function applyPendingArtifactUpdate() {
  const detail = pendingArtifact.value
  if (!detail || hasDirtyComposer.value) return

  pendingArtifact.value = undefined
  pendingArtifactUpdate.value = false
  await replaceLiveState(detail, true, false)
}

function setComposerDirty(id: string, dirty: boolean) {
  const next = new Set(dirtyComposers.value)
  if (dirty) next.add(id)
  else next.delete(id)
  dirtyComposers.value = next
}

watch(hasDirtyComposer, (dirty) => {
  if (!dirty) {
    applyPendingArtifactUpdate().catch((error) => {
      actionFailure.value = `Live update failed: ${describe(error)}`
    })
  }
})

async function canonicalizeArtifactSelection(detail: ScopeDetail, generation: number) {
  if (
    generation !== loadGeneration ||
    kind.value !== 'change' ||
    hasExplicitArtifactSelection.value ||
    !detail.artifacts[0]
  ) {
    return
  }

  await router.replace({
    query: { ...route.query, artifact: detail.artifacts[0].path },
  })
}

async function refresh(generation = loadGeneration) {
  const detail = await fetchScope(kind.value, key.value)
  if (generation === loadGeneration) {
    scope.value = detail
    pendingArtifact.value = undefined
    pendingArtifactUpdate.value = false
    await canonicalizeArtifactSelection(detail, generation)
  }
}

async function refreshReviewState(generation = loadGeneration, markRemoteReceipt = true) {
  const detail = await fetchScope(kind.value, key.value)
  if (generation !== loadGeneration) return

  if (pendingArtifact.value) pendingArtifact.value = mergeReviewState(pendingArtifact.value, detail)
  await replaceLiveState(detail, false, true, markRemoteReceipt)
}

function queueArtifactUpdate(detail: ScopeDetail, reviewStateChanged: boolean) {
  pendingArtifact.value =
    reviewStateChanged || !pendingArtifact.value
      ? detail
      : mergeReviewState(detail, pendingArtifact.value)
  pendingArtifactUpdate.value = true
}

async function reconcileLiveUpdate(detail: ScopeDetail, update: LiveUpdate) {
  if (update.artifactsChanged && hasDirtyComposer.value) {
    queueArtifactUpdate(detail, update.reviewStateChanged)
    if (update.reviewStateChanged) await replaceLiveState(detail, false, true)
    return
  }

  if (update.reviewStateChanged && pendingArtifact.value) {
    pendingArtifact.value = mergeReviewState(pendingArtifact.value, detail)
  }
  await replaceLiveState(detail, update.artifactsChanged, update.reviewStateChanged)
}

function observe() {
  events?.close()
  if (typeof EventSource === 'undefined') return
  events = new EventSource(eventPath(kind.value, key.value))
  events.onmessage = (event) => {
    const generation = loadGeneration
    void fetchScope(kind.value, key.value)
      .then((detail) => {
        if (generation === loadGeneration)
          return reconcileLiveUpdate(detail, parseLiveUpdate(event.data))
      })
      .catch((error) => {
        if (generation === loadGeneration) {
          actionFailure.value = `Live update failed: ${describe(error)}`
        }
      })
  }
  events.onerror = () => {
    actionFailure.value = 'Live update channel disconnected. Reopen scope to reconnect.'
  }
}

watch(
  [() => route.name, key],
  async () => {
    loadGeneration += 1
    const generation = loadGeneration
    scope.value = undefined
    failure.value = undefined
    actionFailure.value = undefined
    busy.value = false
    transmission.value = undefined
    pendingArtifact.value = undefined
    pendingArtifactUpdate.value = false
    dirtyComposers.value = new Set()
    activeThreadId.value = undefined
    artifactTransitionSource.value = undefined
    artifactFocusTarget.value = undefined
    documentArrival.clear()
    reviewReceipt.clear()
    artifactAcquisition.clear()
    triangulation.clear()
    reconfiguration.clear()
    events?.close()
    try {
      await refresh(generation)
      if (generation === loadGeneration) observe()
    } catch (error) {
      if (generation === loadGeneration) failure.value = describe(error)
    }
  },
  { immediate: true },
)

onBeforeUnmount(() => events?.close())

async function mutate(
  nextTransmission: TransmissionEvent,
  operation: () => Promise<MutationOutcome>,
): Promise<void> {
  busy.value = true
  transmission.value = nextTransmission
  actionFailure.value = undefined
  reviewReceipt.clear()
  try {
    const outcome = await operation()
    await refreshReviewState(loadGeneration, false)
    reviewReceipt.signal(receiptForOutcome(outcome))
  } catch (error) {
    actionFailure.value = describe(error)
  } finally {
    transmission.value = undefined
    busy.value = false
  }
}

function navigationBehavior(): ScrollBehavior {
  return prefersReducedMotion() ? 'auto' : 'smooth'
}

async function selectArtifact(path: string) {
  if (kind.value !== 'change') return
  if (path === requestedArtifactPath.value) {
    document.getElementById('selected-artifact-title')?.focus()
    return
  }

  activeThreadId.value = undefined
  artifactFocusTarget.value = path
  artifactTransitionSource.value = path
  // Flush initiating coordinate before native transition captures old pixels. Without this tick,
  // browser sees only destination name and falls back to an unrelated root crossfade.
  await nextTick()
  try {
    await withViewTransition(async () => {
      await router.push({ query: { ...route.query, artifact: path } })
      artifactTransitionSource.value = undefined
      artifactAcquisition.signal(path)
      await nextTick()
    })
  } finally {
    artifactTransitionSource.value = undefined
    if (artifactFocusTarget.value === path) artifactFocusTarget.value = undefined
  }
}

function toggleConversation(): void {
  reconfiguration.signal('conversation')
  conversationCollapsed.value = !conversationCollapsed.value
}

async function activateThreadFromMarker(threadId: string) {
  activeThreadId.value = threadId
  triangulation.signal({ threadId, origin: 'source', destination: 'thread' })
  await nextTick()
  const thread = document.getElementById(`artifact-thread-${threadId}`)
  thread?.focus({ preventScroll: true })
  thread?.scrollIntoView({ block: 'center', behavior: navigationBehavior() })
}

async function activateThreadFromConversation(threadId: string) {
  activeThreadId.value = threadId
  triangulation.signal({ threadId, origin: 'thread', destination: 'source' })
  await nextTick()
  artifactDocument.value?.focusThread(threadId)
}

function addComment(comment: NewComment) {
  return mutate({ kind: 'comment', target: 'artifact-comment' }, async () => ({
    kind: 'created-thread',
    threadId: (await createComment(kind.value, key.value, comment)).id,
  }))
}

function reply(commentId: string, body: string) {
  return mutate({ kind: 'reply', threadId: commentId }, async () => {
    await replyToComment(kind.value, key.value, commentId, body)
    return { kind: 'extended-thread', threadId: commentId }
  })
}

function setStatus(commentId: string, status: 'open' | 'resolved') {
  return mutate({ kind: 'status', threadId: commentId }, async () => {
    await setCommentStatus(kind.value, key.value, commentId, status)
    return { kind: 'changed-status', threadId: commentId, status }
  })
}

function submit(verdict: Verdict, comment: string) {
  return mutate({ kind: 'verdict', target: 'standing-verdict' }, async () => {
    if (comment) {
      await createComment(kind.value, key.value, { kind: 'unanchored', body: comment })
    }
    await submitVerdict(kind.value, key.value, verdict)
    return { kind: 'standing-verdict' }
  })
}
</script>

<template>
  <main class="scope-workbench">
    <section v-if="failure" class="index-state index-state--failure" role="alert">
      <InstrumentLabel>Scope signal / failed</InstrumentLabel>
      <h1 class="scope-register__title">Scope unavailable</h1>
      <p class="index-state__message">{{ failure }}</p>
    </section>

    <section
      v-else-if="!scope"
      class="index-state scope-route-acquisition"
      data-motion-event="acquire"
      aria-live="polite"
      aria-busy="true"
    >
      <InstrumentLabel>Scope signal / acquiring</InstrumentLabel>
      <h1 class="scope-register__title">Acquiring route coordinate</h1>
      <code class="scope-route-acquisition__key">{{ key }}</code>
    </section>

    <template v-else>
      <ScopeHeader :scope="scope" :receipt="reviewReceipt.event.value" />

      <div v-if="actionFailure" class="scope-action-error" role="alert">
        <strong>Action not recorded.</strong>
        <span>{{ actionFailure }}</span>
      </div>

      <div v-if="pendingArtifactUpdate" class="scope-artifact-update" role="status" aria-live="polite">
        <strong>Artifact changed.</strong>
        <span>Finish or dismiss composer to refresh document.</span>
      </div>

      <div
        class="scope-layout"
        :class="{
          'scope-layout--conversation-collapsed': conversationCollapsed,
          'scope-layout--triangulating-to-source': triangulation.event.value?.destination === 'source',
          'scope-layout--triangulating-to-thread': triangulation.event.value?.destination === 'thread',
        }"
      >
        <span
          v-if="triangulation.event.value"
          class="scope-layout__direction-trace"
          data-motion-event="triangulate"
          aria-hidden="true"
        />
        <aside class="scope-utility" aria-label="Scope instruments">
          <section class="scope-utility__route">
            <InstrumentLabel>Route coordinate</InstrumentLabel>
            <RouterLink to="/" class="scope-index-link">
              <span aria-hidden="true">←</span>
              Observation index
            </RouterLink>
          </section>
          <section class="scope-utility__state">
            <InstrumentLabel>Comment state / all artifacts</InstrumentLabel>
            <StatusMark kind="open" :label="`${scope.commentCounts.open} open`" />
            <StatusMark kind="addressed" :label="`${scope.commentCounts.addressed} addressed`" />
            <StatusMark kind="resolved" :label="`${scope.commentCounts.resolved} resolved`" />
          </section>
          <ArtifactNavigator
            :paths="scope.artifacts.map((artifact) => artifact.path)"
            :selected-path="selectedArtifact?.path"
            :thread-counts="artifactThreadCounts"
            :interactive="scope.kind === 'change'"
            :acquiring-path="artifactTransitionSource"
            @select="selectArtifact"
          />
          <div class="scope-utility__art" aria-hidden="true">
            <img src="/assets/images/observatory-task-updated.webp" alt="" />
          </div>
        </aside>

        <section class="scope-document" aria-label="Selected artifact">
          <ArtifactDocument
            ref="artifactDocument"
            :artifact="selectedArtifact"
            :comments="selectedThreads"
            :requested-path="requestedArtifactPath"
            :unavailable="selectionUnavailable"
            :active-thread-id="activeThreadId"
            :triangulation="triangulation.event.value"
            :acquired="artifactAcquisition.event.value === selectedArtifact?.path"
            :replaced="documentReplaced"
            :transmission="transmission"
            :receipt="reviewReceipt.event.value"
            :busy="busy"
            @comment="addComment"
            @activate-thread="activateThreadFromMarker"
            @composer="(id, dirty) => setComposerDirty(id, dirty)"
          />
        </section>

        <aside
          class="scope-conversation"
          :class="{
            'scope-conversation--collapsed': conversationCollapsed,
            'scope-conversation--reconfiguring':
              reconfiguration.event.value === 'conversation',
          }"
          :data-motion-event="
            reconfiguration.event.value === 'conversation' ? 'reconfigure' : undefined
          "
          aria-label="Artifact conversation"
        >
          <Button
            type="button"
            variant="instrument"
            size="sm"
            class="scope-conversation__toggle"
            :aria-expanded="!conversationCollapsed"
            aria-controls="scope-conversation-body"
            :aria-label="conversationCollapsed ? 'Expand anchored threads' : 'Collapse anchored threads'"
            @click="toggleConversation"
          >
            <span aria-hidden="true">{{ conversationCollapsed ? '←' : '→' }}</span>
            <span class="scope-conversation__toggle-label">Threads</span>
          </Button>

          <div class="scope-conversation__frame">
            <div v-show="!conversationCollapsed" id="scope-conversation-body" class="scope-conversation__body">
              <ArtifactConversation
                v-if="selectedArtifact"
                :artifact="selectedArtifact"
                :threads="selectedThreads"
                :active-thread-id="activeThreadId"
                :triangulation="triangulation.event.value"
                :transmission="transmission"
                :receipt="reviewReceipt.event.value"
                :busy="busy"
                @activate="activateThreadFromConversation"
                @reply="reply"
                @status="setStatus"
                @composer="(id, dirty) => setComposerDirty(id, dirty)"
              />
              <section v-else class="artifact-conversation artifact-conversation--idle">
                <InstrumentLabel>Artifact conversation / idle</InstrumentLabel>
                <p>Select an available artifact coordinate to inspect its resolved threads.</p>
              </section>

              <div class="scope-conversation__art" aria-hidden="true">
                <img src="/assets/images/observatory-comment-updated.webp" alt="" />
              </div>
            </div>

            <DecisionInstrument
              :scope="scope"
              :transmission="transmission"
              :receipt="reviewReceipt.event.value"
              :busy="busy"
              @submit="submit"
              @reply="reply"
              @status="setStatus"
              @composer="(id, dirty) => setComposerDirty(id, dirty)"
            />
          </div>
        </aside>
      </div>
    </template>
  </main>
</template>
