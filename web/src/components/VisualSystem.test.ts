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
import { EVENT_ACQUIRE_MS } from '@/lib/event-channel'

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

function stylesheet(): string {
  return readFileSync(resolve(process.cwd(), 'src/style.css'), 'utf8')
}

/** The declarations of one top-level rule, so a rule can be asked what it does not say. */
function ruleBody(style: string, selector: string): string {
  const start = style.indexOf(`\n${selector} {`)
  if (start < 0) throw new Error(`style.css declares no ${selector} rule`)
  const open = style.indexOf('{', start)
  const close = style.indexOf('}', open)
  return style.slice(open + 1, close)
}

describe('observatory visual system', () => {
  it.each(kinds)('renders %s with visible text and a distinct non-color glyph', (kind, glyph) => {
    render(StatusMark, { props: { kind, label: kind } })

    expect(screen.getByText(kind)).toBeTruthy()
    expect(screen.getByText(glyph).getAttribute('aria-hidden')).toBe('true')
  })

  it('keeps orbital framing decorative and non-interactive', () => {
    const { container } = render(OrbitalFrame)

    const frame = container.firstElementChild
    expect(frame?.getAttribute('aria-hidden')).toBe('true')
    expect(frame?.querySelector('svg')).toBeTruthy()
    expect(frame?.querySelector('img, button, a, input')).toBeNull()
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
    // Suppression is global, so an effect added later needs no entry of its own here.
    expect(style).toContain('*:not([data-motion="required"])')
    expect(style).toContain('animation-duration: 0ms !important')
    expect(style).toContain('transition-duration: 0ms !important')
    expect(style).toContain('--motion-nudge: 0px')
    expect(style).toContain('--motion-duration-acquire: 0ms')
    expect(source).not.toMatch(/https?:\/\/(?:fonts\.|[^\s"']+\.(?:woff2?|ttf))/)
    expect(source).not.toContain('openspec-doc-theme')
    expect(source).not.toContain('.dark')
  })

  /* The channel gating a gesture removes its class when the dwell ends, which cuts the animation
     off wherever it happens to be. A dwell shorter than the duration therefore makes the CSS value
     a number the reviewer never sees, and nothing about that failure is visible in either file
     alone — the acquire gesture ran at 160ms of its declared 500ms until this was checked. */
  it('dwells the acquisition channel at least as long as every gesture it gates', () => {
    const style = readFileSync(resolve(process.cwd(), 'src/style.css'), 'utf8')
    const gated = ['--motion-duration-acquire', '--motion-duration-resolve'].map((token) => {
      const declared = new RegExp(`${token}:\\s*(\\d+)ms`).exec(style)
      if (!declared) throw new Error(`style.css declares no ${token} token`)
      return Number(declared[1])
    })

    expect(EVENT_ACQUIRE_MS).toBeGreaterThanOrEqual(Math.max(...gated))
  })

  it('places observatory artwork in the selected document and desktop instrument rails', () => {
    const source = (suffix: string) =>
      Object.entries(sourceModules).find(([path]) => path.endsWith(suffix))?.[1]
    const scopeView = source('/views/ScopeView.vue')
    const artifactDocument = source('/review/ArtifactDocument.vue')
    if (!scopeView || !artifactDocument) throw new Error('scope visual sources must be indexed')

    expect(artifactDocument).toContain('artifact-document__arrival-art')
    expect(artifactDocument).toContain('/assets/images/observatory-field.webp')
    expect(artifactDocument).toContain('alt=""')
    expect(artifactDocument).toContain('aria-hidden="true"')
    expect(scopeView).not.toContain('scope-observation-band')
    expect(scopeView).toContain('/assets/images/observatory-task-updated.webp')
    expect(scopeView).toContain('/assets/images/observatory-comment-updated.webp')
    expect(scopeView).toContain('scope-utility__art')
    expect(scopeView).toContain('scope-conversation__art')
    expect(`${scopeView}\n${artifactDocument}`).not.toMatch(
      /repository|activity feed|validation result|agent online/i,
    )
  })

  /* Two routes each stating their own width is what let the index stop at 90rem while the scope
     workbench ran edge to edge. The defect is the second copy, so what is checked is that neither
     route has one — not that the two happen to agree today. */
  it('states the shell width and padding once and lets both routes read them', () => {
    const style = stylesheet()

    expect(style).toContain('--shell-width: 100%')
    expect(style).toContain('--shell-padding: clamp(var(--space-half), 1.5vw, var(--space-2))')
    for (const selector of ['.observatory-shell', '.scope-workbench']) {
      const body = ruleBody(style, selector)
      expect(body, selector).toContain('width: var(--shell-width)')
      expect(body, selector).toContain('padding: var(--shell-padding)')
      expect(body, selector).not.toMatch(/margin:\s*0 auto/)
      expect(body, selector).not.toMatch(/width:\s*min\(/)
    }
    // A ceiling restated in a media query drifts exactly as readily as one restated in a rule.
    expect(style).not.toMatch(/\.observatory-shell\s*\{[^}]*padding:\s*var\(--space/)
  })

  /* The gesture used to terminate at `.scope-route-acquisition__key` — the `<code>` inside the
     loading placeholder, which the scope discards the moment it loads. It reported the arrival of
     something the reviewer never selected, and it looked like a working transition. */
  it('lands the scope coordinate on a real identity, never on the loading placeholder', () => {
    const style = stylesheet()

    expect(ruleBody(style, '.scope-header__identity .document-heading__title')).toContain(
      'view-transition-name: scope-coordinate',
    )
    expect(ruleBody(style, '.scope-entry--coordinate .scope-entry__link')).toContain(
      'view-transition-name: scope-coordinate',
    )
    expect(ruleBody(style, '.scope-route-acquisition__key')).not.toContain('view-transition-name')
    expect(style.match(/view-transition-name: scope-coordinate/g)).toHaveLength(2)
  })

  /* Artwork continuing behind panelling is one step from wallpaper. The declared fade and the
     opaque registers are what keep it on the right side of that line, and both are checkable. */
  it('keeps the index field decorative, faded, and under fully opaque panelling', () => {
    const style = stylesheet()
    const indexView = Object.entries(sourceModules).find(([path]) =>
      path.endsWith('/views/IndexView.vue'),
    )?.[1]
    if (!indexView) throw new Error('index view source must be indexed')

    expect(indexView).toContain('class="index-field" aria-hidden="true"')
    expect(indexView).toContain('/assets/images/observatory-field.webp')
    expect(indexView).not.toMatch(/index-field[\s\S]{0,400}?<(?:a|button|input)\b/)
    const field = ruleBody(style, '.index-field')
    expect(field).toContain('pointer-events: none')
    expect(field).toContain('mask-image: linear-gradient(')
    expect(field).toContain('rgb(0 0 0 / 0)')
    // Every panel over the field is a solid colour token, so no text is ever read against artwork.
    for (const selector of ['.ruled-register', '.index-observation__content']) {
      expect(ruleBody(style, selector), selector).toMatch(/background: var\(--(?:surface|canvas)\)/)
    }
    for (const token of ['--surface: #', '--canvas: #']) {
      expect(style).toContain(token)
    }
  })

  it('keeps observation artwork static and free of gaze plumbing', () => {
    const style = stylesheet()
    const source = (suffix: string) =>
      Object.entries(sourceModules).find(([path]) => path.endsWith(suffix))?.[1]
    const indexView = source('/views/IndexView.vue')
    const artifactDocument = source('/review/ArtifactDocument.vue')
    if (!indexView || !artifactDocument) throw new Error('observation sources must be indexed')

    const combined = `${style}\n${indexView}\n${artifactDocument}`
    expect(combined).not.toMatch(
      /ObservationGaze|trackPointerField|field-gaze|observation-gaze|eyelid/i,
    )
    expect(indexView).toContain('/assets/images/observatory-field.webp')
    expect(artifactDocument).toContain('/assets/images/observatory-field.webp')
    expect(ruleBody(style, '.index-field__image')).toContain('object-fit: cover')
    expect(ruleBody(style, '.artifact-document__arrival-image')).toContain('object-fit: cover')
  })

  it('keeps raw state colors out of component-local styles', () => {
    const rawColor = /(?:#[\da-f]{3,8}\b|oklch\()/i

    for (const [path, source] of Object.entries(sourceModules)) {
      expect(source, path).not.toMatch(rawColor)
    }
  })
})
