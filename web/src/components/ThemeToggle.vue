<script setup lang="ts">
import { Moon, Sun } from '@lucide/vue'
import { useColorMode } from '@vueuse/core'
import { Button } from '@/components/ui/button'

// Two explicit themes and no `auto`: the value in storage is what the pre-paint
// script in `index.html` reads, and that script can only act on a resolved
// theme. The system preference is the starting point, not a standing mode.
const preferred = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
const mode = useColorMode({
  storageKey: 'openspec-doc-theme',
  initialValue: preferred,
  disableTransition: true,
})

function toggle() {
  mode.value = mode.value === 'dark' ? 'light' : 'dark'
}
</script>

<template>
  <Button variant="outline" size="icon" :aria-label="`Switch to the ${mode === 'dark' ? 'light' : 'dark'} theme`" @click="toggle">
    <Sun v-if="mode === 'dark'" class="size-4" />
    <Moon v-else class="size-4" />
  </Button>
</template>
