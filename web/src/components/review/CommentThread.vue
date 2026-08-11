<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import StatusMark from '@/components/StatusMark.vue'
import { Button } from '@/components/ui/button'
import { age } from '@/lib/age'
import type { ReviewReceipt, TransmissionEvent } from '@/lib/motion-events'
import type { Thread } from '@/lib/scope-review'

const props = withDefaults(
  defineProps<{
    thread: Thread
    transmission?: TransmissionEvent
    receipt?: ReviewReceipt
    busy?: boolean
    showAnchorLoss?: boolean
  }>(),
  { transmission: undefined, receipt: undefined, busy: false, showAnchorLoss: false },
)

const emit = defineEmits<{
  reply: [commentId: string, body: string]
  status: [commentId: string, status: 'open' | 'resolved']
  composer: [dirty: boolean]
}>()

const replying = ref(false)
const replyBody = ref('')
const replyComposerDirty = computed(() => replying.value && replyBody.value.length > 0)
const isTransmittingReply = computed(
  () =>
    props.transmission?.kind === 'reply' && props.transmission.threadId === props.thread.comment.id,
)
const isTransmittingStatus = computed(
  () =>
    props.transmission?.kind === 'status' &&
    props.transmission.threadId === props.thread.comment.id,
)
const receivedStatus = computed(
  () =>
    props.receipt?.kind === 'changed-status' && props.receipt.threadId === props.thread.comment.id,
)

watch(replyComposerDirty, (dirty) => emit('composer', dirty), { immediate: true })
watch(
  () => props.receipt,
  (receipt) => {
    if (
      receipt?.source === 'reviewer' &&
      receipt.kind === 'extended-thread' &&
      receipt.threadId === props.thread.comment.id
    ) {
      cancelReply()
    }
  },
)

function cancelReply() {
  replying.value = false
  replyBody.value = ''
}

function submitReply() {
  const body = replyBody.value.trim()
  if (!body) return
  emit('reply', props.thread.comment.id, body)
}
</script>

<template>
  <article
    class="comment-thread"
    :class="{
      'comment-thread--transmitting': isTransmittingReply || isTransmittingStatus,
      'comment-thread--received-status': receivedStatus,
    }"
    :data-motion-event="
      isTransmittingReply || isTransmittingStatus ? 'transmit' : receivedStatus ? 'resolve' : undefined
    "
    :aria-busy="isTransmittingReply || isTransmittingStatus"
    :aria-labelledby="`comment-${thread.comment.id}`"
  >
    <header class="comment-thread__header">
      <StatusMark
        :kind="thread.status"
        :label="thread.status"
        :event="receivedStatus ? 'resolve' : undefined"
      />
      <span v-if="thread.anchorState === 'fuzzy'" class="comment-thread__moved">↝ Anchor moved</span>
    </header>

    <div class="comment-message comment-message--reviewer">
      <div class="comment-message__meta">
        <StatusMark kind="reviewer" label="Reviewer" />
        <time :datetime="thread.comment.createdAt">{{ age(thread.comment.createdAt) }}</time>
      </div>
      <p :id="`comment-${thread.comment.id}`">{{ thread.comment.body }}</p>
    </div>

    <div v-if="showAnchorLoss && thread.comment.anchor" class="comment-thread__lost" role="note">
      <strong>Anchor lost</strong>
      <blockquote>{{ thread.comment.anchor.selectedText }}</blockquote>
    </div>

    <div
      v-for="reply in thread.replies"
      :key="reply.id"
      class="comment-message"
      :class="`comment-message--${reply.author}`"
    >
      <div class="comment-message__meta">
        <StatusMark
          :kind="reply.author"
          :label="reply.author === 'agent' ? 'Agent' : 'Reviewer'"
        />
        <time :datetime="reply.createdAt">{{ age(reply.createdAt) }}</time>
      </div>
      <p>{{ reply.body }}</p>
    </div>

    <p v-if="isTransmittingReply" class="transmission-status" role="status">
      Transmitting reply to thread
    </p>
    <p v-else-if="isTransmittingStatus" class="transmission-status" role="status">
      Transmitting comment status
    </p>

    <p v-if="thread.status === 'addressed'" class="comment-thread__claim">
      Agent claims work complete. Review response, then accept or reopen.
    </p>

    <form v-if="replying" class="comment-thread__reply" @submit.prevent="submitReply">
      <label :for="`reply-${thread.comment.id}`">Reply to thread</label>
      <textarea
        :id="`reply-${thread.comment.id}`"
        v-model="replyBody"
        rows="3"
        required
        :disabled="busy"
      />
      <div class="comment-thread__actions">
        <Button type="submit" size="sm" :disabled="busy || !replyBody.trim()">
          {{ isTransmittingReply ? 'Transmitting reply…' : 'Record reply' }}
        </Button>
        <Button type="button" variant="ghost" size="sm" :disabled="busy" @click="cancelReply">
          Cancel
        </Button>
      </div>
    </form>

    <footer v-else class="comment-thread__actions">
      <Button
        v-if="thread.status === 'open'"
        type="button"
        variant="ghost"
        size="sm"
        :disabled="busy"
        @click="replying = true"
      >
        Reply
      </Button>
      <Button
        v-if="thread.status === 'open'"
        type="button"
        variant="instrument"
        size="sm"
        :disabled="busy"
        @click="emit('status', thread.comment.id, 'resolved')"
      >
        Resolve
      </Button>
      <Button
        v-if="thread.status === 'addressed'"
        type="button"
        variant="instrument"
        size="sm"
        :disabled="busy"
        @click="emit('status', thread.comment.id, 'resolved')"
      >
        Accept as resolved
      </Button>
      <Button
        v-if="thread.status !== 'open'"
        type="button"
        variant="ghost"
        size="sm"
        :disabled="busy"
        @click="emit('status', thread.comment.id, 'open')"
      >
        Reopen
      </Button>
    </footer>
  </article>
</template>
