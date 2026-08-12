<script setup lang="ts">
import type { ComponentPublicInstance } from 'vue'
import { computed, ref, watch } from 'vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import { Button } from '@/components/ui/button'
import { scopeRelativeArtifactPath } from '@/lib/artifact-path'
import type { ReviewReceipt, TransmissionEvent, TriangulationEvent } from '@/lib/motion-events'
import type { Artifact, Block, NewComment, Thread } from '@/lib/scope-review'
import { prefersReducedMotion } from '@/lib/view-transition'

const props = withDefaults(
  defineProps<{
    artifact?: Artifact
    comments: Thread[]
    requestedPath?: string
    unavailable?: boolean
    activeThreadId?: string
    triangulation?: TriangulationEvent
    acquired?: boolean
    replaced?: boolean
    transmission?: TransmissionEvent
    receipt?: ReviewReceipt
    busy?: boolean
  }>(),
  {
    artifact: undefined,
    requestedPath: undefined,
    unavailable: false,
    activeThreadId: undefined,
    triangulation: undefined,
    acquired: false,
    replaced: false,
    transmission: undefined,
    receipt: undefined,
    busy: false,
  },
)

const emit = defineEmits<{
  comment: [comment: NewComment]
  activateThread: [threadId: string]
  composer: [id: string, dirty: boolean]
}>()

interface Section {
  id: string
  blocks: Block[]
}

interface CommentTarget {
  artifactPath: string
  block: Block
  selectedText: string
  selected: boolean
}

const target = ref<CommentTarget>()
const commentBody = ref('')
const blockElements = new Map<string, HTMLElement>()
const commentComposerDirty = computed(() => Boolean(target.value && commentBody.value.length > 0))
const isTransmittingComment = computed(
  () => props.transmission?.kind === 'comment' && props.transmission.target === 'artifact-comment',
)

watch(commentComposerDirty, (dirty) => emit('composer', 'artifact-comment', dirty), {
  immediate: true,
})
watch(
  () => props.receipt,
  (receipt) => {
    if (receipt?.source === 'reviewer' && receipt.kind === 'created-thread' && target.value) {
      cancelComment()
      window.getSelection()?.removeAllRanges()
    }
  },
)

// A heading and the blocks that follow it are one readable unit, so the document is grouped into
// sections rather than printing one bordered row per block. Blocks stay individually anchorable;
// only the separator moves outward, to the section boundary.
const sections = computed<Section[]>(() => {
  const grouped: Section[] = []
  for (const block of props.artifact?.blocks ?? []) {
    const current = grouped[grouped.length - 1]
    if (!current || /^<h[1-6][\s>]/.test(block.html)) {
      grouped.push({ id: block.id, blocks: [block] })
    } else {
      current.blocks.push(block)
    }
  }
  return grouped
})

const blockNumbers = computed(() => {
  const numbers = new Map<string, number>()
  for (const [index, block] of (props.artifact?.blocks ?? []).entries()) {
    numbers.set(block.id, index + 1)
  }
  return numbers
})

const commentsByBlock = computed(() => {
  const grouped = new Map<string, Thread[]>()
  if (!props.artifact) return grouped

  for (const thread of props.comments) {
    if (!thread.blockId || thread.comment.anchor?.artifactPath !== props.artifact.path) continue
    const current = grouped.get(thread.blockId) ?? []
    current.push(thread)
    grouped.set(thread.blockId, current)
  }
  return grouped
})

function artifactLabel(path: string): string {
  if (path.startsWith('.openspec-doc/scratch/')) return 'Exploration scratch'
  return scopeRelativeArtifactPath(path)
}

function blockThreads(blockId: string): Thread[] {
  return commentsByBlock.value.get(blockId) ?? []
}

function triangulationRole(blockId: string): 'origin' | 'destination' | undefined {
  if (!props.triangulation) return undefined
  const includesTarget = blockThreads(blockId).some(
    (thread) => thread.comment.id === props.triangulation?.threadId,
  )
  if (!includesTarget) return undefined
  if (props.triangulation.origin === 'source') return 'origin'
  if (props.triangulation.destination === 'source') return 'destination'
  return undefined
}

