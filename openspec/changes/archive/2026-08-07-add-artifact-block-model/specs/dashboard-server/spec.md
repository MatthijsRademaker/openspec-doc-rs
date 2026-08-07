## ADDED Requirements

### Requirement: An artifact is decomposed into anchorable blocks
The system SHALL decompose an artifact's markdown into blocks, each carrying a stable identifier, its rendered form, the exact source text it was sliced from, and that text's byte range in the artifact.

Blocks are paragraphs, headings, list items, table rows, and lines within a code fence. A list item is a block rather than the whole list, because a tasks file is one list and commenting on it as a single unit says nothing.

The source text is sliced from the artifact by the reported range and is never reconstructed from the parse events. Reconstruction produces text that differs from the file wherever the parser normalised anything, and that difference is invisible until an anchor's substring search fails in front of a reviewer.

#### Scenario: Every block's source occurs in its artifact
- **WHEN** an artifact is decomposed into blocks
- **THEN** each block's source text SHALL occur in that artifact's markdown

#### Scenario: A block reports where its source is
- **WHEN** an artifact is decomposed into blocks
- **THEN** each block SHALL report the byte range its source text occupies in the artifact

### Requirement: Markup in an artifact is not markup on the page
The system SHALL render raw HTML found in an artifact as text rather than emitting it as markup.

Artifacts are written by an agent and rendered into the reviewer's browser. Passing agent-written markup through to the page is a stored-injection path, and "it is only localhost" is the reasoning that keeps one alive until the day it matters.

#### Scenario: A script element in an artifact arrives as text
- **WHEN** an artifact contains a `<script>` element
- **THEN** the block containing it SHALL carry that element as escaped text and SHALL NOT carry it as markup
