<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import CommentThread from '@/components/review/CommentThread.vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import { Button } from '@/components/ui/button'
import type { ScopeDetail } from '@/lib/scope-review'
import type { Verdict } from '@/lib/scopes'

const props = withDefaults(
  defineProps<{
    scope: ScopeDetail
    busy?: boolean
  }>(),
  { busy: false },
)

const emit = defineEmits<{
  submit: [verdict: Verdict, comment: string]
  reply: [commentId: string, body: string]
  status: [commentId: string, status: 'open' | 'resolved']
  composer: [id: string, dirty: boolean]
}>()

const open = ref(false)
const comment = ref('')
const composerDirty = computed(() => open.value && comment.value.length > 0)

watch(composerDirty, (dirty) => emit('composer', 'decision-comment', dirty), { immediate: true })

const looseThreads = computed(() => props.scope.comments.filter((thread) => !thread.blockId))
const composerVerdict = computed<Verdict>(() =>
  props.scope.kind === 'session' ? 'keep-exploring' : 'comment-resolution',
)

function forwardReply(commentId: string, body: string) {
  emit('reply', commentId, body)
}

function forwardStatus(commentId: string, status: 'open' | 'resolved') {
  emit('status', commentId, status)
}

function forwardComposer(commentId: string, dirty: boolean) {
  emit('composer', `decision-reply:${commentId}`, dirty)
}

function closeComposer() {
  open.value = false
  comment.value = ''
}

function toggleComposer() {
  if (open.value) closeComposer()
  else open.value = true
}

function submitComposer() {
  emit('submit', composerVerdict.value, comment.value.trim())
  comment.value = ''
  open.value = false
}
</script>

<template>
  <aside class="decision-instrument" aria-label="Review decisions">
    <section
      v-if="open"
      id="scope-comment-panel"
      class="decision-instrument__drawer"
      aria-labelledby="scope-comment-panel-title"
    >
      <header class="decision-instrument__drawer-header">
        <div>
          <InstrumentLabel>Comments without block / {{ looseThreads.length }}</InstrumentLabel>
          <h2 id="scope-comment-panel-title">Scope transmission</h2>
        </div>
        <Button type="button" variant="ghost" size="icon-sm" aria-label="Close composer" @click="closeComposer">
          ×
        </Button>
      </header>

      <div v-if="looseThreads.length" class="decision-instrument__loose-comments">
        <CommentThread
          v-for="thread in looseThreads"
          :key="thread.comment.id"
          :thread="thread"
          :busy="busy"
          :show-anchor-loss="thread.anchorState === 'orphaned' || thread.anchorState === 'missing'"
          @reply="forwardReply"
          @status="forwardStatus"
          @composer="(dirty) => forwardComposer(thread.comment.id, dirty)"
        />
      </div>
      <p v-else class="decision-instrument__empty">No unanchored or lost comments.</p>

      <form class="decision-instrument__composer" @submit.prevent="submitComposer">
        <label for="scope-comment">Optional scope comment</label>
        <textarea
          id="scope-comment"
          v-model="comment"
          rows="4"
          :disabled="busy"
          placeholder="Record context beyond anchored comments…"
        />
        <Button type="submit" :disabled="busy">
          {{ scope.kind === 'session' ? 'Keep exploring' : 'Send comments to agent' }}
        </Button>
      </form>
    </section>

    <div class="decision-instrument__bar">
      <Button
        type="button"
        variant="instrument"
        size="lg"
        :aria-expanded="open"
        aria-controls="scope-comment-panel"
        :disabled="busy"
        @click="toggleComposer"
      >
        <span aria-hidden="true">+</span>
        Comment / {{ looseThreads.length }} without block
      </Button>
      <Button
        v-if="scope.kind === 'session'"
        type="button"
        size="lg"
        :disabled="busy"
        @click="emit('submit', 'move-to-proposal', '')"
      >
        Move to proposal
      </Button>
    </div>
  </aside>
</template>
