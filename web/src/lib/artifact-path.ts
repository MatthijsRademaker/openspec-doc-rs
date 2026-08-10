const SESSION_SCRATCH_ROOT = '.openspec-doc/scratch/_session/'
const SCRATCH_ROOT = '.openspec-doc/scratch/'
const CHANGE_MARKER = '/changes/'

export function scopeRelativeArtifactPath(path: string): string {
  if (path.startsWith(SESSION_SCRATCH_ROOT)) return path.slice(SESSION_SCRATCH_ROOT.length)
  if (path.startsWith(SCRATCH_ROOT)) return path.slice(SCRATCH_ROOT.length)

  const markerIndex = path.indexOf(CHANGE_MARKER)
  if (markerIndex < 0) return path

  const changePath = path.slice(markerIndex + CHANGE_MARKER.length)
  const separator = changePath.indexOf('/')
  return separator < 0 ? changePath : changePath.slice(separator + 1)
}
