import { readdir, readFile } from 'node:fs/promises'
import { dirname, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const webRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const distRoot = resolve(webRoot, 'dist')
const textPattern = /\.(?:css|html|js|json|map|mjs)$/i

/**
 * The mock lane is development-only: its handlers, fixtures and trigger reach the browser through
 * the `import.meta.env.DEV` gate in `main.ts`, and its Service Worker script through a
 * `apply: 'serve'` Vite plugin. Both exclusions are invisible when they break — a `public/` file or
 * a stray static import puts a mock handler in every installed binary and nothing else notices.
 */
const forbidden = [
  ['mockServiceWorker', 'MSW Service Worker script'],
  ['setupWorker', 'MSW browser worker'],
  ['artifact-rewrite', 'mock artifact-rewrite trigger'],
  ['add-dashboard-lifecycle', 'mock scope fixture'],
]

async function filesBelow(root) {
  const entries = await readdir(root, { withFileTypes: true })
  const files = []
  for (const entry of entries) {
    const path = resolve(root, entry.name)
    if (entry.isDirectory()) files.push(...(await filesBelow(path)))
    else if (entry.isFile()) files.push(path)
  }
  return files
}

const failures = []
for (const path of await filesBelow(distRoot)) {
  const local = relative(distRoot, path)
  for (const [needle, description] of forbidden) {
    if (local.includes(needle)) {
      failures.push(`${local} is the ${description}`)
      continue
    }
    if (!textPattern.test(local)) continue
    if ((await readFile(path, 'utf8')).includes(needle)) {
      failures.push(`${local} references the ${description} (${needle})`)
    }
  }
}

if (failures.length) {
  console.error(`mock lane present in production build:\n${failures.map((f) => `  ${f}`).join('\n')}`)
  process.exit(1)
}
console.log('mock lane absent from production build')
