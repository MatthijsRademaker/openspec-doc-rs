# Observatory review workbench

Status: implementation reference for `web/` dashboard work. Source boards live in
`designs/visual-language/`; they guide composition but never enter runtime builds.

## Required visual source

Read these before changing dashboard layout, typography, state, or motion:

1. `designs/visual-language/design-system.png` — palette, typography, rhythm, states,
   geometry, and motion.
2. `designs/visual-language/dashboard-mockup.png` — document-first composition and balance
   between review content and celestial framing.
3. Supporting PNGs in `designs/visual-language/` — motif vocabulary only. They are not a
   runtime asset library.

Product posture: observatory for agent-guided spec review. Reviewer observes, agent acts as
instrument, spec remains primary object. Reject generic SaaS chrome, rounded card grids,
drop shadows, bright status fills, stock dashboard sidebars, and detached chat layouts.

## Identity and typography

Dashboard has one dark identity from first paint. Canvas is near-black, content is bone,
and depth comes from stepped surfaces plus hairlines. No theme control or persisted browser
preference may alter it.

Use three bundled roles:

- **Display — Cormorant Garamond:** product wordmark, route and artifact titles,
  requirement identifiers, major document headings.
- **Prose — IBM Plex Sans Variable:** sustained artifact prose and comment bodies.
- **Instrument — IBM Plex Mono:** paths, identifiers, timestamps, state labels, commands,
  compact controls, and uppercase microcopy.

No remote font, icon, or image request. Long identifiers stay exact and wrap.

## Instrument language

Microcopy is terse, operational, and calm: `SCOPE REGISTER`, `OBSERVING INDEX`,
`3 OPEN COMMENTS`, `MOVE TO PROPOSAL`. Avoid marketing language, emoji, decorative status,
or invented repository/activity facts.

Every state combines central token, visible text, and distinct glyph or line treatment:

| Meaning | Visible treatment |
| --- | --- |
| Open | `◇ Open` plus interrupted line |
| Addressed | `↗ Addressed` plus directional line |
| Resolved | `✓ Resolved` plus closed line |
| Verdict | `◆ <verdict>` plus decision rule |
| Delivery | `→ <delivery state>` plus transport rule |
| Reviewer | `● Reviewer` plus document-side rule |
| Agent | `□ Agent` plus instrument-side rule |

Color is secondary. Components use semantic classes backed by `web/src/style.css`; no
component-local raw state colors.

## Composition

Every scope route uses one selected-artifact workbench:

```text
compact route identity + standing review state
scope/artifact rail | complete selected document | artifact conversation
                    | browser window scrolls     | loose comments + decisions
```

Change routes expose every server-supplied artifact through exact-path navigation. Exact
`Artifact.path` owns `?artifact=`; first load canonicalizes missing selection with Router
replace, deliberate selection pushes history, and invalid or removed coordinates stay
visible as unavailable state. Session routes select sole scratch artifact without fake tabs,
folders, or destinations.

Selected artifact renders completely in source order. Browser window remains sole primary
scroll owner; document stage never gains fixed-height primary scroller. Resolved exact and
fuzzy threads appear only in selected-artifact conversation. Marker and thread activation
lock same source block visibly and navigate in both directions. Missing, orphaned, and
unanchored threads remain in decision instrument. At narrow width DOM order is route state,
artifact identity/navigation, complete document, artifact conversation, loose comments,
then decisions—without duplicate interactive thread markup.

The decision instrument is one trigger and one drawer, positioned separately. At desktop the
trigger is laid out at the foot of the conversation rail, which tracks the viewport so the
trigger stays reachable from any scroll position in a document taller than the rail; with the
rail collapsed the trigger keeps its glyph and drops its label to fit the collapsed column. At
narrow width the layout is one column and the rail is last in the flow, so the trigger is
instead docked fixed above the safe-area inset and the flow reserves clearance below itself.
The drawer is modal in both: teleported out of the layout, backdropped, focus-trapped, and
overlaying the page only while a reviewer holds it open.

