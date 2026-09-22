import type { Block, SourceSpan } from '@/lib/scope-review'

export interface SourceSelection {
  /** Visible text shown in the composer. */
  displayText: string
  /** Markdown source text sent to the anchor writer. */
  selectedText: string
  /** Absolute UTF-8 byte offset at which the source search starts. */
  searchFrom: number
}

type Boundary = 'start' | 'end'

/**
 * Convert a browser selection over rendered markdown into source text.
 *
 * Browser ranges describe rendered text, while the anchor writer searches raw
 * markdown. Source spans bridge that representation boundary without making
 * the renderer expose markdown syntax to the reviewer.
 */
export function sourceSelection(
  content: HTMLElement,
  selection: Selection,
  block: Block,
): SourceSelection | undefined {
  if (selection.isCollapsed || selection.rangeCount === 0) return undefined

  const range = selection.getRangeAt(0)
  if (
    !containsBoundary(content, range.startContainer) ||
    !containsBoundary(content, range.endContainer)
  ) {
    return undefined
  }

  const selected = range.toString()
  const displayText = selected.trim()
  if (!displayText) return undefined

  const start = textOffset(content, range.startContainer, range.startOffset)
  const end = textOffset(content, range.endContainer, range.endOffset)
  if (start === undefined || end === undefined || end <= start) return undefined

  const leading = selected.length - selected.trimStart().length
  const trailing = selected.length - selected.trimEnd().length
  const visibleStart = start + leading
  const visibleEnd = end - trailing
  if (visibleEnd <= visibleStart) return undefined

  const spans = block.sourceSpans?.filter((span) => span.text.length > 0) ?? []
  if (spans.length === 0) return sourceSelectionWithoutSpans(displayText, block)

  const rendered = spans.map((span) => span.text).join('')
  const [mappedStart, mappedEnd] = renderedSelection(
    rendered,
    displayText,
    visibleStart,
    visibleEnd,
  )
  if (mappedStart === undefined || mappedEnd === undefined) return undefined

  const sourceStart = sourceOffsetAt(block, spans, mappedStart, 'start')
  const sourceEnd = sourceOffsetAt(block, spans, mappedEnd, 'end')
  if (sourceStart === undefined || sourceEnd === undefined || sourceEnd <= sourceStart) {
    return undefined
  }

  const sourceStartIndex = sourceIndexAtByte(block.source, sourceStart - block.range.start)
  const sourceEndIndex = sourceIndexAtByte(block.source, sourceEnd - block.range.start)
  if (sourceStartIndex === undefined || sourceEndIndex === undefined) return undefined

  const rawSource = block.source.slice(sourceStartIndex, sourceEndIndex)
  const selectedText = rawSource.trim()
  if (!selectedText) return undefined

  const trimmedStart = rawSource.length - rawSource.trimStart().length
  return {
    displayText,
    selectedText,
    searchFrom: sourceStart + utf8Length(rawSource.slice(0, trimmedStart)),
  }
}

function sourceSelectionWithoutSpans(
  displayText: string,
  block: Block,
): SourceSelection | undefined {
  const sourceIndex = block.source.indexOf(displayText)
  if (sourceIndex < 0) return undefined

  return {
    displayText,
    selectedText: displayText,
    searchFrom: block.range.start + utf8Length(block.source.slice(0, sourceIndex)),
  }
}

function containsBoundary(root: HTMLElement, node: Node): boolean {
  return node === root || root.contains(node)
}

function textOffset(root: HTMLElement, node: Node, offset: number): number | undefined {
  if (!containsBoundary(root, node)) return undefined

  const range = document.createRange()
  range.selectNodeContents(root)
  range.setEnd(node, offset)
  return range.toString().length
}

function renderedSelection(
  rendered: string,
  selected: string,
  hintStart: number,
  hintEnd: number,
): [number | undefined, number | undefined] {
  if (rendered.slice(hintStart, hintEnd) === selected) return [hintStart, hintEnd]

  let bestStart: number | undefined
  let bestDistance = Number.POSITIVE_INFINITY
  let searchFrom = 0
  while (searchFrom <= rendered.length) {
    const start = rendered.indexOf(selected, searchFrom)
    if (start < 0) break
    const distance = Math.abs(start - hintStart)
    if (distance < bestDistance) {
      bestStart = start
      bestDistance = distance
    }
    searchFrom = start + 1
  }

  return bestStart === undefined ? [undefined, undefined] : [bestStart, bestStart + selected.length]
}

function sourceOffsetAt(
  block: Block,
  spans: SourceSpan[],
  visibleOffset: number,
  boundary: Boundary,
): number | undefined {
  let cursor = 0

  for (const span of spans) {
    const next = cursor + span.text.length
    if (visibleOffset < next || (visibleOffset === next && boundary === 'end')) {
      return sourceOffsetInsideSpan(block, span, visibleOffset - cursor, boundary)
    }
    cursor = next
  }

  return visibleOffset === cursor ? spans[spans.length - 1]?.sourceEnd : undefined
}

function sourceOffsetInsideSpan(
  block: Block,
  span: SourceSpan,
  visibleOffset: number,
  boundary: Boundary,
): number | undefined {
  if (visibleOffset <= 0) return span.sourceStart
  if (visibleOffset >= span.text.length) return span.sourceEnd

  const sourceStartIndex = sourceIndexAtByte(block.source, span.sourceStart - block.range.start)
  const sourceEndIndex = sourceIndexAtByte(block.source, span.sourceEnd - block.range.start)
  if (sourceStartIndex === undefined || sourceEndIndex === undefined) return undefined

  const raw = block.source.slice(sourceStartIndex, sourceEndIndex)
  const renderedIndex = raw.indexOf(span.text)
  if (renderedIndex >= 0) {
    return span.sourceStart + utf8Length(raw.slice(0, renderedIndex + visibleOffset))
  }

  const prefix = span.text.slice(0, visibleOffset)
  const prefixIndex = raw.indexOf(prefix)
  if (prefixIndex >= 0)
    return span.sourceStart + utf8Length(raw.slice(0, prefixIndex + prefix.length))

  const suffix = span.text.slice(visibleOffset)
  const suffixIndex = raw.indexOf(suffix)
  if (suffixIndex >= 0) return span.sourceStart + utf8Length(raw.slice(0, suffixIndex))

  // A decoded entity or escaped character has no one-to-one source offset.
  // Keep its complete source segment so the anchor remains valid.
  return boundary === 'start' ? span.sourceStart : span.sourceEnd
}

function utf8Length(value: string): number {
  return new TextEncoder().encode(value).length
}

function sourceIndexAtByte(value: string, byteOffset: number): number | undefined {
  if (byteOffset < 0) return undefined
  if (byteOffset === 0) return 0

  let bytes = 0
  for (let index = 0; index < value.length; ) {
    const codePoint = value.codePointAt(index)
    if (codePoint === undefined) return undefined
    const character = String.fromCodePoint(codePoint)
    const nextBytes = bytes + utf8Length(character)
    if (nextBytes > byteOffset) return undefined
    index += character.length
    bytes = nextBytes
    if (bytes === byteOffset) return index
  }

  return bytes === byteOffset ? value.length : undefined
}
