import { fileURLToPath, URL } from 'node:url'

import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

import { resolveApiProxyTarget } from './src/lib/api-proxy-target'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
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
