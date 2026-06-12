## ADDED Requirements

### Requirement: Active change companion navigation
The browser viewer SHALL provide navigation to review companion mode only when companion mode is explicitly enabled.

#### Scenario: Companion link appears when enabled
- **WHEN** a user opens an active change review page while review companion mode is enabled
- **THEN** the page provides a link or control to open the same change in companion mode

#### Scenario: Companion link hidden when disabled
- **WHEN** a user opens an active change review page while review companion mode is disabled
- **THEN** the page does not provide browser-side feedback controls or companion write actions

### Requirement: Review companion preserves default read-only viewer
The browser viewer SHALL keep default `serve` behavior read-only for target OpenSpec artifacts.

#### Scenario: Default serve remains read-only
- **WHEN** a user runs `openspec-doc serve` without review companion mode
- **THEN** browser pages do not create, modify, or delete OpenSpec artifacts or review sidecar files

#### Scenario: Companion writes are scoped
- **WHEN** review companion mode is enabled and a user saves review feedback
- **THEN** the write is limited to the documented review sidecar storage path
- **AND** no file under the target project's `openspec/` directory is modified
