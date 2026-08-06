## MODIFIED Requirements

### Requirement: Anchor context creation
The system SHALL create an anchor context from selected text and its containing markdown, capturing the heading path, before/after text context, and character offsets, and SHALL fail loudly when the selected text is not found in the markdown. A caller that knows where in the markdown the selection came from SHALL be able to say so, and the system SHALL search from that point rather than from the start.

Anchor creation records the first occurrence of the selected text and derives the heading path from it. Resolution then confirms that offset rather than questioning it — `exact` resolution succeeds against the wrong text just as readily as against the right text — so an anchor created at the wrong occurrence is wrong permanently, under the wrong heading, with no error anywhere. When the selection is a whole markdown block, the selected text is exactly the block's source, and two identical list items in one document are ordinary. The caller holds the offset; without a way to pass it, it is discarded at the API boundary.

#### Scenario: Anchor context is created for text present in the markdown
- **WHEN** selected text exists in the given markdown
- **THEN** the system SHALL produce an anchor context including heading path, before/after text, and start/end offsets

#### Scenario: Selected text not found fails loudly
- **WHEN** selected text does not exist in the given markdown
- **THEN** the system SHALL return an error rather than producing a partial or guessed anchor context

#### Scenario: A repeated selection anchors to the occurrence the caller names
- **WHEN** the selected text occurs more than once and the caller supplies the offset it came from
- **THEN** the system SHALL anchor to the occurrence at or after that offset, and SHALL derive the heading path from it

#### Scenario: A caller that does not know where the text came from anchors to the first occurrence
- **WHEN** the caller supplies no starting offset
- **THEN** the system SHALL anchor to the first occurrence of the selected text
