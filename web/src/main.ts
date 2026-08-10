import { createApp } from 'vue'

import App from './App.vue'
import { router } from './router'
import './style.css'

async function enableMocking(): Promise<void> {
  if (!import.meta.env.DEV) return
  if (import.meta.env.MODE !== 'mock' && import.meta.env.VITE_API_MOCKS !== 'true') return

  const [{ worker }, { mountMockLaneControls }] = await Promise.all([
    import('./mocks/browser'),
    import('./mocks/controls'),
  ])
  await worker.start({ onUnhandledRequest: 'error' })
  mountMockLaneControls()
}

await enableMocking()
createApp(App).use(router).mount('#app')
