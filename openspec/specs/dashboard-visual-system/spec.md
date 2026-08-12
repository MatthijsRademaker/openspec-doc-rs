# dashboard-visual-system Specification

## Purpose

Define dashboard-wide observatory identity, typography, semantic state language, responsive composition, motion, accessibility, and runtime-asset discipline.
## Requirements
### Requirement: The dashboard presents one observatory visual identity

The system SHALL present every dashboard route on a near-black layered canvas with bone-toned content, hairline depth, restrained geometry, and cartographic instrument motifs. It SHALL NOT present a generic light dashboard theme or a theme-selection control.

#### Scenario: Dashboard arrives in the observatory theme

- **WHEN** a reviewer opens the dashboard with no stored browser preferences
- **THEN** the system SHALL render the dark observatory interface on first paint without requiring a theme initialization script

#### Scenario: Old theme preference does not alter the interface

- **WHEN** a browser retains a theme preference written by the previous interface
- **THEN** the system SHALL still render the sole observatory theme and SHALL NOT offer a theme toggle

### Requirement: Typography communicates document and instrument roles

The system SHALL use a bundled display serif for product and document gravity, IBM Plex Mono for operational labels and code-shaped values, and a bundled readable text face for sustained prose and comment bodies. It SHALL make no remote font request.

#### Scenario: Document title and metadata have different voices

- **WHEN** a view renders a document or scope title with its path, timestamp, or state metadata
- **THEN** the system SHALL render the title in the display role and the operational metadata in the mono role

#### Scenario: Typography works offline

- **WHEN** the embedded dashboard is used with network access disabled
- **THEN** all display, prose, and mono text SHALL render from assets embedded in the binary

### Requirement: Review meaning is not carried by color alone

The system SHALL render open, addressed, resolved, verdict, delivery, reviewer, and agent meanings through central semantic tokens combined with visible text and distinct glyph, line, or layout treatment. Components SHALL NOT depend on raw local colors to communicate those meanings.

#### Scenario: State remains identifiable without hue

- **WHEN** a reviewer sees open, addressed, and resolved states with color perception unavailable
- **THEN** each state SHALL remain identifiable from its visible label and non-color treatment

#### Scenario: Reviewer and agent remain distinguishable

- **WHEN** a conversation contains both reviewer and agent entries
- **THEN** role labels and structural treatment SHALL distinguish them without relying on color

### Requirement: Celestial artwork frames rather than obscures review content

The system SHALL use celestial, orbital, crosshair, scan-line, and stipple motifs only as atmosphere or wayfinding around content. Artwork SHALL NOT reduce readability, cover controls, intercept interaction, or sit beneath primary document prose or instrumentation text. An artwork's legible extent SHALL be a function of the space its container's content leaves free, and SHALL NOT be a fixed fraction of the viewport that content can grow into. Artwork MAY extend behind the content above it only through a declared fade, and SHALL reach full strength only in free space. Artwork MAY additionally continue behind a fully opaque panel, because the panel and not the artwork is what the text sits on; that continuation SHALL be through the same declared fade, SHALL reach the panel already attenuated, and SHALL NOT be used to place any text on a translucent or partially transparent surface.

Bounding artwork by the viewport works only while the content beside it stays short. A rail whose list grows with the number of artifacts will eventually reach into a region sized without reference to it, and the resulting overlap appears as a function of fixture size — visible with six artifacts, invisible with three, and therefore invisible to a suite that keeps its fixtures small.

The fade is the distinction that matters. Atmosphere rising behind the foot of a list is what the rails are composed around; stipple at full strength under a mono path is the defect. Making the fade a declared distance rather than a judgement call keeps the second testable without banning the first. Opacity of the panel is the same kind of distinction: a field continuing behind a solid register is composition, and the same field behind a translucent one is the defect, because only in the second case is anything read through it.

#### Scenario: Artwork yields to document content

- **WHEN** a view places celestial artwork near a document heading or content region
- **THEN** the artwork SHALL occupy a bounded framing region and the document text SHALL retain an unobstructed high-contrast surface

#### Scenario: Artwork yields to instrumentation content

- **WHEN** a rail's instrumentation content grows to occupy the region an artwork would otherwise fill
- **THEN** the artwork's legible region SHALL shrink or disappear, and no identifier, path, state, or label SHALL be rendered over any part of the artwork outside its declared fade