The selected document holds a readable measure derived from the viewport rather than from its
column, so collapsing the conversation rail turns freed width into margin instead of re-wrapping
prose. The measure grows with the viewport on stages wide enough that a fixed one would spend the
growth on void, but never with the column.
The collapse control therefore buys the reviewer quiet — the threads go away and the chassis
breathes — and no longer buys a wider document. That is a smaller benefit than it used to
advertise; if it turns out not to be worth a control, remove the control rather than restore
the reflow.

Index uses same language at lower density: instrument masthead, separate ruled session and
change registers, exact identifiers, complete metadata, explicit recent activity, and
distinct loading, empty, and failure instruments. It must not invent scope routes,
repository names, activity events, or review controls unsupported by current data.

## Motif boundary

Orbital maps, crosshairs, scan lines, nodes, and stipple may frame headings, unused margins,
rail thumbnails, and empty instruments. They are `aria-hidden`, non-interactive, and never
sit behind prose, controls, focus rings, or state labels at legible strength. Rail artwork
occupies the space its rail's content leaves free and may rise a declared `--art-bleed` behind
the foot of that content, inside a fade three times as long — so the region behind a path or a
thread is always a trace, and artwork with no free space below it never resolves at all. At
narrow width atmosphere shrinks or disappears before content.

The index observation field is the one artwork that continues past its own band. It is a
page-level layer beneath the hero *and* the change and session registers, sized by the source's
own proportions rather than by the band, so widening the display enlarges the composition instead
of opening a gap inside it. Its fade is a declared distance and not a judgement: full strength
lasts only as long as the hero's free space, the field reaches the top of the register panelling
at roughly a third of full strength, and it decays to nothing over the remainder. It ends above
the lower plate strip, which carries its own hard-cropped artwork and would otherwise compound
with it. This stays composition rather than wallpaper for one structural reason — the registers
are opaque `--surface` panels, so the field is seen *around and between* them and never through
them, and no identifier, metadata value, state label, link, or focus ring is ever read against
it. The gaps in the register grid are therefore load-bearing: welding the registers into one
chassis would leave the bleed nowhere to show. At 390px the field stops being ground and returns
to the bounded crop below the hero.

Stipple has one sanctioned use *on* content rather than beside it: an acquired artifact identity
resolves through a halftone mask, so for the length of that gesture the dot grid is what the title
is made of and it prints in the same texture as the plate beside it. It is a transient reveal and
not a layer — it advances monotonically toward full coverage, never starts from nothing, and leaves
no mask behind. Any other motif over live text is out.

Only named runtime slots may use raster derivatives. Shared observatory allowlist:

| Runtime slot | Source and crop | Output | Index and scope roles |
| --- | --- | --- | --- |
| `web/public/assets/images/observatory-field.webp` | `main-panel-background.png`; crop `x=128, y=75, w=1408, h=845` excludes far-left pseudo-control rail while retaining black safe space and right-side face/orbit field | 1200×720 WebP, quality 84, 95,710 bytes | Decorative asymmetric field. Index: right-entering masthead art beside protected text, continuing below the hero as the attenuated ground the opaque registers sit on, ending above the plate strip, then bounded crop at 390px. Scope: bounded right-entering selected-document arrival plane beside opaque artifact identity; hidden at 390px before document measure or controls yield. |
| `web/public/assets/images/observatory-plate-sun.webp` | `abstract-sun.png`; crop `x=196, y=120, w=1280, h=640` centers dark solar aperture and radiating line field | 640×320 WebP, quality 82, 65,168 bytes | Decorative first hard-edged plate in index lower strip. Scope has no pre-document gallery; any future scope use is limited to subordinate peripheral chassis cells. |
| `web/public/assets/images/observatory-plate-face.webp` | `abstract-face.png`; crop `x=196, y=150, w=1280, h=640` isolates eye and halftone texture rather than repeating masthead face composition | 640×320 WebP, quality 82, 73,302 bytes | Decorative second hard-edged plate in index lower strip. Scope has no pre-document gallery; any future scope use is limited to subordinate peripheral chassis cells. |
| `web/public/assets/images/observatory-plate-star-system.webp` | `abstract-star-system.png`; crop `x=168, y=230, w=1280, h=640` places orbital intersections and nodes off center | 640×320 WebP, quality 82, 15,416 bytes | Decorative third hard-edged plate in index lower strip. Scope has no pre-document gallery; any future scope use is limited to subordinate peripheral chassis cells. |

