<script setup lang="ts">
import { onMounted, ref } from 'vue'
import DocumentHeading from '@/components/DocumentHeading.vue'
import InstrumentLabel from '@/components/InstrumentLabel.vue'
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
  <main class="observatory-shell observatory-index">
    <header class="index-observation">
      <div class="index-observation__content">
        <p class="index-product">openspec-doc <span aria-hidden="true">/</span> review index</p>
        <DocumentHeading
          id="changes-title"
          :level="1"
          kicker="Index / observation field"
          title="Changes"
          description="Select exact scope coordinates."
        />
      </div>
      <div class="index-observation__art" aria-hidden="true">
        <img
          class="index-observation__image"
          src="/assets/images/observatory-field.webp"
          alt=""
          aria-hidden="true"
        />
      </div>
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

    <template v-else>
      <div class="index-workbench" aria-labelledby="changes-title">
        <ScopeRegister
          :scopes="index.changes"
          prefix="changes"
          title="Change register"
          presentation="primary"
        />
        <ScopeRegister
          :scopes="index.sessions"
          prefix="sessions"
          title="Sessions"
          presentation="secondary"
        />
      </div>

      <aside class="index-plates" aria-label="Observation posture">
        <div class="index-plate index-plate--sun" aria-hidden="true">
          <img
            class="index-plate__image"
            src="/assets/images/observatory-plate-sun.webp"
            alt=""
            aria-hidden="true"
          />
        </div>
        <div class="index-plate index-plate--face" aria-hidden="true">
          <img
            class="index-plate__image"
            src="/assets/images/observatory-plate-face.webp"
            alt=""
            aria-hidden="true"
          />
        </div>
        <div class="index-plate index-plate--star-system" aria-hidden="true">
          <img
            class="index-plate__image"
            src="/assets/images/observatory-plate-star-system.webp"
            alt=""
            aria-hidden="true"
          />
        </div>
        <div class="index-plates__posture">
          <p>Observe / plan / execute / verify</p>
          <p>The agent follows the spec. You guide the direction.</p>
        </div>
      </aside>
    </template>
  </main>
</template>
