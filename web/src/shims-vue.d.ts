/**
 * A plain TypeScript server cannot resolve `.vue` imports and reports every
 * single-file component as a missing module. `vue-tsc` — the typecheck gate
 * that actually runs here — resolves SFCs natively through Volar and takes
 * precedence over this wildcard, so the declaration only serves tools without
 * Vue support; it never weakens the gate.
 */
declare module '*.vue' {
  import type { DefineComponent } from 'vue'

  const component: DefineComponent<Record<string, never>, Record<string, never>, unknown>
  export default component
}
