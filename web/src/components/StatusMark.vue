<script setup lang="ts">
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'

type StatusKind = 'open' | 'addressed' | 'resolved' | 'verdict' | 'delivery' | 'reviewer' | 'agent'

const props = defineProps<{
  kind: StatusKind
  label: string
}>()

const glyph = computed(
  () =>
    ({
      open: '◇',
      addressed: '↗',
      resolved: '✓',
      verdict: '◆',
      delivery: '→',
      reviewer: '●',
      agent: '□',
    })[props.kind],
)
</script>

<template>
  <Badge :variant="props.kind" :class="`status-mark status-mark--${props.kind}`">
    <span class="status-mark__glyph" aria-hidden="true">{{ glyph }}</span>
    <span>{{ props.label }}</span>
  </Badge>
</template>