These four shared slots replace `index-orbit.webp` and every index- or scope-prefixed alias;
retaining aliases would create competing art systems and duplicate payload. Every image and
plate group remains non-interactive, uses empty alt text, is `aria-hidden`, and stays outside
text and focus planes. Original source boards remain in `designs/visual-language/`. Runtime
raster payload is checked after every production build and must stay at or below 6 MiB.

## Geometry, responsiveness, and motion

Use 8px rhythm, with 4px only for tight icon/text alignment. Prefer square or 2–4px corners,
hairline rules, and stepped surfaces. Pills belong only to compact state/control shapes.
Focus is visibly offset from borders.

- **Desktop, 1280px+:** full-bleed hairline chassis reserves roughly 14–18rem for scope and
  artifact instrumentation, flexible readable document measure, and 20–26rem for selected
  artifact conversation plus decisions. Route chrome stays compact enough for artifact
  identity and first meaningful source block to enter 1536×1024 initial viewport.
- **Intermediate:** instrumentation enters flow first, then conversation follows document;
  artwork and peripheral cells yield before paths, prose, state, comments, or controls.
- **Narrow, 390px:** one logical flow, no page-level horizontal overflow. Exact paths wrap,
  complete document precedes conversation, comment actions stay visible without hover, and
  final content clears decision controls and safe-area inset.

Motion uses six event verbs tied to real system work:

| Verb | Trigger | Geometry | Settled truth |
| --- | --- | --- | --- |
| Acquire | Scope, artifact, or thread selection | Target ticks, and — for an artifact — the identity printing out of the plate's halftone in discrete passes on a re-registering grid while the source rises into that fixed frame | Existing selected/current state |
| Triangulate | Source/thread navigation | Paired origin/destination locks and bounded edge trace | Existing active block and thread |
| Transmit | Unresolved reviewer mutation | Carrier rule at submitting surface | Returned thread, status, or verdict |
| Receive | Successful mutation or remote update | Landing edge or lock at the surface the content landed on | Updated content and semantic state |
| Resolve | Status or delivery change | Glyph and line geometry closes or redirects | Immediate visible label and final glyph |
| Reconfigure | Rail or decision surface opens/closes | Hard mask and compatible chassis-track movement | Existing expanded, collapsed, or modal state |

One event produces one hero gesture. Same-channel events replace older marks; document receipt
and reviewer confirmation use independent channels. Effects disappear after reporting and never
become persisted state. Desktop triangulation may use a viewport-edge direction trace, while
narrow flow keeps marks local and vertical rather than drawing through content.

Standard transitions finish in 100–200ms using the central 160ms token and
`cubic-bezier(0.2, 0, 0.2, 1)`. Acquire is the one exception. The coordinate transfer and the
source's rise share a 400ms token so the movement reads as one gesture rather than a fast transfer
followed by a slow landing; the identity's halftone resolve runs a longer 600ms token of its own,
because it is the one treatment a reviewer watches rather than reads through and at 400ms it was
over before the eye had settled on the name. The transfer is a native view transition, which
holds the page un-hit-testable while it runs, so that token is a latency budget as much as a motion
one — raising it lengthens the window in which the rail ignores clicks by the same amount. The cost
is pointer targeting only: route, history, focus, semantic text, and content have all updated
before the window opens, and the click that opened it has already taken effect. The event channel
gating a gesture dwells at least as long as the gesture, since a shorter dwell truncates the
animation and the declared duration becomes one the interface never shows. Allowed geometry includes
opacity, slight registration displacement, line growth, hard masks, clipping, halftone reveals in
the plates' own texture, and glyph/line metamorphosis. A nonblocking arrival report may dwell longer
when it withholds nothing, intercepts nothing, and clears itself. Content, Router history, focus,
semantic text, and final geometry update immediately rather than waiting for presentation.

