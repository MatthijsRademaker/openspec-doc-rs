import { cleanup } from '@testing-library/vue'
import { afterEach } from 'vitest'

// jsdom has no matchMedia, and ThemeToggle reads the preferred scheme through
// it at setup. Light-by-default keeps every assertion about the initial theme
// deterministic.
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  }),
})

afterEach(() => {
  cleanup()
  // useColorMode writes the resolved theme onto <html> and into localStorage;
  // both would leak into the next test.
  window.localStorage.clear()
  document.documentElement.className = ''
})
