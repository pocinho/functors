# 0007: Create Functors-Owned Text and Widget Layers

- Status: Accepted
- Date: 2026-10-03
- Related: 0006

## Context

Floem demonstrates a useful combination of shaped text, editor-oriented
geometry, widgets, layout, themes, focus, and multiple rendering backends. Its
implementation is valuable reference material for Functors, especially its
Parley-based text layout and Vello glyph emission.

Adopting Floem wholesale would also introduce a second state architecture.
Floem's retained reactive view tree and signals would overlap with Functors'
pure MVU transitions, document invariants, and backend-neutral
`FrameDescription`. A direct adoption could duplicate editor state and make
layout, focus, and editing behavior harder to replay and test.

Functor therefore needs its own text and widget ownership boundaries. They can
learn from Floem without making Functors depend on Floem's reactive runtime or
adopting its full application framework.

## Decision

Create two Functors-owned layers:

- `functors-text` owns shaped text layout and text interaction geometry.
- `functors-widgets` owns semantic controls, widget layout, focus, and widget
  presentation descriptions.

During the current flat Phase 1 implementation, these should begin as small
internal modules, such as `src/text.rs` and `src/widgets.rs`, or private module
folders. Extract separate crates only when the ownership and API boundaries have
grown enough to meet the repository's extraction triggers.

The dependency direction is:

```text
Model/MVU -> functors-text + functors-widgets -> FrameDescription -> Vello -> WGPU
```

Neither layer may depend on WGPU, Vello, window handles, filesystem APIs,
clocks, or platform event types. Backend-specific conversion belongs in the
renderer adapter. Floem remains a reference implementation and optional proof-
of-concept integration, not a required runtime dependency.

## `functors-text` Responsibilities

`functors-text` must provide backend-neutral, testable text behavior:

- font family and fallback-chain descriptions;
- font loading and platform font discovery behind an explicit resource
  boundary;
- Unicode shaping and glyph runs using a proven shaping stack;
- glyph identifiers, advances, offsets, clusters, baselines, and bounds;
- line breaking, wrapping, tab expansion, and visual-line mapping;
- byte or scalar position mapping with documented coordinate semantics;
- cursor hit testing and caret rectangles;
- forward and reversed selection geometry;
- composition/IME preedit geometry;
- diagnostics ranges, underlines, and marker geometry;
- scale-aware metrics and cache invalidation inputs;
- deterministic tests for ASCII, combining marks, fallback scripts, emoji,
  tabs, multiline text, clipping, and fractional positions.

The text layer must produce the geometry used by both cursor/selection logic and
glyph emission. The renderer must not independently recompute advances or hit
testing from raw strings.

The first implementation should evaluate proven components such as Parley,
Fontique, and Swash, using Floem's text implementation as a reference. Do not
hand-roll Unicode shaping, font fallback, or glyph rasterization.

## `functors-widgets` Responsibilities

`functors-widgets` must provide backend-neutral semantic controls and layout:

- stable widget identity and tree composition;
- layout constraints, measured bounds, and invalidation;
- panels, stacks, menus, command bars, dialogs, tabs, scrollbars, and status
  surfaces;
- pointer hit testing and keyboard focus navigation;
- semantic widget messages such as activate, focus, change, submit, and scroll;
- pressed, hovered, focused, disabled, selected, and error states;
- theme tokens for colors, typography, spacing, borders, radii, and emphasis;
- accessibility roles, names, states, and navigation metadata;
- clipping, z-order, overlays, and deterministic paint descriptions;
- headless tests for layout, hit testing, focus order, and state transitions.

Widget state must distinguish persistent application state from presentation
state. Document edits, workspace state, commands, and diagnostics remain in the
Functors model and MVU loop. Hover, focus, pointer capture, measurement, and
animation progress may be widget-local state when they do not alter domain
semantics.

The widget layer should emit semantic geometry or extend `FrameDescription`.
It must not call Vello or inspect a WGPU surface. This keeps the widget system
usable with a future renderer, a software test renderer, or a Floem adapter.

## Sequencing

Implement the layers in this order:

1. **Text contract:** define shaped runs, cursor positions, selection geometry,
   and font/resource boundaries without changing the renderer.
2. **Text proof:** validate one shaped editor line through the Vello path with
   fallback text, emoji, IME preedit, clipping, and fractional positioning.
3. **Widget primitives:** extract panel, menu, command bar, scrollbar, focus,
   and hit-testing behavior from the current frame geometry.
4. **Widget state:** add semantic control states, theme tokens, and headless
   interaction tests.
5. **Renderer integration:** translate text and widget descriptions into Vello
   scenes while preserving `FrameDescription` as the inspectable boundary.
6. **Crate extraction:** create `functors-text` and `functors-widgets` crates
   only after the modules have stable public facades, independent tests, and
   different dependency needs.

Do not begin notebook widgets, HTML output widgets, or custom frameless chrome
until the primitive focus, layout, accessibility, and output-surface boundaries
are explicit.

## Alternatives Considered

### Adopt Floem as the complete UI framework

Rejected as the default direction for now. Floem provides substantial value,
but its reactive view-tree ownership would overlap with Functors' MVU model and
would create a large pre-1.0 dependency commitment. It remains appropriate for
a bounded proof of concept and a source of implementation patterns.

### Keep all UI behavior in `view.rs` and `renderer.rs`

Rejected for the medium term. This preserves the current prototype's small
surface but would continue mixing layout, hit testing, focus, text metrics, and
paint conversion. The result would make Unicode text and widget behavior harder
to test and extend.

### Fork or copy Floem's editor and widget implementations

Rejected. Copying code would create synchronization and licensing/maintenance
costs without preserving the architectural reasons Functors is choosing its
own state boundaries. Use compatible upstream libraries and document borrowed
patterns instead.

### Create separate crates immediately

Deferred. The Phase 1 crate is intentionally flat, and premature crate
boundaries would add API and build overhead before the ownership boundaries are
proven. Start with modules and extract when the repository's normal triggers
are met.

## Consequences

- Functors retains one authoritative MVU/domain state path.
- Text quality, cursor alignment, and widget behavior become independently
  testable without a GPU.
- Vello remains a renderer implementation rather than leaking through the
  text or widget APIs.
- Floem can be evaluated through adapters without becoming foundational
  infrastructure.
- Functors assumes responsibility for difficult text and accessibility
  contracts instead of receiving them automatically from a UI framework.
- The implementation will initially duplicate some infrastructure that Floem
  already provides, so reuse of proven libraries and focused scope are required.

## Review Triggers

Revisit this decision if:

- the text layer cannot meet shaping, fallback, emoji, IME, or performance
  requirements on supported platforms;
- widget accessibility or focus behavior requires a framework-level solution;
- a Floem proof of concept preserves MVU ownership with materially lower cost;
- the layers acquire stable public APIs and distinct dependency graphs, making
  crate extraction beneficial; or
- notebook and HTML output requirements require a different presentation
  boundary.