### Acquire is the worked example — build new motion against it

A prohibition list produces treatments that break none of the rules and mean nothing. Acquire is
the reference gesture; check a proposed treatment against these five, all of which it earns
concretely. Acquire previously reported itself with a lit bar sliding across the artwork under two
registration rules — that broke no prohibition and was still wrong.

1. **Say what the surface is made of.** The identity sits beside a dithered photographic plate, so
   it prints in that plate's halftone: a 6px dot cell masking the title, coverage 30% → 100%. The
   bar it replaced would have looked identical over a chart, a table, or a login form. If a
   treatment would suit any surface, it is describing none.
2. **Quantize it.** `steps(6, jump-start)`, and the grid re-registers on each pass via
   `mask-position`, so the name breaks up differently every strike. The same geometry run
   continuously is just a fade. The steps *are* the instrument.
3. **Eat, never withhold.** Coverage starts at 30%, not 0%, and only climbs. The source rises from
   `2 × --motion-rule` with its fade finished by 35% — inside the standard bound — so the gesture
   may run long but the reading never waits for it.
4. **Pick the duration against the mechanism.** The transfer is a native view transition and
   suspends hit testing for exactly its own length, so it stays on the 400ms token; the halftone is
   plain CSS on a real element and costs nothing, so it takes 600ms. Two numbers, deliberately, and
   `--motion-duration-acquire` and `--motion-duration-resolve` are separate for that reason. Merging
   them re-buys ~200ms of dead rail on every artifact click.
5. **Make the gate outlast the gesture.** `EVENT_ACQUIRE_MS` covers the longest animation the
   acquired class gates. It once did not, and the 500ms gesture ran for 160ms — invisible in either
   file alone, and caught only by measuring. `VisualSystem.test.ts` now holds this.

Rejected ambient effects: perpetual orbit, parallax, pointer-following fields, cursor trails,
sound, haptics, spring spectacle, whole-page glitch, animated artwork, and decorative loaders.
`prefers-reduced-motion: reduce` skips native view transitions, removes smooth scrolling and
decorative displacement, and makes masks and transitions immediate. Suppression is global, so an
effect added later is inert without being listed; any required surviving effect must opt out
explicitly.

## Interaction and accessibility

Appearance may be strange; interaction stays conventional. Use landmarks, logical headings,
real buttons and links, visible focus, accessible names, reading-order focus, and live
regions for loading/failure state. Reviewer and Agent always have visible role labels.
Target WCAG AA. Verify keyboard traversal, reduced motion, network-offline assets, desktop,
and 390px against embedded Rust output.

## Implementation boundary

Reuse shadcn-vue behavior from `web/src/components/ui`; adapt variants through central
observatory tokens instead of cloning interaction logic. Vue Router owns implemented SPA
routes. API modules preserve failures. `web/dist/` is generated, gitignored, and embedded by
Rust, so build it before Cargo.

`ScopeView` owns route fetches, exact query selection, active thread, mutations, SSE,
generation guards, and dirty-composer artifact deferral. `ArtifactNavigator` emits exact
paths only. `ArtifactDocument` owns rendered blocks and source-accurate anchor construction.
`ArtifactConversation` owns selected-artifact thread presentation without fetching.
`DecisionInstrument` owns loose/lost comments and verdict actions. No global store, nested
primary document scroller, duplicate session/change workbench, or compatibility layout.