#### Scenario: A field continuing behind a panel keeps text off it

- **WHEN** an observation field continues behind a content panel
- **THEN** that panel SHALL be fully opaque, the field SHALL be attenuated by its declared fade before it reaches the panel, and no text, identifier, state, link, or focus indicator SHALL be rendered against the field itself

#### Scenario: Narrow layout removes atmosphere first

- **WHEN** the dashboard is rendered at 390 CSS pixels wide
- **THEN** decorative artwork SHALL shrink, crop, or disappear before any identifier, state, comment, or control is hidden or clipped

### Requirement: The document's readable measure is stable across chrome changes

The system SHALL hold the selected document's readable measure constant when surrounding chrome is shown, hidden, collapsed, or expanded, and SHALL absorb the freed or lost width as margin rather than as line length.

A control that collapses a rail exists to get the rail out of the way. If collapsing it also re-wraps every paragraph in the document, the reviewer loses their place in the text as the price of putting the threads away, and the two effects are indistinguishable from each other.

#### Scenario: Collapsing a rail does not re-wrap the document

- **WHEN** a reviewer collapses or expands a rail beside the selected document
- **THEN** the document's line breaks SHALL be unchanged and the freed width SHALL become margin

#### Scenario: The measure still yields at narrow widths

- **WHEN** available width is narrower than the document's readable measure
- **THEN** the document SHALL use the available width, without horizontal page overflow

### Requirement: Layout remains dense, calm, and responsive

The system SHALL compose information on an 8px rhythm with hairline separation and readable document measure. Every route SHALL occupy the full viewport width from one shared shell declaration rather than holding its own width ceiling, so that no route renders as a centred column beside one that fills the display. Readable measure SHALL be held by the document's own measure rules rather than by capping the page. At narrow widths the system SHALL preserve content, identifiers, states, and controls in one logical flow without horizontal page scrolling.

Two routes each stating their own width is how the index came to stop at 1440px while the scope workbench ran edge to edge — the same decision written twice, free to drift, and drifted. Stating it once is what stops the next route disagreeing with both.

#### Scenario: Desktop preserves hierarchy

- **WHEN** the dashboard is rendered at 1280 CSS pixels or wider
- **THEN** product instrumentation, primary content, metadata, and review state SHALL be visually distinct without reducing the primary content to a narrow card column

#### Scenario: Every route occupies the display it is given

- **WHEN** the index and a scope route are each rendered at 2560 CSS pixels wide
- **THEN** both SHALL use the full viewport width at the same shell padding, neither SHALL reserve symmetric empty canvas by capping its own width, and the two SHALL read as the same product at the same scale

#### Scenario: Shell width is declared once

- **WHEN** a route's outer shell width or padding is set
- **THEN** it SHALL be read from the shared shell declaration rather than restated locally, and a route restating its own ceiling SHALL be treated as a defect

#### Scenario: Narrow view preserves information

- **WHEN** the dashboard is rendered at 390 CSS pixels wide
- **THEN** long identifiers SHALL wrap, metadata and states SHALL remain visible, controls SHALL remain reachable, and the page SHALL have no horizontal overflow

### Requirement: Motion explains state without becoming spectacle

The system SHALL express dashboard motion as a short event-driven instrument language for acquiring coordinates, triangulating linked content, transmitting reviewer intent, receiving content, resolving semantic state, and reconfiguring interface chrome. Motion SHALL be tied to real navigation, mutation, state, or remote-update events; SHALL complete standard transitions within 100–200 milliseconds; and SHALL complete coordinate acquisition, the sole exception to that bound, within 600 milliseconds, of which the coordinate transfer and the source's rise SHALL share one shorter duration and the identity's halftone resolve MAY take the whole.

Motion MAY hold a navigation before it commits, where holding it is what lets the gesture terminate at a real destination instead of at scaffolding. A hold SHALL leave the origin fully rendered and interactive for its whole duration, SHALL end at a declared ceiling after which the navigation commits without the gesture, and SHALL commit the navigation immediately if what it waits for fails. Motion SHALL NOT hold a navigation for decoration that would look the same without the wait, SHALL NOT hold one without a declared end, and SHALL NOT leave a reviewer's action unanswered. The system SHALL NOT use decorative perpetual motion, parallax, pointer-following fields, cursor trails, sound, whole-page glitch effects, spring spectacle, or animated loading ornament.

