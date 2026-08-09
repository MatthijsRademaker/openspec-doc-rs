<script setup lang="ts">
import { computed } from 'vue'
import CommentThread from '@/components/review/CommentThread.vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import { Button } from '@/components/ui/button'
import type { Artifact, Thread } from '@/lib/scope-review'

const props = withDefaults(
  defineProps<{
    artifact: Artifact
    threads: Thread[]
    activeThreadId?: string
    busy?: boolean
  }>(),
  { activeThreadId: undefined, busy: false },
)

const emit = defineEmits<{
  activate: [threadId: string]
  reply: [commentId: string, body: string]
  status: [commentId: string, status: 'open' | 'resolved']
  composer: [id: string, dirty: boolean]
}>()

const orderedThreads = computed(() => {
  const blockPositions = new Map(props.artifact.blocks.map((block, index) => [block.id, index]))
  return props.threads
    .map((thread, creationIndex) => ({ thread, creationIndex }))
    .filter(
      ({ thread }) =>
        thread.comment.anchor?.artifactPath === props.artifact.path &&
        thread.blockId !== null &&
        blockPositions.has(thread.blockId),
    )
    .sort((left, right) => {
      const blockOrder =
        (blockPositions.get(left.thread.blockId ?? '') ?? Number.MAX_SAFE_INTEGER) -
        (blockPositions.get(right.thread.blockId ?? '') ?? Number.MAX_SAFE_INTEGER)
      if (blockOrder) return blockOrder
      const creationOrder = left.thread.comment.createdAt.localeCompare(
        right.thread.comment.createdAt,
      )
      return creationOrder || left.creationIndex - right.creationIndex
    })
    .map(({ thread }) => thread)
})
</script>

<template>
  <section class="artifact-conversation" aria-labelledby="artifact-conversation-title">
    <header class="artifact-conversation__header">
      <InstrumentLabel>Artifact conversation / {{ orderedThreads.length }}</InstrumentLabel>
      <h2 id="artifact-conversation-title">Anchored threads</h2>
      <code>{{ artifact.path }}</code>
    </header>

    <div v-if="orderedThreads.length" class="artifact-conversation__threads">
      <article
        v-for="thread in orderedThreads"
        :id="`artifact-thread-${thread.comment.id}`"
        :key="thread.comment.id"
        class="artifact-conversation__thread"
        :class="{ 'artifact-conversation__thread--active': thread.comment.id === activeThreadId }"
        :data-thread-id="thread.comment.id"
        tabindex="-1"
      >
        <Button
          type="button"
          variant="ghost"
          size="sm"
          class="artifact-conversation__anchor-action"
          :aria-label="`Locate source for comment ${thread.comment.id}`"
          @click="emit('activate', thread.comment.id)"
        >
          <span aria-hidden="true">⌖</span>
          Locate source
        </Button>
        <CommentThread
          :thread="thread"
          :busy="busy"
          @reply="(commentId, body) => emit('reply', commentId, body)"
          @status="(commentId, status) => emit('status', commentId, status)"
          @composer="(dirty) => emit('composer', `artifact-reply:${thread.comment.id}`, dirty)"
        />
      </article>
    </div>

    <div v-else class="artifact-conversation__empty">
      <span class="artifact-conversation__crosshair" aria-hidden="true" />
      <p>No threads resolve to selected artifact.</p>
    </div>
  </section>
</template>
