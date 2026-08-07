<script setup lang="ts">
import { Card } from '@/components/ui/card'

defineProps<{
  labelledBy: string
}>()
</script>

<template>
  <Card as="section" class="ruled-register" :aria-labelledby="labelledBy">
    <slot />
  </Card>
</template>