A blanket rule that motion may never delay content reads as discipline and is not. It was written against decoration that makes a reviewer wait for nothing, but it also forbids the one case where waiting is the difference between a gesture that lands on the thing the reviewer selected and one that lands on a placeholder — and the dashboard shipped the placeholder version under it. What matters is not whether a wait exists but whether it is bounded, whether the reviewer is stranded during it, and whether anything is actually withheld: an origin page that stays live withholds nothing.

Motion MAY use opacity, slight position, registration-line growth, hard masks, clipping, halftone reveals in the plates' own texture, or glyph and line-geometry changes when those effects explain the event and leave no decorative movement after settling. A mark reporting an arrival MAY remain visible longer than a transition takes to complete, and SHALL decay on its own rather than persisting as a state the interface must later clear. The duration bound governs how long a transition takes to finish; a bounded hold before a navigation commits is governed by its own declared ceiling, and neither requires a nonblocking notice to vanish before it has been seen.

#### Scenario: Navigation acquires a real coordinate

- **WHEN** a reviewer navigates to a scope, artifact, or linked thread destination
- **THEN** any acquisition motion SHALL originate from a real selected coordinate or control and SHALL terminate at the real destination rather than at a loading placeholder, and route, history, focus, and content updates SHALL NOT be held back to let decoration finish once the navigation has committed

#### Scenario: A navigation hold is bounded and answers the action

- **WHEN** a navigation is held so its gesture can terminate at a destination that does not exist yet
- **THEN** the origin SHALL remain rendered and interactive throughout, the hold SHALL end at a declared ceiling after which the route commits without the gesture, a failed load SHALL commit the route so the destination reports its own failure, and what the hold waited for SHALL be handed to the destination rather than fetched again

#### Scenario: Transmission follows real request state

- **WHEN** a reviewer submits a comment, reply, status change, or verdict
- **THEN** transmission treatment SHALL remain in flight only while the real mutation is unresolved, SHALL terminate at the actual successful destination, and SHALL NOT present successful receipt when the mutation fails

#### Scenario: Semantic state remains immediate

- **WHEN** an open, addressed, resolved, verdict, or delivery state changes
- **THEN** visible text and final non-color geometry SHALL update from real state without waiting for its decorative transition to complete

#### Scenario: Remote content receives a bounded arrival mark

- **WHEN** content arrives through a live update the reviewer did not initiate
- **THEN** receive treatment MAY mark the surface where replacement lands but SHALL NOT obscure content, intercept interaction, move reading position, or claim which document blocks changed

#### Scenario: Standard motion is bounded

- **WHEN** a panel, route, review state, or instrument event enters or changes
- **THEN** each standard transition SHALL complete within 100–200 milliseconds and SHALL leave the interface still after the event settles

#### Scenario: Acquiring a coordinate is bounded above the standard transition

- **WHEN** a reviewer selects an artifact coordinate, the coordinate transfers from the rail to the document header, and the document reports its own arrival
- **THEN** the transfer and the source's rise SHALL run one duration so the movement reads as a single gesture, the identity's halftone resolve MAY outlast them because it is watched rather than read through, no part SHALL exceed 600 milliseconds, this SHALL be the only motion granted more than the standard bound, and the channel that gates them SHALL dwell no less than the longest of them so the reviewer sees every gesture the interface declares

#### Scenario: A suspended-input window is bounded and costs no reviewer action

- **WHEN** a coordinate transfer runs, replacing the live document with a snapshot so that the interface cannot be clicked while it does
- **THEN** that window SHALL last no longer than the acquisition duration, SHALL never exceed it by growing a treatment the snapshot does not need, and SHALL cost only pointer targeting: by the time it opens, the reviewer's originating action SHALL have taken effect, and route, history, focus, semantic text, and the selected content SHALL already have updated — for a navigation this means the window opens after the route has committed, never during a hold that precedes it

#### Scenario: Acquired identity resolves in the plate's own texture

- **WHEN** the acquisition gesture reports an artifact or scope whose source has replaced the previous one
- **THEN** the identity SHALL resolve through the halftone the observatory plates are built from, advancing in discrete passes rather than as a continuous fade, the dot grid SHALL re-register between passes so the name breaks up differently each time rather than one fixed screen filling in, the resolve SHALL start deep enough that the name reads as texture rather than as text behind a screen, and it SHALL NOT be reported instead by lit bars travelling across the header or across an index entry, which describe nothing about the surface they cross

