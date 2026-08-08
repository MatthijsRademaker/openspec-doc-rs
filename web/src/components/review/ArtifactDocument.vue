<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import CommentThread from '@/components/review/CommentThread.vue'
import { Button } from '@/components/ui/button'
import type { Artifact, Block, NewComment, Thread } from '@/lib/scope-review'

const props = withDefaults(
  defineProps<{
    artifacts: Artifact[]
    comments: Thread[]
    busy?: boolean
  }>(),
  { busy: false },
)

const emit = defineEmits<{
  comment: [comment: NewComment]
  reply: [commentId: string, body: string]
  status: [commentId: string, status: 'open' | 'resolved']
  composer: [id: string, dirty: boolean]
}>()

interface CommentTarget {
  artifactPath: string
  block: Block
  selectedText: string
  selected: boolean
}

const expanded = ref<string>()
const target = ref<CommentTarget>()
const commentBody = ref('')
const commentComposerDirty = computed(() => Boolean(target.value && commentBody.value.length > 0))

watch(commentComposerDirty, (dirty) => emit('composer', 'artifact-comment', dirty), {
  immediate: true,
})

const commentsByBlock = computed(() => {
  const grouped = new Map<string, Thread[]>()
  for (const thread of props.comments) {
    if (!thread.blockId) continue
    const artifactPath = thread.comment.anchor?.artifactPath
    if (!artifactPath) continue
    const key = `${artifactPath}\u0000${thread.blockId}`
    const current = grouped.get(key) ?? []
    current.push(thread)
    grouped.set(key, current)
  }
  return grouped
})

function artifactLabel(path: string): string {
  if (path.includes('/specs/')) {
    const capability = path.split('/specs/')[1]?.split('/')[0]
    return capability ? `${capability} / spec delta` : path
  }
  return path.split('/').slice(-1)[0]?.replace(/\.md$/, '') ?? path
}

function blockThreads(artifactPath: string, blockId: string): Thread[] {
  return commentsByBlock.value.get(`${artifactPath}\u0000${blockId}`) ?? []
}

function startBlockComment(artifactPath: string, block: Block) {
  target.value = {
    artifactPath,
    block,
    selectedText: block.source,
    selected: false,
  }
  commentBody.value = ''
}

function startSelectionComment(event: MouseEvent, artifactPath: string, block: Block) {
  const container = event.currentTarget
  if (!(container instanceof HTMLElement)) return
  const selection = window.getSelection()
  const selectedText = selection?.toString().trim() ?? ''
  if (!selection || selection.isCollapsed || !selectedText) return
  if (!container.contains(selection.anchorNode) || !container.contains(selection.focusNode)) return

  target.value = { artifactPath, block, selectedText, selected: true }
  commentBody.value = ''
}

function forwardReply(commentId: string, body: string) {
  emit('reply', commentId, body)
}

function forwardStatus(commentId: string, status: 'open' | 'resolved') {
  emit('status', commentId, status)
}

function forwardComposer(commentId: string, dirty: boolean) {
  emit('composer', `artifact-reply:${commentId}`, dirty)
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
  <div v-if="artifacts.length" class="artifact-stack">
    <section
      v-for="(artifact, artifactIndex) in artifacts"
      :key="artifact.path"
      class="artifact-document"
      :aria-labelledby="`artifact-${artifactIndex}`"
    >
      <header class="artifact-document__header">
        <InstrumentLabel>Artifact {{ String(artifactIndex + 1).padStart(2, '0') }}</InstrumentLabel>
        <h2 :id="`artifact-${artifactIndex}`">{{ artifactLabel(artifact.path) }}</h2>
        <code>{{ artifact.path }}</code>
      </header>

      <div class="artifact-document__blocks">
        <div v-for="(block, blockIndex) in artifact.blocks" :key="block.id" class="review-block-row">
          <div
            class="review-block"
            :class="{ 'review-block--commented': blockThreads(artifact.path, block.id).length > 0 }"
            :data-block-id="block.id"
            @mouseup="startSelectionComment($event, artifact.path, block)"
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

            <Button
              type="button"
              variant="instrument"
              size="icon-sm"
              class="review-block__comment-action"
              :aria-label="`Comment on ${artifact.path}, block ${blockIndex + 1}`"
              :disabled="busy"
              @click="startBlockComment(artifact.path, block)"
            >
              +
            </Button>

            <div
              v-if="blockThreads(artifact.path, block.id).length"
              class="review-block__markers"
              aria-label="Comments"
            >
              <button
                v-for="(thread, threadIndex) in blockThreads(artifact.path, block.id)"
                :key="thread.comment.id"
                type="button"
                class="review-block__marker"
                :class="`review-block__marker--${thread.status}`"
                :aria-expanded="expanded === thread.comment.id"
                :aria-controls="`thread-${thread.comment.id}`"
                @click="expanded = expanded === thread.comment.id ? undefined : thread.comment.id"
              >
                {{ threadIndex + 1 }}
                <span class="sr-only">{{ thread.status }} comment</span>
              </button>
            </div>
          </div>

          <div
            v-for="thread in blockThreads(artifact.path, block.id).filter(
              (candidate) => candidate.comment.id === expanded,
            )"
            :id="`thread-${thread.comment.id}`"
            :key="thread.comment.id"
            class="review-block-row__conversation"
          >
            <CommentThread
              :thread="thread"
              :busy="busy"
              @reply="forwardReply"
              @status="forwardStatus"
              @composer="(dirty) => forwardComposer(thread.comment.id, dirty)"
            />
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
      </div>
    </section>
  </div>

  <section v-else class="scope-empty" aria-labelledby="scope-empty-title">
    <InstrumentLabel>Document signal / waiting</InstrumentLabel>
    <h2 id="scope-empty-title">No artifacts written yet</h2>
    <p>Scope exists. Document spine will appear when agent writes first artifact.</p>
  </section>
</template>
