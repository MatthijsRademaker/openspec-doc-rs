<script setup lang="ts">
import { computed } from 'vue'
import DocumentHeading from '@/components/DocumentHeading.vue'
import RuledRegister from '@/components/RuledRegister.vue'
import StatusMark from '@/components/StatusMark.vue'
import { Badge } from '@/components/ui/badge'
import { age } from '@/lib/age'
import type { Scope } from '@/lib/scopes'

const props = defineProps<{
  scopes: Scope[]
  prefix: 'sessions' | 'changes'
  title: string
}>()

const headingId = computed(() => `${props.prefix}-register-title`)
const scopeCount = computed(
  () => `${props.scopes.length} ${props.scopes.length === 1 ? 'scope' : 'scopes'}`,
)
const emptyLabel = computed(() => `No ${props.prefix} discovered.`)
</script>

<template>
  <RuledRegister :labelled-by="headingId">
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
        <article class="scope-entry">
          <div class="scope-entry__identity">
            <div class="scope-entry__title-line">
              <!-- Identity owns navigation. Title is mutable display text only. -->
              <a class="scope-entry__link" :href="`/${props.prefix}/${scope.key}`">
                {{ scope.title ?? scope.key }}
              </a>
              <Badge v-if="scope.mostRecentlyActive" variant="instrument" class="scope-entry__recent">
                <span aria-hidden="true">◎</span>
                most recently active
              </Badge>
            </div>
            <code class="scope-entry__key">{{ scope.key }}</code>
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