#### Scenario: Acquired source rises into a fixed frame

- **WHEN** the identity resolves
- **THEN** the source blocks MAY rise into place from a displacement bounded by the shared displacement token, the identity and its plate SHALL stay fixed as the frame that displacement resolves against, the source SHALL reach full legibility within the standard transition bound rather than the gesture's, and neither mask nor offset SHALL survive the gesture

#### Scenario: A resolving surface is eaten, never withheld

- **WHEN** either surface is part-way through the gesture
- **THEN** it SHALL already be rendered at partial coverage rather than absent, its legibility SHALL only improve as the gesture runs, and the reviewer SHALL be able to read, select, and comment on it throughout

#### Scenario: An arrival mark decays without blocking

- **WHEN** the interface marks content that arrived without the reviewer acting
- **THEN** the mark SHALL fade out on its own, SHALL leave no residual state once it has, and SHALL NOT intercept interaction or delay access to the content it marks

#### Scenario: Loading is not decorated

- **WHEN** the interface is waiting for a scope, an index, or a mutation to resolve
- **THEN** it SHALL report the wait in text and SHALL NOT present an animated loading ornament

#### Scenario: Narrow motion preserves content order

- **WHEN** an event occurs at 390 CSS pixels wide and its desktop source and destination no longer occupy adjacent planes
- **THEN** the system SHALL use local or reading-direction treatment and SHALL NOT draw a connector across document content, reorder content, create horizontal overflow, or hide controls

#### Scenario: Reduced motion is immediate

- **WHEN** the reviewer requests reduced motion
- **THEN** native view transitions, smooth scrolling, transforms, masks, clipping transitions, decorative displacement, and navigation holds SHALL be removed or made immediate, while text, focus, semantic state, and final geometry remain understandable

#### Scenario: Reduced motion covers effects added later

- **WHEN** a new transition or animation is added to the interface
- **THEN** it SHALL be suppressed under reduced motion without requiring that it be listed individually, and any effect that must survive reduced motion SHALL opt out explicitly

#### Scenario: Celestial artwork remains still

- **WHEN** orbital, face, stipple, scan-line, or star-system artwork frames an interaction
- **THEN** the artwork SHALL remain static unless a bounded non-interactive overlay reports a real event, and it SHALL NOT track the pointer or animate perpetually

### Requirement: Coordinate acquisition is the reference gesture for dashboard motion

Motion rules stated as prohibitions produce treatments that break none of them and mean nothing. The system SHALL therefore treat coordinate acquisition — the identity resolving through the plate's halftone, the source rising into that fixed frame, and the coordinate transferring from rail to header — as the worked example of dashboard motion, and any new treatment SHALL be justified against the rules it demonstrates rather than against the prohibition list alone. Each rule below is drawn from a treatment this dashboard shipped and then replaced, not from taste: acquisition previously reported itself with a lit bar travelling across the artwork under two registration rules, which broke no prohibition and still had to go.

#### Scenario: A treatment says what the surface is made of

- **WHEN** a treatment is chosen to report an event on a surface
- **THEN** it SHALL be drawn from what that surface is — its texture, its geometry, or the instrument it represents — rather than from generic interface movement that would look the same on any surface, and a treatment that merely travels across a surface SHALL be rejected in favour of one that resolves it

#### Scenario: Instrument motion is quantized, not continuous

- **WHEN** a treatment reveals, resolves, or replaces content over time
- **THEN** it SHOULD advance in discrete passes rather than as a continuous ramp, because a continuous version of the same geometry reads as an ordinary fade and reports nothing about the instrument performing it

#### Scenario: A surface under treatment is eaten, never withheld

- **WHEN** a treatment operates on content the reviewer came to read
- **THEN** that content SHALL already be rendered when the treatment starts, its legibility SHALL only improve as the treatment runs, and the reviewer SHALL be able to read, select, and act on it throughout

#### Scenario: Duration is chosen against the mechanism, not the feel

- **WHEN** a duration is chosen for a treatment
- **THEN** a treatment carried by a mechanism that suspends interaction SHALL have its duration read as a latency budget and stay within the standard bound, a treatment carried by real elements MAY run longer because it costs the reviewer nothing, and the two SHALL be allowed to differ rather than being unified into one number

