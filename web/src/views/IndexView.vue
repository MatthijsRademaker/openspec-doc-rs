<script setup lang="ts">
import { onMounted, ref } from 'vue'
import ScopeTable from '@/components/ScopeTable.vue'
import ThemeToggle from '@/components/ThemeToggle.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { fetchIndex, type Index } from '@/lib/scopes'

const index = ref<Index>()
// A failed load says so. An empty index and an unreachable server are different
// facts about the project and must not render the same.
const failure = ref<string>()

onMounted(async () => {
  try {
    index.value = await fetchIndex()
  } catch (error) {
    failure.value = error instanceof Error ? error.message : String(error)
  }
})
</script>

<template>
  <main class="mx-auto flex max-w-4xl flex-col gap-6 p-8">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold tracking-tight">openspec-doc</h1>
      <ThemeToggle />
    </header>

    <p v-if="failure" class="text-destructive">
      The index could not be loaded: {{ failure }}
    </p>
    <p v-else-if="!index" class="text-muted-foreground">Loading…</p>
    <template v-else>
      <Card>
        <CardHeader>
          <CardTitle>Sessions</CardTitle>
        </CardHeader>
        <CardContent>
          <ScopeTable :scopes="index.sessions" prefix="sessions" />
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Changes</CardTitle>
        </CardHeader>
        <CardContent>
          <ScopeTable :scopes="index.changes" prefix="changes" />
        </CardContent>
      </Card>
    </template>
  </main>
</template>
