<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import CommentThread from '@/components/review/CommentThread.vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import { Button } from '@/components/ui/button'
import type { ScopeDetail } from '@/lib/scope-review'
import type { Verdict } from '@/lib/scopes'

const props = withDefaults(
  defineProps<{
    scope: ScopeDetail
    recordedThreadId?: string
    busy?: boolean
  }>(),
  { recordedThreadId: undefined, busy: false },
)

const emit = defineEmits<{
  submit: [verdict: Verdict, comment: string]
  reply: [commentId: string, body: string]
  status: [commentId: string, status: 'open' | 'resolved']
  composer: [id: string, dirty: boolean]
}>()

const open = ref(false)
const comment = ref('')
const commentPanel = ref<HTMLElement>()
const composerDirty = computed(() => open.value && comment.value.length > 0)

watch(composerDirty, (dirty) => emit('composer', 'decision-comment', dirty), { immediate: true })

const looseThreads = computed(() => props.scope.comments.filter((thread) => !thread.blockId))
const scopeNoun = computed(() => (props.scope.kind === 'session' ? 'exploration' : 'change'))
const looseNoteCount = computed(
  () => `${looseThreads.value.length} unplaced note${looseThreads.value.length === 1 ? '' : 's'}`,
)
const triggerLabel = computed(() =>
  open.value ? 'Close scope feedback' : `Open scope feedback, ${looseNoteCount.value}`,
)
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
  const restoreFocus = open.value
  open.value = false
  comment.value = ''
  if (restoreFocus) {
    void nextTick(() => document.getElementById('scope-comment-trigger')?.focus())
  }
}

async function toggleComposer() {
  if (open.value) closeComposer()
  else {
    open.value = true
    await nextTick()
    // The panel, not the composer. Focusing the textarea scrolls this panel to its own foot,
    // hiding the heading the dialog is named by. `preventScroll` or a scroll-back-to-top would
    // only split keyboard focus from the visible region, so the first Tab reads as a jump
    // backwards through content the reviewer has not seen.
    commentPanel.value?.focus()
  }
}

function submitComposer() {
  emit('submit', composerVerdict.value, comment.value.trim())
  closeComposer()
}

function trapFocus(event: KeyboardEvent) {
  if (event.key !== 'Tab' || !commentPanel.value) return
  const focusable = Array.from(
    commentPanel.value.querySelectorAll<HTMLElement>(
      'button:not([disabled]), textarea:not([disabled]), [href], [tabindex]:not([tabindex="-1"])',
    ),
  )
  const first = focusable[0]
  const last = focusable[focusable.length - 1]
  if (!first || !last) return
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
}
</script>

<template>
  <aside class="decision-instrument" aria-label="Review decisions">
    <div class="decision-instrument__bar">
      <Button
        id="scope-comment-trigger"
        type="button"
        variant="instrument"
        size="lg"
        :aria-expanded="open"
        :aria-label="triggerLabel"
        aria-controls="scope-comment-panel"
        :disabled="busy"
        @click="toggleComposer"
      >
        <span aria-hidden="true">{{ open ? '×' : '+' }}</span>
        <span class="decision-instrument__label">
          {{ open ? 'Close scope feedback' : `Scope feedback · ${looseThreads.length}` }}
        </span>
      </Button>
      <Button
        v-if="scope.kind === 'session'"
        type="button"
        size="lg"
        aria-label="Move to proposal"
        :disabled="busy"
        @click="emit('submit', 'move-to-proposal', '')"
      >
        <span aria-hidden="true">→</span>
        <span class="decision-instrument__label">Move to proposal</span>
      </Button>
    </div>
  </aside>

  <Teleport to="body">
    <Transition name="drawer-backdrop">
      <div
        v-if="open"
        class="decision-instrument__backdrop"
        aria-hidden="true"
        @click="closeComposer"
      />
    </Transition>

    <Transition name="drawer">
      <section
        v-if="open"
        id="scope-comment-panel"
        ref="commentPanel"
        class="decision-instrument__drawer"
        role="dialog"
        aria-modal="true"
        aria-labelledby="scope-comment-panel-title"
        tabindex="-1"
        @keydown="trapFocus"
        @keydown.esc.stop.prevent="closeComposer"
      >
        <header class="decision-instrument__drawer-header">
          <div>
            <InstrumentLabel>Scope feedback / {{ looseNoteCount }}</InstrumentLabel>
            <h2 id="scope-comment-panel-title">Review the whole {{ scopeNoun }}</h2>
            <p class="decision-instrument__guidance">
              Use this sheet for feedback that applies across the {{ scopeNoun }} or has lost its
              passage anchor.
            </p>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            aria-label="Close scope feedback"
            @click="closeComposer"
          >
            ×
          </Button>
        </header>

        <div v-if="looseThreads.length" class="decision-instrument__loose-comments">
          <CommentThread
            v-for="thread in looseThreads"
            :key="thread.comment.id"
            :class="{ 'comment-thread--recorded': thread.comment.id === recordedThreadId }"
            :thread="thread"
            :busy="busy"
            :show-anchor-loss="thread.anchorState === 'orphaned' || thread.anchorState === 'missing'"
            @reply="forwardReply"
            @status="forwardStatus"
            @composer="(dirty) => forwardComposer(thread.comment.id, dirty)"
          />
        </div>
        <p v-else class="decision-instrument__empty">No scope-level or displaced feedback yet.</p>

        <form class="decision-instrument__composer" @submit.prevent="submitComposer">
          <label for="scope-comment">New whole-{{ scopeNoun }} note</label>
          <textarea
            id="scope-comment"
            v-model="comment"
            rows="4"
            :disabled="busy"
            :placeholder="`What should the agent reconsider across this ${scopeNoun}?`"
          />
          <Button type="submit" :disabled="busy">
            {{ scope.kind === 'session' ? 'Keep exploring with this feedback' : 'Send review to agent' }}
          </Button>
        </form>
      </section>
    </Transition>
  </Teleport>
</template>