#### Scenario: Whatever gates a treatment outlasts it

- **WHEN** a treatment is gated by a class, channel, or state that is cleared on a timer
- **THEN** that gate SHALL persist at least as long as the longest treatment it gates, and a declared duration longer than its gate SHALL be treated as a defect, because the interface then never shows the motion its own source declares

### Requirement: Design references are not runtime payload

The system SHALL keep visual source references outside the frontend runtime asset tree and SHALL embed only explicitly selected, optimized runtime derivatives. The complete runtime raster image payload SHALL not exceed 6 MiB.

#### Scenario: Reference boards are absent from the build

- **WHEN** the frontend production build completes
- **THEN** neither `design-system.png` nor `dashboard-mockup.png` SHALL appear in `dist/`

#### Scenario: Runtime images are bounded

- **WHEN** the production build's runtime raster images are measured
- **THEN** every image SHALL be on the runtime allowlist and their combined size SHALL be at most 6 MiB

### Requirement: The index uses an authored image-led observation composition

The system SHALL make the index artwork a load-bearing part of its observatory composition through a dominant asymmetric observation field and a lower strip of hard-cropped visual plates. It SHALL preserve black negative space, halftone texture, clipped celestial geometry, and hairline alignment from the approved visual sources, and SHALL NOT reduce those sources to generic image cards. The observation field SHALL continue past the hero as the ground the scope panelling sits on, through the declared fade, and SHALL NOT be rendered at full strength beneath that panelling or used as undifferentiated wallpaper.

The index hero SHALL hold its composition proportionally at every width: the panel, the divider, and the artwork crop SHALL be expressed in the same units, so that widening the display enlarges the composition rather than opening a gap inside it. A panel bounded by a fixed length while the divider beside it is bounded by a percentage agrees only over the narrow range where the two happen to coincide.

#### Scenario: Main observation field uses the approved face and orbit source

- **WHEN** the index renders at desktop width
- **THEN** it SHALL render an optimized derivative of `designs/visual-language/main-panel-background.png` with the face and orbital field entering asymmetrically from the right and with the source's baked-in far-left pseudo-control rail excluded from the runtime crop

#### Scenario: The hero holds its proportions at any width

- **WHEN** the index is rendered at 1280 and at 2560 CSS pixels wide
- **THEN** the hero panel, its divider, and the artwork crop SHALL occupy the same proportions of the page at both widths, and no band of bare canvas SHALL open between the panel's edge and the artwork at either

#### Scenario: The observation field grounds the panelling below the hero

- **WHEN** the index renders its change and session registers at desktop width
- **THEN** the observation field SHALL continue behind them as attenuated ground visible around and between the opaque registers, SHALL reach full strength only in the hero's free space, and SHALL end above the lower plate strip rather than compounding with it

#### Scenario: Lower plates use the approved abstract sources

- **WHEN** the complete desktop index renders
- **THEN** it SHALL include a subordinate hard-edged plate strip using optimized derivatives of `abstract-sun.png`, `abstract-face.png`, and `abstract-star-system.png`, without captions or labels that imply activity or lifecycle state

#### Scenario: Artwork and content occupy distinct planes

- **WHEN** the index observation field or lower plates appear near scope content
- **THEN** artwork SHALL remain non-interactive and absent from the accessibility tree, and no text, link, focus indicator, loading message, failure message, or state label SHALL be rendered against artwork rather than against an opaque surface

#### Scenario: Index atmosphere yields at narrow width

- **WHEN** the index is rendered at 390 CSS pixels wide
- **THEN** the main artwork SHALL reduce to a bounded crop, the field SHALL stop continuing behind the panelling, and the lower plate strip SHALL disappear before any scope identity, metadata, state, or link is hidden or clipped

### Requirement: The index runtime artwork is an exact optimized derivative set

The system SHALL ship the index and scope observation artwork only as one shared set of purpose-cropped WebP derivatives in named observatory runtime slots, SHALL keep source PNGs outside the frontend runtime tree, and SHALL remove the superseded index-prefixed filenames rather than duplicating equivalent raster payload for each route.

#### Scenario: Production build contains the shared observatory derivative set

