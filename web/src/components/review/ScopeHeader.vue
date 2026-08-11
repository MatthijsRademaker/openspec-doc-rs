<script setup lang="ts">
import DocumentHeading from '@/components/DocumentHeading.vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import StatusMark from '@/components/StatusMark.vue'
import { age } from '@/lib/age'
import type { ReviewReceipt } from '@/lib/motion-events'
import type { ScopeDetail } from '@/lib/scope-review'

defineProps<{ scope: ScopeDetail; receipt?: ReviewReceipt }>()
</script>

<template>
  <header class="scope-header">
    <div class="scope-header__identity">
      <DocumentHeading
        :level="1"
        :kicker="`${scope.kind} / review scope`"
        :title="scope.title ?? scope.key"
        compact
      />
      <code class="scope-header__key">{{ scope.key }}</code>
    </div>

    <div class="scope-header__state">
      <section
        :class="{
          'scope-header__standing--received':
            receipt?.kind === 'standing-verdict' || receipt?.kind === 'delivery',
        }"
        :data-motion-event="
          receipt?.kind === 'standing-verdict'
            ? 'receive'
            : receipt?.kind === 'delivery'
              ? 'resolve'
              : undefined
        "
        aria-labelledby="standing-verdict-title"
      >
        <InstrumentLabel>Standing verdict</InstrumentLabel>
        <h2 id="standing-verdict-title" class="sr-only">Standing verdict</h2>
        <template v-if="scope.standingVerdict">
          <StatusMark
            kind="verdict"
            :label="scope.standingVerdict.verdict"
            :event="receipt?.kind === 'standing-verdict' ? 'receive' : undefined"
          />
          <time :datetime="scope.standingVerdict.createdAt">
            Submitted {{ age(scope.standingVerdict.createdAt) }}
          </time>
          <StatusMark
            kind="delivery"
            :event="receipt?.kind === 'delivery' ? 'resolve' : undefined"
            :label="
              scope.standingVerdict.directiveDelivered
                ? 'Delivered to agent'
                : scope.standingVerdict.directivePending
                  ? 'Awaiting agent'
                  : 'Not yet delivered'
            "
          />
        </template>
        <p v-else>No verdict recorded</p>
      </section>

      <details class="scope-header__history">
        <summary>Verdict history / {{ scope.verdicts.length }}</summary>
        <ol v-if="scope.verdicts.length">
          <li v-for="record in [...scope.verdicts].reverse()" :key="record.id">
            <StatusMark kind="verdict" :label="record.verdict" />
            <time :datetime="record.createdAt">{{ age(record.createdAt) }}</time>
          </li>
        </ol>
        <p v-else>No decisions transmitted.</p>
      </details>
    </div>

  </header>
</template>
