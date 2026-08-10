import { describe, expect, it } from 'vitest'
import { scopeRelativeArtifactPath } from '@/lib/artifact-path'

describe('scopeRelativeArtifactPath', () => {
  it('renders scratch paths relative to the openspec-doc state root', () => {
    expect(
      scopeRelativeArtifactPath(
        '.openspec-doc/scratch/_session/0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74.md',
      ),
    ).toBe('0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74.md')
  })

  it('renders change artifacts relative to the selected change', () => {
    expect(
      scopeRelativeArtifactPath(
        'openspec/changes/observatory-design/specs/dashboard-visual-system/spec.md',
      ),
    ).toBe('specs/dashboard-visual-system/spec.md')
  })

  it('leaves unknown paths intact', () => {
    expect(scopeRelativeArtifactPath('notes/review.md')).toBe('notes/review.md')
  })
})
