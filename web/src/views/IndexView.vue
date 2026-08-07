<script setup lang="ts">
import { onMounted, ref } from 'vue'
import DocumentHeading from '@/components/DocumentHeading.vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
import OrbitalFrame from '@/components/OrbitalFrame.vue'
import ScopeRegister from '@/components/ScopeRegister.vue'
import { fetchIndex, type Index } from '@/lib/scopes'

const index = ref<Index>()
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
  <main class="observatory-shell">
    <header class="observatory-masthead">
      <div class="observatory-masthead__content">
        <DocumentHeading
          :level="1"
          display
          kicker="Index / review scopes"
          title="openspec-doc"
          description="Observe active explorations and proposed changes. Select exact scope coordinates to continue review."
        />
      </div>
      <OrbitalFrame image-src="/assets/images/index-orbit.webp" />
    </header>

    <section
      v-if="failure"
      class="index-state index-state--failure"
      role="alert"
      aria-labelledby="index-failure-title"
    >
      <InstrumentLabel>Index signal / failed</InstrumentLabel>
      <h2 id="index-failure-title" class="scope-register__title">Index unavailable</h2>
      <p class="index-state__message">The index could not be loaded: {{ failure }}</p>
    </section>

    <section v-else-if="!index" class="index-state" aria-live="polite" aria-busy="true">
      <InstrumentLabel>Index signal / observing</InstrumentLabel>
      <h2 class="scope-register__title">Observing available scopes…</h2>
    </section>

    <div v-else class="scope-registers">
      <ScopeRegister :scopes="index.sessions" prefix="sessions" title="Sessions" />
      <ScopeRegister :scopes="index.changes" prefix="changes" title="Changes" />
    </div>
  </main>
</template>
