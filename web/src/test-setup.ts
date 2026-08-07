import { cleanup } from '@testing-library/vue'
import { afterAll, afterEach, beforeAll } from 'vitest'
import { resetMockState } from './mocks/data'
import { server } from './mocks/node'

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))

afterAll(() => server.close())

afterEach(() => {
  server.resetHandlers()
  resetMockState()
  cleanup()
  window.localStorage.clear()
  document.documentElement.className = ''
})
