import { createReadStream } from 'node:fs'
import { fileURLToPath, URL } from 'node:url'

import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig, type Plugin } from 'vitest/config'

import { resolveApiProxyTarget } from './src/lib/api-proxy-target'

/**
 * The MSW worker script has to be reachable at the origin root to claim a root scope, but a file
 * in `public/` is copied verbatim into `dist/` and from there embedded into the binary. Serving it
 * from `src/mocks/` through a development-only middleware keeps the whole mock lane behind the
 * mode gate instead of shipping a mock handler to every installation.
 */
const mockServiceWorker: Plugin = {
  name: 'openspec-doc:mock-service-worker',
  apply: 'serve',
  configureServer(server) {
    const script = fileURLToPath(new URL('./src/mocks/mockServiceWorker.js', import.meta.url))
    server.middlewares.use('/mockServiceWorker.js', (_request, response) => {
      response.setHeader('content-type', 'text/javascript')
      response.setHeader('service-worker-allowed', '/')
      createReadStream(script).pipe(response)
    })
  },
}

export default defineConfig({
  plugins: [
    // Root URLs address public assets in the embedded filesystem; importing them turns `/assets`
    // into `file:///assets` under Windows test runners.
    vue({ template: { transformAssetUrls: { includeAbsolute: false } } }),
    tailwindcss(),
    mockServiceWorker,
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    proxy: {
      // Same-origin in development, same as production: the browser never sees
      // the Rust server's address, so no CORS configuration exists to drift.
      '/api': { target: resolveApiProxyTarget(process.env), changeOrigin: true },
    },
  },
  test: {
    // jsdom over happy-dom: the component tests assert on real DOM semantics
    // (roles, accessible names), and jsdom's are the reference implementation.
    environment: 'jsdom',
    setupFiles: ['src/test-setup.ts'],
    include: ['src/**/*.test.ts'],
  },
})
