<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = withDefaults(
  defineProps<{
    as?: 'div' | 'section' | 'article'
    class?: HTMLAttributes['class']
    size?: 'default' | 'sm'
  }>(),
  {
    as: 'div',
    size: 'default',
  },
)
</script>

<template>
  <component
    :is="props.as"
    data-slot="card"
    :data-size="size"
    :class="cn('bg-card text-card-foreground overflow-hidden rounded-[var(--radius)] border border-border py-0 text-sm shadow-none group/card flex flex-col', props.class)"
  >
    <slot />
  </component>
</template>