function setBlockElement(blockId: string, element: Element | ComponentPublicInstance | null): void {
  if (element instanceof HTMLElement) blockElements.set(blockId, element)
  else blockElements.delete(blockId)
}

function navigationBehavior(): ScrollBehavior {
  return prefersReducedMotion() ? 'auto' : 'smooth'
}

function focusThread(threadId: string): void {
  const thread = props.comments.find((candidate) => candidate.comment.id === threadId)
  if (!thread?.blockId) return
  const block = blockElements.get(thread.blockId)
  if (!block) return
  block.focus({ preventScroll: true })
  block.scrollIntoView({ block: 'center', behavior: navigationBehavior() })
}

defineExpose({ focusThread })

function startBlockComment(block: Block) {
  if (!props.artifact) return
  target.value = {
    artifactPath: props.artifact.path,
    block,
    selectedText: block.source,
    selected: false,
  }
  commentBody.value = ''
}

function startSelectionComment(event: MouseEvent, block: Block) {
  if (!props.artifact) return
  const container = event.currentTarget
  if (!(container instanceof HTMLElement)) return
  const selection = window.getSelection()
  const selectedText = selection?.toString().trim() ?? ''
  if (!selection || selection.isCollapsed || !selectedText) return
  if (!container.contains(selection.anchorNode) || !container.contains(selection.focusNode)) return

  target.value = { artifactPath: props.artifact.path, block, selectedText, selected: true }
  commentBody.value = ''
}

function cancelComment() {
  target.value = undefined
  commentBody.value = ''
}

function submitComment() {
  if (!target.value || !commentBody.value.trim()) return
  emit('comment', {
    kind: 'anchored',
    artifactPath: target.value.artifactPath,
    selectedText: target.value.selectedText,
    searchFrom: target.value.block.range.start,
    body: commentBody.value.trim(),
  })
}
</script>

