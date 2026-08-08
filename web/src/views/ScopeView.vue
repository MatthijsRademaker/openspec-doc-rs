<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ArtifactDocument from '@/components/review/ArtifactDocument.vue'
import DecisionInstrument from '@/components/review/DecisionInstrument.vue'
import ScopeHeader from '@/components/review/ScopeHeader.vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import StatusMark from '@/components/StatusMark.vue'
import {
  createComment,
  eventPath,
  fetchScope,
  type NewComment,
  replyToComment,
  type ScopeDetail,
  type ScopeKind,
  setCommentStatus,
  submitVerdict,
} from '@/lib/scope-review'
import type { Verdict } from '@/lib/scopes'

const route = useRoute()
const scope = ref<ScopeDetail>()
const failure = ref<string>()
const actionFailure = ref<string>()
const busy = ref(false)
const pendingArtifact = ref<ScopeDetail>()
const pendingArtifactUpdate = ref(false)
const dirtyComposers = ref(new Set<string>())
let events: EventSource | undefined
let loadGeneration = 0

const hasDirtyComposer = computed(() => dirtyComposers.value.size > 0)

const kind = computed<ScopeKind>(() => (route.name === 'session' ? 'session' : 'change'))
const key = computed(() => String(route.params.id ?? route.params.name ?? ''))

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

async function replaceLiveState(
  detail: ScopeDetail,
  artifactsChanged: boolean,
  reviewStateChanged: boolean,
) {
  if (!scope.value) return

  const scrollTop = window.scrollY
  const scrollLeft = window.scrollX
  const next = artifactsChanged
    ? { ...scope.value, artifacts: detail.artifacts, comments: detail.comments }
    : scope.value
  scope.value = reviewStateChanged ? mergeReviewState(next, detail) : next
  await nextTick()

  if (scrollTop || scrollLeft) window.scrollTo(scrollLeft, scrollTop)
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

async function refresh(generation = loadGeneration) {
  const detail = await fetchScope(kind.value, key.value)
  if (generation === loadGeneration) {
    scope.value = detail
    pendingArtifact.value = undefined
    pendingArtifactUpdate.value = false
  }
}

async function refreshReviewState(generation = loadGeneration) {
  const detail = await fetchScope(kind.value, key.value)
  if (generation !== loadGeneration) return

  if (pendingArtifact.value) pendingArtifact.value = mergeReviewState(pendingArtifact.value, detail)
  await replaceLiveState(detail, false, true)
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
  () => [route.name, key.value],
  async () => {
    loadGeneration += 1
    const generation = loadGeneration
    scope.value = undefined
    failure.value = undefined
    actionFailure.value = undefined
    pendingArtifact.value = undefined
    pendingArtifactUpdate.value = false
    dirtyComposers.value = new Set()
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

async function mutate(operation: () => Promise<unknown>) {
  busy.value = true
  actionFailure.value = undefined
  try {
    await operation()
    await refreshReviewState()
  } catch (error) {
    actionFailure.value = describe(error)
  } finally {
    busy.value = false
  }
}

function addComment(comment: NewComment) {
  return mutate(() => createComment(kind.value, key.value, comment))
}

function reply(commentId: string, body: string) {
  return mutate(() => replyToComment(kind.value, key.value, commentId, body))
}

function setStatus(commentId: string, status: 'open' | 'resolved') {
  return mutate(() => setCommentStatus(kind.value, key.value, commentId, status))
}

function submit(verdict: Verdict, comment: string) {
  return mutate(async () => {
    if (comment) {
      await createComment(kind.value, key.value, { kind: 'unanchored', body: comment })
    }
    await submitVerdict(kind.value, key.value, verdict)
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

    <section v-else-if="!scope" class="index-state" aria-live="polite" aria-busy="true">
      <InstrumentLabel>Scope signal / observing</InstrumentLabel>
      <h1 class="scope-register__title">Resolving document coordinates…</h1>
    </section>

    <template v-else>
      <ScopeHeader :scope="scope" />

      <div v-if="actionFailure" class="scope-action-error" role="alert">
        <strong>Action not recorded.</strong>
        <span>{{ actionFailure }}</span>
      </div>

      <div v-if="pendingArtifactUpdate" class="scope-artifact-update" role="status" aria-live="polite">
        <strong>Artifact changed.</strong>
        <span>Finish or dismiss composer to refresh document.</span>
      </div>

      <div class="scope-layout">
        <aside class="scope-utility" aria-label="Scope instruments">
          <section>
            <InstrumentLabel>Comment state</InstrumentLabel>
            <StatusMark kind="open" :label="`${scope.commentCounts.open} open`" />
            <StatusMark kind="addressed" :label="`${scope.commentCounts.addressed} addressed`" />
            <StatusMark kind="resolved" :label="`${scope.commentCounts.resolved} resolved`" />
          </section>
          <nav aria-label="Artifacts">
            <InstrumentLabel>Document coordinates</InstrumentLabel>
            <ol>
              <li v-for="(artifact, index) in scope.artifacts" :key="artifact.path">
                <a :href="`#artifact-${index}`">{{ artifact.path }}</a>
              </li>
            </ol>
          </nav>
        </aside>

        <section class="scope-document" aria-label="Scope artifacts">
          <ArtifactDocument
            :artifacts="scope.artifacts"
            :comments="scope.comments"
            :busy="busy"
            @comment="addComment"
            @reply="reply"
            @status="setStatus"
            @composer="(id, dirty) => setComposerDirty(id, dirty)"
          />
        </section>
      </div>

      <DecisionInstrument
        :scope="scope"
        :busy="busy"
        @submit="submit"
        @reply="reply"
        @status="setStatus"
        @composer="(id, dirty) => setComposerDirty(id, dirty)"
      />
    </template>
  </main>
</template>
