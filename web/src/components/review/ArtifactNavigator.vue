<script setup lang="ts">
import { computed } from 'vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import { scopeRelativeArtifactPath } from '@/lib/artifact-path'
import type { CommentCounts } from '@/lib/scope-review'

const props = withDefaults(
  defineProps<{
    paths: string[]
    selectedPath?: string
    threadCounts?: Record<string, CommentCounts>
    interactive?: boolean
  }>(),
  { selectedPath: undefined, threadCounts: () => ({}), interactive: true },
)

const emit = defineEmits<{ select: [path: string] }>()

const entries = computed(() =>
  props.paths.map((path) => {
    const localPath = scopeRelativeArtifactPath(path)
    const separator = localPath.lastIndexOf('/')
    return {
      path,
      directory: separator >= 0 ? localPath.slice(0, separator + 1) : '',
      filename: separator >= 0 ? localPath.slice(separator + 1) : localPath,
      counts: props.threadCounts[path] ?? { open: 0, addressed: 0, resolved: 0 },
    }
  }),
)

function threadLabel(counts: CommentCounts): string {
  const total = counts.open + counts.addressed + counts.resolved
  if (!total) return '0 threads'
  return `${counts.open} open · ${counts.addressed} addressed · ${counts.resolved} resolved`
}
</script>

<template>
  <section v-if="!interactive" class="artifact-navigator artifact-navigator--identity">
    <InstrumentLabel>Scratch coordinate</InstrumentLabel>
    <code v-if="paths[0]" class="artifact-navigator__scratch" :title="paths[0]">
      {{ scopeRelativeArtifactPath(paths[0]) }}
    </code>
  </section>

  <nav v-else class="artifact-navigator" aria-label="Artifacts">
    <InstrumentLabel>Artifact coordinates</InstrumentLabel>
    <ol>
      <li v-for="entry in entries" :key="entry.path">
        <button
          type="button"
          class="artifact-navigator__path"
          :class="{ 'artifact-navigator__path--selected': entry.path === selectedPath }"
          :aria-current="entry.path === selectedPath ? 'page' : undefined"
          :title="entry.path"
          @click="emit('select', entry.path)"
        >
          <span class="artifact-navigator__lock" aria-hidden="true">
            {{ entry.path === selectedPath ? '◆' : '◇' }}
          </span>
          <span class="artifact-navigator__coordinate">
            <span v-if="entry.directory" class="artifact-navigator__directory">
              {{ entry.directory }}
            </span>
            <strong>{{ entry.filename }}</strong>
            <code class="artifact-navigator__exact">{{ scopeRelativeArtifactPath(entry.path) }}</code>
          </span>
          <span class="artifact-navigator__threads">{{ threadLabel(entry.counts) }}</span>
        </button>
      </li>
    </ol>
  </nav>
</template>
