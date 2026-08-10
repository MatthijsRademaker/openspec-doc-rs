import { readdir, stat } from 'node:fs/promises'
import { dirname, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const webRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const publicRoot = resolve(webRoot, 'public')
const distRoot = resolve(webRoot, 'dist')
const rasterPattern = /\.(?:avif|gif|jpe?g|png|webp)$/i
const sourceReferences = new Set([
  'abstract-face.png',
  'abstract-star-system.png',
  'abstract-sun.png',
  'dashboard-mockup.png',
  'design-system.png',
  'main-panel-background.png',
])
const allowlist = new Set([
  'assets/images/observatory-field.webp',
  'assets/images/observatory-comment-updated.webp',
  'assets/images/observatory-plate-face.webp',
  'assets/images/observatory-plate-star-system.webp',
  'assets/images/observatory-plate-sun.webp',
  'assets/images/observatory-task-updated.webp',
])
const supersededRasters = new Set([
  'assets/images/index-observation-field.webp',
  'assets/images/index-orbit.webp',
  'assets/images/index-plate-face.webp',
  'assets/images/index-plate-star-system.webp',
  'assets/images/index-plate-sun.webp',
  'assets/images/scope-observation-field.webp',
  'assets/images/scope-plate-face.webp',
  'assets/images/scope-plate-star-system.webp',
  'assets/images/scope-plate-sun.webp',
])
const maxRuntimeRasterBytes = 6 * 1024 * 1024

async function filesBelow(root) {
  const entries = await readdir(root, { withFileTypes: true })
  const files = []
  for (const entry of entries) {
    const path = resolve(root, entry.name)
    if (entry.isDirectory()) {
      files.push(...(await filesBelow(path)))
    } else if (entry.isFile()) {
      files.push(path)
    }
  }
  return files
}

function normalizedRelative(root, path) {
  return relative(root, path).split('\\').join('/')
}

async function runtimeRasters(root) {
  return (await filesBelow(root))
    .filter((path) => rasterPattern.test(path))
    .map((path) => normalizedRelative(root, path))
    .sort()
}

const publicRasters = await runtimeRasters(publicRoot)
const distRasters = await runtimeRasters(distRoot)
const failures = []

for (const expected of allowlist) {
  if (!publicRasters.includes(expected)) {
    failures.push(`allowlisted runtime image is missing from public/: ${expected}`)
  }
  if (!distRasters.includes(expected)) {
    failures.push(`allowlisted runtime image is missing from dist/: ${expected}`)
  }
}
for (const path of publicRasters) {
  if (supersededRasters.has(path)) {
    failures.push(`superseded runtime image remains in public/: ${path}`)
  } else if (!allowlist.has(path)) {
    failures.push(`unlisted raster in public/: ${path}`)
  }
}
for (const path of distRasters) {
  if (supersededRasters.has(path)) {
    failures.push(`superseded runtime image remains in dist/: ${path}`)
  } else if (!allowlist.has(path)) {
    failures.push(`unlisted raster in dist/: ${path}`)
  }
  if (sourceReferences.has(path.split('/').at(-1))) {
    failures.push(`design source reached dist/: ${path}`)
  }
}

let totalBytes = 0
for (const path of distRasters) {
  totalBytes += (await stat(resolve(distRoot, path))).size
}
if (totalBytes > maxRuntimeRasterBytes) {
  failures.push(
    `runtime raster payload ${totalBytes} bytes exceeds ${maxRuntimeRasterBytes}-byte budget`,
  )
}

if (failures.length > 0) {
  for (const failure of failures) {
    process.stderr.write(`runtime image check failed: ${failure}\n`)
  }
  process.exitCode = 1
} else {
  process.stdout.write(
    `runtime images allowed: ${distRasters.length} file, ${totalBytes} / ${maxRuntimeRasterBytes} bytes\n`,
  )
}
