/**
 * How long ago a timestamp was, at the granularity a reviewer is choosing on.
 * Relative because "20m ago" is the actual question being asked of an mtime, and
 * because the page then needs no timezone handling.
 */
export function age(at: string): string {
  const elapsed = Date.now() - Date.parse(at)

  // A file dated in the future is a clock that disagrees with itself, not
  // something worth rendering an error for.
  if (elapsed < 0) {
    return 'just now'
  }

  const seconds = Math.floor(elapsed / 1000)
  if (seconds < 60) {
    return 'just now'
  }
  if (seconds < 3600) {
    return `${Math.floor(seconds / 60)}m ago`
  }
  if (seconds < 86400) {
    return `${Math.floor(seconds / 3600)}h ago`
  }

  return `${Math.floor(seconds / 86400)}d ago`
}
