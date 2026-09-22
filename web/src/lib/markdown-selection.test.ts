import { afterEach, describe, expect, it } from 'vitest'
import { sourceSelection } from '@/lib/markdown-selection'
import type { Block, SourceSpan } from '@/lib/scope-review'

function span(source: string, text: string): SourceSpan {
  const sourceStart = source.indexOf(text)
  if (sourceStart < 0) throw new Error(`missing source text ${text}`)
  const prefix = source.slice(0, sourceStart)
  return {
    text,
    sourceStart: new TextEncoder().encode(prefix).length,
    sourceEnd: new TextEncoder().encode(prefix + text).length,
  }
}

afterEach(() => {
  document.body.replaceChildren()
  window.getSelection()?.removeAllRanges()
})

describe('sourceSelection', () => {
  it('maps a rendered selection across inline markdown syntax to raw source', () => {
    const source = 'The only way is `cargo install` from **clone**.\n'
    const content = document.createElement('div')
    content.innerHTML =
      '<p>The only way is <code>cargo install</code> from <strong>clone</strong>.</p>'
    document.body.append(content)

    const block: Block = {
      id: 'paragraph',
      html: content.innerHTML,
      source: source.slice(0, -1),
      sourceSpans: [
        span(source, 'The only way is '),
        span(source, 'cargo install'),
        span(source, ' from '),
        span(source, 'clone'),
        span(source, '.'),
      ],
      range: { start: 0, end: new TextEncoder().encode(source).length - 1 },
    }
    const paragraph = content.querySelector('p')
    if (!paragraph) throw new Error('paragraph fixture missing')
    const range = document.createRange()
    range.selectNodeContents(paragraph)
    const selection = window.getSelection()
    if (!selection) throw new Error('selection unavailable')
    selection.removeAllRanges()
    selection.addRange(range)

    expect(sourceSelection(content, selection, block)).toEqual({
      displayText: 'The only way is cargo install from clone.',
      selectedText: 'The only way is `cargo install` from **clone**.',
      searchFrom: 0,
    })
  })

  it('uses UTF-8 byte offsets when selection follows non-ASCII source text', () => {
    const source = 'Before — `code` after.'
    const content = document.createElement('div')
    content.innerHTML = '<p>Before — <code>code</code> after.</p>'
    document.body.append(content)

    const codeStart = new TextEncoder().encode('Before — ').length
    const codeEnd = new TextEncoder().encode('Before — `code`').length
    const block: Block = {
      id: 'unicode',
      html: content.innerHTML,
      source,
      sourceSpans: [
        {
          text: 'Before — ',
          sourceStart: 0,
          sourceEnd: codeStart,
        },
        { text: 'code', sourceStart: codeStart, sourceEnd: codeEnd },
        {
          text: ' after.',
          sourceStart: codeEnd,
          sourceEnd: new TextEncoder().encode(source).length,
        },
      ],
      range: { start: 0, end: new TextEncoder().encode(source).length },
    }
    const text = content.querySelector('p')?.firstChild
    if (!text) throw new Error('text fixture missing')
    const range = document.createRange()
    range.setStart(text, 'Before — '.length)
    range.setEnd(content.querySelector('code')?.firstChild ?? text, 4)
    const selection = window.getSelection()
    if (!selection) throw new Error('selection unavailable')
    selection.removeAllRanges()
    selection.addRange(range)

    expect(sourceSelection(content, selection, block)).toEqual({
      displayText: 'code',
      selectedText: '`code`',
      searchFrom: codeStart,
    })
  })
})
