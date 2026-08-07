/// <reference types="node" />

import { render, screen } from '@testing-library/vue'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import indexHtml from '../../index.html?raw'
import DocumentHeading from '@/components/DocumentHeading.vue'
import OrbitalFrame from '@/components/OrbitalFrame.vue'
import StatusMark from '@/components/StatusMark.vue'
import { Button } from '@/components/ui/button'

const sourceModules = import.meta.glob('../**/*.{vue,ts}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>

const kinds = [
  ['open', '◇'],
  ['addressed', '↗'],
  ['resolved', '✓'],
  ['verdict', '◆'],
  ['delivery', '→'],
  ['reviewer', '●'],
  ['agent', '□'],
] as const

describe('observatory visual system', () => {
  it.each(kinds)('renders %s with visible text and a distinct non-color glyph', (kind, glyph) => {
    render(StatusMark, { props: { kind, label: kind } })

    expect(screen.getByText(kind)).toBeTruthy()
    expect(screen.getByText(glyph).getAttribute('aria-hidden')).toBe('true')
  })

  it('keeps orbital framing decorative and non-interactive', () => {
    const { container } = render(OrbitalFrame, {
      props: { imageSrc: '/assets/images/index-orbit.webp' },
    })

    const frame = container.firstElementChild
    expect(frame?.getAttribute('aria-hidden')).toBe('true')
    expect(frame?.querySelector('img')?.getAttribute('alt')).toBe('')
    expect(frame?.querySelector('button, a, input')).toBeNull()
  })

  it('separates display heading and instrument metadata roles', () => {
    render(DocumentHeading, {
      props: { level: 1, kicker: 'Scope / instrument label', title: 'Document title' },
    })

    expect(screen.getByRole('heading', { name: 'Document title' }).classList).toContain(
      'document-heading__title',
    )
    expect(screen.getByText('Scope / instrument label').classList).toContain('instrument-label')
  })

  it('retains native keyboard focus on adapted controls', () => {
    render(Button, { slots: { default: 'Execute' }, props: { variant: 'instrument' } })

    const button = screen.getByRole('button', { name: 'Execute' })
    button.focus()
    expect(document.activeElement).toBe(button)
    expect(button.className).toContain('focus-visible:ring-2')
  })

  it('centralizes roles, state colors, bundled fonts, and reduced motion', () => {
    const style = readFileSync(resolve(process.cwd(), 'src/style.css'), 'utf8')
    const source = `${style}\n${indexHtml}`

    for (const token of [
      '--canvas:',
      '--surface:',
      '--raised:',
      '--border-ink:',
      '--bone:',
      '--muted-ink:',
      '--accent-ink:',
      '--focus-ink:',
      '--open:',
      '--addressed:',
      '--resolved:',
      '--verdict:',
      '--delivery:',
      '--reviewer:',
      '--agent:',
    ]) {
      expect(style).toContain(token)
    }
    expect(style).toContain('@import "@fontsource/cormorant-garamond/latin-400.css"')
    expect(style).toContain('--font-display: "Cormorant Garamond"')
    expect(style).toContain('--font-sans: "IBM Plex Sans Variable"')
    expect(style).toContain('--font-mono: "IBM Plex Mono"')
    expect(style).toContain('@media (prefers-reduced-motion: reduce)')
    expect(style).toContain('--motion-duration: 0ms')
    expect(style).toContain('transition: none')
    expect(source).not.toMatch(/https?:\/\/(?:fonts\.|[^\s"']+\.(?:woff2?|ttf))/)
    expect(source).not.toContain('openspec-doc-theme')
    expect(source).not.toContain('.dark')
  })

  it('keeps raw state colors out of component-local styles', () => {
    const rawColor = /(?:#[\da-f]{3,8}\b|oklch\()/i

    for (const [path, source] of Object.entries(sourceModules)) {
      expect(source, path).not.toMatch(rawColor)
    }
  })
})
