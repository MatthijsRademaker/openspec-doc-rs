<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import DocumentHeading from '@/components/DocumentHeading.vue'
import RuledRegister from '@/components/RuledRegister.vue'
import StatusMark from '@/components/StatusMark.vue'
import { Badge } from '@/components/ui/badge'
import { age } from '@/lib/age'
import { createLatestEventChannel, EVENT_ACQUIRE_MS } from '@/lib/event-channel'
import { returningScopeKey } from '@/lib/route-transition'
import type { Scope } from '@/lib/scopes'

const props = withDefaults(
  defineProps<{
    scopes: Scope[]
    prefix: 'sessions' | 'changes'
    title: string
    presentation?: 'primary' | 'secondary'
  }>(),
  { presentation: 'primary' },
)

const headingId = computed(() => `${props.prefix}-register-title`)
const scopeCount = computed(
  () => `${props.scopes.length} ${props.scopes.length === 1 ? 'scope' : 'scopes'}`,
)
const emptyLabel = computed(() => `No ${props.prefix} discovered.`)
// The gate outlasts the longest treatment it opens — the identity's halftone resolve, which runs
// `--motion-duration-resolve`. A shorter dwell would cut the print off mid-pass.
const acquisition = createLatestEventChannel<string>(EVENT_ACQUIRE_MS)

function isPrimaryActivation(event: MouseEvent): boolean {
  return (
    event.button === 0 &&
    !event.altKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.shiftKey &&
    !event.defaultPrevented
  )
}

/**
 * Reports the acquisition and nothing else. The Router owns the navigation and the transition
 * around it, because a click handler here cannot see the Back that returns through the same
 * gesture.
 */
function reportAcquisition(event: MouseEvent, key: string): void {
  if (isPrimaryActivation(event)) acquisition.signal(key)
}
</script>

<template>
  <RuledRegister
    :labelled-by="headingId"
    :class="`scope-register scope-register--${props.presentation}`"
  >
    <header class="scope-register__header">
      <DocumentHeading
        :id="headingId"
        :level="2"
        compact
        :kicker="`${props.prefix} / scope register`"
        :title="props.title"
      />
      <span class="instrument-label">{{ scopeCount }}</span>
    </header>

    <p v-if="props.scopes.length === 0" class="scope-register__empty">{{ emptyLabel }}</p>
    <ol v-else class="scope-register__list">
      <li v-for="scope in props.scopes" :key="scope.key" class="scope-register__item">
        <article
          class="scope-entry"
          :class="{
            'scope-entry--acquiring': acquisition.event.value === scope.key,
            'scope-entry--coordinate':
              acquisition.event.value === scope.key || returningScopeKey === scope.key,
          }"
          :data-motion-event="acquisition.event.value === scope.key ? 'acquire' : undefined"
        >
          <div class="scope-entry__identity">
            <div class="scope-entry__title-line">
              <!-- Exact identity owns Router navigation. Title remains mutable display text only. -->
              <RouterLink
                :class="[
                  'scope-entry__link',
                  { 'scope-entry__link--identifier': !scope.title },
                ]"
                :to="`/${props.prefix}/${scope.key}`"
                @click="reportAcquisition($event, scope.key)"
              >
                {{ scope.title ?? scope.key }}
              </RouterLink>
              <Badge v-if="scope.mostRecentlyActive" variant="instrument" class="scope-entry__recent">
                <span aria-hidden="true">◎</span>
                most recently active
              </Badge>
            </div>
            <code v-if="scope.title" class="scope-entry__key">{{ scope.key }}</code>
          </div>

          <dl class="scope-entry__metadata">
            <div class="scope-entry__datum">
              <dt class="scope-entry__term">Modified</dt>
              <dd class="scope-entry__value">
                {{ scope.modifiedAt ? age(scope.modifiedAt) : '—' }}
              </dd>
            </div>
            <div class="scope-entry__datum">
              <dt class="scope-entry__term">Open comments</dt>
              <dd class="scope-entry__value">
                <StatusMark
                  v-if="scope.openComments > 0"
                  kind="open"
                  :label="`${scope.openComments} open`"
                />
                <span v-else>0</span>
              </dd>
            </div>
            <div class="scope-entry__datum">
              <dt class="scope-entry__term">Verdict</dt>
              <dd class="scope-entry__value">
                <StatusMark v-if="scope.verdict" kind="verdict" :label="scope.verdict" />
                <span v-else>—</span>
              </dd>
            </div>
          </dl>
        </article>
      </li>
    </ol>
  </RuledRegister>
</template>