<template>
  <section
    v-if="artifact"
    class="artifact-document"
    :class="{
      'artifact-document--acquired': acquired,
      'artifact-document--replaced': replaced,
    }"
    :data-motion-event="acquired ? 'acquire' : replaced ? 'receive' : undefined"
    aria-labelledby="selected-artifact-title"
  >
    <header class="artifact-document__header">
      <div class="artifact-document__identity">
        <InstrumentLabel>Selected artifact</InstrumentLabel>
        <h2 id="selected-artifact-title" tabindex="-1">{{ artifactLabel(artifact.path) }}</h2>
        <div class="artifact-document__coordinate">
          <code :title="artifact.path">{{ scopeRelativeArtifactPath(artifact.path) }}</code>
          <span
            v-if="acquired"
            class="artifact-document__coordinate-lock"
            aria-hidden="true"
          />
        </div>
        <!-- The report is text as well as an edge, so reduced motion still reports the arrival. It
             names the replacement and nothing about which blocks differ: block identity across a
             rewrite belongs to the anchor resolver. The region is always present — a live region
             created together with its text is not announced — and reserves its line, so the report
             appearing moves no prose. -->
        <p class="artifact-document__replacement" role="status">
          <Transition name="arrival">
            <span v-if="replaced">Document content replaced</span>
          </Transition>
        </p>
      </div>
      <div class="artifact-document__arrival-art" aria-hidden="true">
        <img
          src="/assets/images/observatory-field.webp"
          alt=""
          aria-hidden="true"
          class="artifact-document__arrival-image"
        />
      </div>
    </header>

    <div class="artifact-document__blocks">
      <section v-for="section in sections" :key="section.id" class="review-section">
        <div v-for="block in section.blocks" :key="block.id" class="review-block-row">
          <div
            :ref="(element) => setBlockElement(block.id, element)"
            class="review-block"
            :class="{
              'review-block--commented': blockThreads(block.id).length > 0,
              'review-block--active': blockThreads(block.id).some(
                (thread) => thread.comment.id === activeThreadId,
              ),
              'review-block--triangulation-origin': triangulationRole(block.id) === 'origin',
              'review-block--triangulation-destination':
                triangulationRole(block.id) === 'destination',
            }"
            :data-motion-event="triangulationRole(block.id) ? 'triangulate' : undefined"
            :data-block-id="block.id"
            tabindex="-1"
            @mouseup="startSelectionComment($event, block)"
          >
            <!-- Rust sanitizes raw HTML before this rendered markdown reaches client. -->
            <!-- pi-lens-ignore: javascript.vue.security.audit.xss.templates.avoid-v-html.avoid-v-html -->
            <div class="review-block__content" v-html="block.html" />

            <div class="review-block__controls">
              <div
                v-if="blockThreads(block.id).length"
                class="review-block__markers"
                aria-label="Comments"
              >
                <button
                  v-for="(thread, threadIndex) in blockThreads(block.id)"
                  :key="thread.comment.id"
                  type="button"
                  class="review-block__marker"
                  :class="[
                    `review-block__marker--${thread.status}`,
                    { 'review-block__marker--active': thread.comment.id === activeThreadId },
                  ]"
                  :aria-current="thread.comment.id === activeThreadId ? 'true' : undefined"
                  :aria-controls="`artifact-thread-${thread.comment.id}`"
                  @click="emit('activateThread', thread.comment.id)"
                >
                  {{ threadIndex + 1 }}
                  <span class="sr-only">{{ thread.status }} comment</span>
                </button>
              </div>

              <Button
                type="button"
                variant="instrument"
                size="icon-sm"
                class="review-block__comment-action"
                :aria-label="`Comment on ${artifact.path}, block ${blockNumbers.get(block.id)}`"
                :disabled="busy"
                @click="startBlockComment(block)"
              >
                +
              </Button>
            </div>
          </div>

          <!-- The slot exists so the composer can expand: it displaces the document by its own
               height, and an interpolated row makes that read as the composer opening rather than as
               the page jolting. -->
          <Transition name="composer">
            <div v-if="target?.block.id === block.id" class="review-block-row__composer-slot">
              <form
                class="review-block-row__composer"
                :class="{ 'review-block-row__composer--transmitting': isTransmittingComment }"
                :data-motion-event="isTransmittingComment ? 'transmit' : undefined"
                :aria-busy="isTransmittingComment"
                @submit.prevent="submitComment"
              >
                <label :for="`comment-${block.id}`">
                  {{ target.selected ? 'Comment on selected text' : 'Comment on block' }}
                </label>
                <blockquote>{{ target.selectedText }}</blockquote>
                <textarea
                  :id="`comment-${block.id}`"
                  v-model="commentBody"
                  rows="4"
                  required
                  autofocus
                  :disabled="busy"
                />
                <div class="comment-thread__actions">
                  <Button type="submit" size="sm" :disabled="busy || !commentBody.trim()">
                    {{ isTransmittingComment ? 'Transmitting comment…' : 'Record comment' }}
                  </Button>
                  <span v-if="isTransmittingComment" class="transmission-status" role="status">
                    Transmitting comment to review record
                  </span>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    :disabled="busy"
                    @click="cancelComment"
                  >
                    Cancel
                  </Button>
                </div>
              </form>
            </div>
          </Transition>
        </div>
      </section>
    </div>
  </section>

  <section v-else-if="unavailable" class="scope-empty scope-empty--unavailable" aria-labelledby="artifact-unavailable-title">
    <InstrumentLabel>Document signal / unavailable</InstrumentLabel>
    <h2 id="artifact-unavailable-title">Selected artifact unavailable</h2>
    <code>{{ requestedPath }}</code>
    <p>Requested exact coordinate does not exist in current scope snapshot.</p>
  </section>

  <section v-else class="scope-empty" aria-labelledby="scope-empty-title">
    <InstrumentLabel>Document signal / waiting</InstrumentLabel>
    <h2 id="scope-empty-title">No artifacts written yet</h2>
    <p>Scope exists. Document stage will appear when agent writes first artifact.</p>
  </section>
</template>
