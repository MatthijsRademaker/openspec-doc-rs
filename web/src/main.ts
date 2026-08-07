import { createApp } from 'vue'

import App from './App.vue'
import { router } from './router'
import './style.css'

async function enableMocking(): Promise<void> {
  if (!import.meta.env.DEV) return
  if (import.meta.env.MODE !== 'mock' && import.meta.env.VITE_API_MOCKS !== 'true') return

  const { worker } = await import('./mocks/browser')
  await worker.start({ onUnhandledRequest: 'error' })
}

await enableMocking()
createApp(App).use(router).mount('#app')
