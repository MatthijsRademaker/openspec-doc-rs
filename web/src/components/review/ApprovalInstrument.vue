<script setup lang="ts">
import { computed } from 'vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import { Button } from '@/components/ui/button'
import type { ApprovalAct, ApprovalState, CommentCounts } from '@/lib/scope-review'

const props = withDefaults(
  defineProps<{
    approval: ApprovalState
    counts: CommentCounts
    busy?: boolean
    /** A refusal or a partial outcome, rendered here rather than in the page's
     *  generic alert: "action not recorded" is the wrong sentence for a bulk
     *  submission whose resolutions landed. */
    failure?: string
  }>(),
  { busy: false, failure: undefined },
)

const emit = defineEmits<{ approval: [act: ApprovalAct] }>()

const outstanding = computed(() => props.counts.open + props.counts.addressed)
/** The two approve controls are mutually exclusive. Showing both would ask the
 *  reviewer to choose between a button that works and one that is refused, on a
 *  page that already knows which is which. */
const sweeps = computed(() => outstanding.value > 0)
const approved = computed(() => props.approval.state === 'approved')

const stateLabel = computed(
  () =>
    ({ approved: 'Approved', stale: 'Stale', 'not-approved': 'Not approved' })[
      props.approval.state
    ],
)

/** What the sweep will do, before it is submitted. This is the one control in
 *  the interface that discards the reviewer's own outstanding feedback. */
const sweepLabel = computed(() => {
  const parts = []
  if (props.counts.open) parts.push(`${props.counts.open} open`)
  if (props.counts.addressed) parts.push(`${props.counts.addressed} addressed`)
  return `Resolve ${outstanding.value} and approve (${parts.join(', ')})`
})
</script>

<template>
  <section
    class="approval-instrument"
    :class="`approval-instrument--${approval.state}`"
    aria-label="Approval state"
  >
    <InstrumentLabel>Approval / {{ stateLabel.toLowerCase() }}</InstrumentLabel>
    <p class="approval-instrument__state">
      <span class="approval-instrument__mark" aria-hidden="true">{{
        approved ? '✓' : approval.state === 'stale' ? '↺' : '◇'
      }}</span>
      <strong>{{ stateLabel }}</strong>
      <span class="approval-instrument__reason">{{ approval.reason }}</span>
    </p>

    <p v-if="approval.changedArtifacts.length" class="approval-instrument__changed">
      Changed since approval: {{ approval.changedArtifacts.join(', ') }}
    </p>

    <p v-if="failure" class="approval-instrument__failure" role="alert">{{ failure }}</p>

    <div class="approval-instrument__controls">
      <Button
        v-if="sweeps"
        type="button"
        size="sm"
        :disabled="busy"
        :aria-label="sweepLabel"
        @click="emit('approval', 'resolve-all-and-approve')"
      >
        <span aria-hidden="true">✓</span>
        <span class="approval-instrument__control-label">{{ sweepLabel }}</span>
      </Button>
      <Button
        v-else-if="!approved"
        type="button"
        size="sm"
        :disabled="busy"
        aria-label="Approve this change"
        @click="emit('approval', 'approve')"
      >
        <span aria-hidden="true">✓</span>
        <span class="approval-instrument__control-label">Approve change</span>
      </Button>
      <Button
        v-if="approved"
        type="button"
        variant="ghost"
        size="sm"
        :disabled="busy"
        aria-label="Withdraw approval"
        @click="emit('approval', 'withdraw')"
      >
        <span aria-hidden="true">↩</span>
        <span class="approval-instrument__control-label">Withdraw approval</span>
      </Button>
    </div>
  </section>
</template>
