## ADDED Requirements

### Requirement: A session's title is read from its scratch note
The system SHALL derive a session's human-readable title from its scratch note's first heading when that heading is level one, and SHALL report no title otherwise.

The agent already writes a heading describing what it is exploring, and keeps it current as the exploration changes shape. Deriving the title from it costs no new state, no new command, and no new thing for the agent to remember. Reporting nothing when there is no such heading is deliberate: a session with nothing written has no topic, and a synthesised one is a guess the reviewer would have to learn to distrust.

The note's *first* heading, rather than the first level-one heading anywhere in it: a note that opens at level two was never given a title, and titling it after some `#` further down names it after one of its sections.

#### Scenario: A note with a heading yields a title
- **WHEN** a session's scratch note begins with a level-one heading
- **THEN** the system SHALL report that heading's text, trimmed, as the session's title

#### Scenario: A note that does not open at level one yields no title
- **WHEN** a session's scratch note is absent, is empty, contains no heading, or opens with a heading below level one
- **THEN** the system SHALL report that the session has no title, rather than reporting a placeholder or an error

#### Scenario: A promoted note titles the change it became
- **WHEN** a change's note, promoted to `.openspec-doc/scratch/<name>.md`, opens with a level-one heading
- **THEN** the system SHALL report that heading's text as the change's title, by the same rule it titles a session

#### Scenario: A heading inside a code fence is not a title
- **WHEN** a session's scratch note contains a line beginning with `#` inside a fenced code block, before any real heading
- **THEN** the system SHALL NOT report that line as the session's title

### Requirement: The session identifier remains the session's identity
The system SHALL continue to identify a session by its session id in every URL, filename, and hook payload, and SHALL treat the title as display text only.

A title read from a file the agent rewrites is mutable by design. Keying a URL or a sidecar path on it would break every existing link and orphan every recorded comment the first time the exploration's topic shifted.

#### Scenario: A title change does not change any address
- **WHEN** the first heading of a session's scratch note is rewritten
- **THEN** the session's page URL, its comment sidecar path, and its verdict sidecar path SHALL be unchanged
