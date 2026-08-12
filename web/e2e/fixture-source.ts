export const repeatedBlock = 'Repeated review target.'

/**
 * The obstruction assertions need a document taller than the 800px desktop viewport: a
 * decision control that covers prose only covers it once the reviewer has scrolled, and a
 * fixture that fits on one screen never reaches that state. The filler blocks below exist for
 * that reason and are the reason the final sentence is worth asserting on.
 */
const scrollFiller = Array.from(
  { length: 12 },
  (_unused, index) =>
    `Scroll pressure block ${index + 1}. The obstruction lane needs this document taller than one desktop viewport.`,
).join('\n\n')

/**
 * The widest ASCII diagram in this project's own design documents is 86 columns, and the bound on
 * the reading column exists to fit it. The row is sliced to exactly that width so the assertion
 * measures the column against a real worst case rather than a hopeful one.
 */
export const diagramColumns = 86

export const diagramSource = [
  '```',
  '  ├─ deterministic diagram row reaching the widest column this repository contains anywhere at all'.slice(
    0,
    diagramColumns,
  ),
  '  └─ narrower row',
  '```',
].join('\n')

export const proposalSource =
  '# Observatory Design System Fixture\n\nBrowser lane must see this proposal.\n\n' +
  `${repeatedBlock}\n\nA bridge between repeated blocks.\n\n${repeatedBlock}\n\n` +
  'Selection with **inline markup** crosses source.\n\n' +
  `${scrollFiller}\n\n` +
  'Final proposal sentence must stay fully legible at the document foot.\n'
