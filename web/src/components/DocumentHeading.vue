<script setup lang="ts">
import { computed } from 'vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'

const props = withDefaults(
  defineProps<{
    kicker: string
    title: string
    description?: string
    id?: string
    level?: 1 | 2 | 3
    compact?: boolean
    display?: boolean
  }>(),
  {
    level: 2,
    description: undefined,
    compact: false,
    display: false,
  },
)

const headingTag = computed(() => `h${props.level}`)
</script>

<template>
  <div
    :class="[
      'document-heading',
      { 'document-heading--compact': props.compact, 'document-heading--display': props.display },
    ]"
  >
    <InstrumentLabel>{{ props.kicker }}</InstrumentLabel>
    <component :is="headingTag" :id="props.id" class="document-heading__title">
      {{ props.title }}
    </component>
    <p v-if="props.description" class="document-heading__description">{{ props.description }}</p>
  </div>
</template>
