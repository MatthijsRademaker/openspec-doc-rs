<script setup lang="ts">
import type { ComponentPublicInstance } from 'vue'
import { computed, ref, watch } from 'vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import { Button } from '@/components/ui/button'
import { scopeRelativeArtifactPath } from '@/lib/artifact-path'
import type { Artifact, Block, NewComment, Thread } from '@/lib/scope-review'

const props = withDefaults(
  defineProps<{
    artifact?: Artifact
    comments: Thread[]
    requestedPath?: string
    unavailable?: boolean
    activeThreadId?: string
    busy?: boolean
  }>(),
  {
    artifact: undefined,
    requestedPath: undefined,
    unavailable: false,
    activeThreadId: undefined,
    busy: false,
  },
)

const emit = defineEmits<{
  comment: [comment: NewComment]
  activateThread: [threadId: string]
  composer: [id: string, dirty: boolean]
}>()

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

watch(commentComposerDirty, (dirty) => emit('composer', 'artifact-comment', dirty), {
  immediate: true,
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

function setBlockElement(blockId: string, element: Element | ComponentPublicInstance | null): void {
  if (element instanceof HTMLElement) blockElements.set(blockId, element)
  else blockElements.delete(blockId)
}

function navigationBehavior(): ScrollBehavior {
  return typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
    ? 'auto'
    : 'smooth'
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
  target.value = undefined
  commentBody.value = ''
  window.getSelection()?.removeAllRanges()
}
</script>

<template>
  <section v-if="artifact" class="artifact-document" aria-labelledby="selected-artifact-title">
    <header class="artifact-document__header">
      <div class="artifact-document__identity">
        <InstrumentLabel>Selected artifact</InstrumentLabel>
        <h2 id="selected-artifact-title" tabindex="-1">{{ artifactLabel(artifact.path) }}</h2>
        <code :title="artifact.path">{{ scopeRelativeArtifactPath(artifact.path) }}</code>
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
      <div v-for="(block, blockIndex) in artifact.blocks" :key="block.id" class="review-block-row">
        <div
          :ref="(element) => setBlockElement(block.id, element)"
          class="review-block"
          :class="{
            'review-block--commented': blockThreads(block.id).length > 0,
            'review-block--active': blockThreads(block.id).some(
              (thread) => thread.comment.id === activeThreadId,
            ),
          }"
          :data-block-id="block.id"
          tabindex="-1"
          @mouseup="startSelectionComment($event, block)"
        >
          <div class="review-block__content">
            <!-- Rust sanitizes raw HTML before this rendered markdown reaches client. -->
            <table v-if="block.html.startsWith('<tr>')" class="review-block__table">
              <!-- pi-lens-ignore: javascript.vue.security.audit.xss.templates.avoid-v-html.avoid-v-html -->
              <tbody v-html="block.html" />
            </table>
            <!-- pi-lens-ignore: javascript.vue.security.audit.xss.templates.avoid-v-html.avoid-v-html -->
            <div v-else v-html="block.html" />
          </div>

          <div class="review-block__controls">
            <div v-if="blockThreads(block.id).length" class="review-block__markers" aria-label="Comments">
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
              :aria-label="`Comment on ${artifact.path}, block ${blockIndex + 1}`"
              :disabled="busy"
              @click="startBlockComment(block)"
            >
              +
            </Button>
          </div>
        </div>

        <form
          v-if="target?.block.id === block.id"
          class="review-block-row__composer"
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
              Record comment
            </Button>
            <Button type="button" variant="ghost" size="sm" :disabled="busy" @click="cancelComment">
              Cancel
            </Button>
          </div>
        </form>
      </div>
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