- **WHEN** the frontend production build completes
- **THEN** `dist/` SHALL contain `assets/images/observatory-field.webp`, `assets/images/observatory-plate-sun.webp`, `assets/images/observatory-plate-face.webp`, and `assets/images/observatory-plate-star-system.webp` as allowlisted runtime rasters used by both index and scope compositions

#### Scenario: Index-specific aliases and source artwork are absent from runtime

- **WHEN** the frontend production build completes
- **THEN** `index-observation-field.webp`, every `index-plate-*.webp` alias, `index-orbit.webp`, `main-panel-background.png`, `abstract-sun.png`, `abstract-face.png`, and `abstract-star-system.png` SHALL NOT appear in `dist/`

#### Scenario: Shared route artwork remains within runtime budget

- **WHEN** the production build's runtime raster payload is measured
- **THEN** every raster SHALL be on the explicit allowlist, equivalent index/scope copies SHALL NOT coexist, and the combined payload SHALL remain at or below 6 MiB

### Requirement: Scope routes use an authored image-led observation composition

The system SHALL compose session and change review routes as one full-bleed observatory instrument with compact route/state chrome, a primary selected-document stage, exact-path instrumentation, artifact-scoped conversation, and peripheral decision controls. The shared face/orbit observation field SHALL enter asymmetrically through a bounded portion of the selected document's arrival plane, while shared observation plates MAY occupy subordinate peripheral chassis cells; artwork SHALL NOT become a detached pre-document gallery or wallpaper behind review content.

#### Scenario: Scope uses connected full-bleed instrument geometry

- **WHEN** a scope route renders at desktop width
- **THEN** route state, artifact instrumentation, selected document, artifact conversation, and decision controls SHALL read as one hairline-connected workbench that uses available viewport width rather than isolated centered cards

#### Scenario: Selected document appears in initial desktop viewport

- **WHEN** the deterministic change fixture renders at 1536 by 1024 CSS pixels
- **THEN** the selected artifact identity and its first meaningful source content SHALL be visible without scrolling past an oversized route hero or image gallery

#### Scenario: Observation field belongs to document arrival

- **WHEN** a selected artifact renders at desktop width
- **THEN** the optimized shared derivative of `main-panel-background.png` SHALL place halftone face and clipped orbital geometry asymmetrically in a bounded arrival region of the selected document, distinct from protected source text, exact paths, state, comments, and focus indicators

#### Scenario: Route header remains operational rather than cinematic

- **WHEN** a titled scope with verdict and delivery state opens
- **THEN** route title, exact identifier, standing verdict, submission age, delivery state, and history SHALL remain compact and legible without forming a second oversized document hero

#### Scenario: Scope does not render a three-equal-plate preamble

- **WHEN** a scope route renders its selected artifact
- **THEN** the sun, face, and star-system derivatives SHALL NOT appear as an equal-width gallery that delays the selected document

#### Scenario: Peripheral plates remain subordinate

- **WHEN** shared observation plates appear in desktop scope composition
- **THEN** they SHALL occupy hard-edged peripheral rail or footer chassis cells, SHALL carry no interactive behavior or state-implying caption, and SHALL yield before document measure, paths, comments, state, or controls

#### Scenario: Artwork is decorative by construction

- **WHEN** scope observation artwork renders
- **THEN** every image SHALL use empty alternative text, remain absent from the accessibility tree, reject pointer interaction, and SHALL NOT sit beneath prose, links, focus indicators, markers, threads, composers, status labels, loading state, or errors

#### Scenario: Empty conversation uses instrument geometry without fake activity

- **WHEN** the selected artifact contains no threads
- **THEN** its conversation region MAY use hairlines, nodes, crosshairs, or restrained artwork but SHALL NOT invent comments, activity events, agents, validation, or review state

#### Scenario: Scope artwork yields at intermediate width

- **WHEN** available width cannot preserve peripheral artwork alongside readable selected-document and conversation measures
- **THEN** peripheral plate cells SHALL disappear before document, exact-path navigation, state, comments, or controls are reduced or hidden

#### Scenario: Scope artwork yields at narrow width

- **WHEN** a scope route is rendered at 390 CSS pixels wide
- **THEN** peripheral plate cells SHALL be absent and the main observation field SHALL reduce to a bounded shallow crop or disappear before any scope identity, artifact path, document content, comment, composer, state, or control is hidden or clipped

