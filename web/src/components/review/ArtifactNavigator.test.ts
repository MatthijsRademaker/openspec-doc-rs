import { fireEvent, render, screen } from '@testing-library/vue'
import { describe, expect, it } from 'vitest'
import ArtifactNavigator from '@/components/review/ArtifactNavigator.vue'

const root = 'openspec/changes/change-with-an-extremely-long-identifier'
const paths = [
  `${root}/proposal.md`,
  `${root}/design.md`,
  `${root}/tasks.md`,
  `${root}/specs/dashboard-html-views/spec.md`,
  `${root}/specs/dashboard-visual-system/spec.md`,
]

describe('ArtifactNavigator', () => {
  it('emits exact paths and marks selection with text plus non-color lock', async () => {
    const { emitted } = render(ArtifactNavigator, {
      props: {
        paths,
        selectedPath: paths[3],
        threadCounts: {
          [paths[3] ?? '']: { open: 1, addressed: 1, resolved: 1 },
        },
      },
    })

    const selected = document.querySelector('.artifact-navigator__path[aria-current="page"]')
    expect(selected?.textContent).toContain('◆')
    expect(selected?.textContent).toContain('specs/dashboard-html-views/spec.md')
    expect(selected?.textContent).toContain('1 open · 1 addressed · 1 resolved')
    expect(document.querySelectorAll('.artifact-navigator__exact')).toHaveLength(paths.length)

    await fireEvent.click(screen.getByRole('button', { name: /design\.md/ }))
    expect(emitted().select?.[0]).toEqual([paths[1]])
  })

  it('renders sole scratch identity without navigation or inactive button', () => {
    const scratch = '.openspec-doc/scratch/_session/session-with-long-identity.md'
    render(ArtifactNavigator, { props: { paths: [scratch], interactive: false } })

    expect(screen.getByText(scratch)).toBeTruthy()
    expect(screen.queryByRole('navigation')).toBeNull()
    expect(screen.queryByRole('button')).toBeNull()
  })
})
