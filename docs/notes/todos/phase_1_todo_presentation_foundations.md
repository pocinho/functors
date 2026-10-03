# Phase 1 TODO: Presentation Foundations

This note turns the roadmap's Phase 1 presentation foundations into an
executable sequence. It covers the Functors-owned text and widget boundaries
that must exist before the Vello renderer becomes the production path.

The current flat crate remains the implementation home. Start with internal
modules and extract `functors-text` or `functors-widgets` crates only when the
extraction triggers in `docs/module_ownership.md` are met.

## Definition of Done

The presentation-foundations stage is complete when:

- shaped text, cursor geometry, selection geometry, and hit testing are
  represented by backend-neutral types;
- the same text layout result drives both glyph positions and editor geometry;
- font loading and shaping resources are isolated from Model, MVU, and view
  state;
- widget identity, bounds, hit testing, focus order, and presentation states
  have deterministic contracts;
- menus, command bars, scrollbars, and panels use the widget interaction model;
- scrollbar thumb dragging and complete menu command behavior work through
  semantic messages rather than ad hoc event branches;
- theme tokens and paint descriptions do not contain Vello or WGPU types;
- model, text, widget, and layout behavior has headless test coverage;
- the Vello renderer can consume the resulting text and widget descriptions
  without adding a second layout or state system.

Notebook cells, HTML output, agents, plugins, and frameless custom chrome remain
outside this checklist.

## Ownership Contract

```text
winit -> input -> semantic Message -> update -> Model
                                      |
                                      v
                         text/widget-aware view and layout
                                      |
                                      v
                              FrameDescription
                                      |
                                      v
                                Vello renderer
```

`functors-text` owns text metrics and interaction geometry. `functors-widgets`
owns semantic controls and presentation state. The Model remains authoritative
for document edits, workspace state, commands, and diagnostics. The renderer
owns only backend resources and scene emission.

Neither presentation layer may depend on WGPU, Vello, window handles,
filesystem APIs, clocks, or platform event types.

## Ordered Work

### 1. Freeze the boundary before implementation

- [ ] List the current `FrameDescription` text, cursor, selection, panel,
      scrollbar, and overlay fields that remain valid.
- [ ] Identify fields whose fixed-width assumptions prevent shaped text.
- [ ] Define the coordinate units for source positions, glyph positions,
      device-independent layout, and physical pixels.
- [ ] Define semantic messages for widget activation, focus, scroll, drag,
      command submission, and menu selection.
- [ ] Record which state is persistent Model state and which is presentation-only
      widget state.

**Checkpoint:** a design review can trace one input from a platform event to a
semantic message, model or widget transition, frame geometry, and renderer
primitive without crossing an ownership boundary.

### 2. Build the `functors-text` foundation

- [ ] Add the smallest internal text module and backend-neutral public types
      for `FontId`, `GlyphRun`, `GlyphPosition`, `TextMetrics`, and bounds.
- [ ] Define source-position mapping with documented byte/scalar/grapheme
      semantics; do not silently mix coordinate units.
- [ ] Evaluate Parley, Fontique, and Swash against the pinned Rust/toolchain
      and WGPU/Vello plan.
- [ ] Add a font resource boundary for platform discovery and fallback chains.
- [ ] Keep font bytes, font databases, and shaping caches out of Model and view.
- [ ] Implement one shaped single-line layout without changing the renderer.

**Tests:** ASCII, tabs, combining marks, emoji, a fallback script, empty text,
multiline text, fractional positions, and out-of-range cursor positions.

**Checkpoint:** the text module returns deterministic glyph runs, advances,
baselines, cursor positions, selection rectangles, and hit-test results for the
same input.

### 3. Connect text geometry to the editor contract

- [ ] Replace fixed `advance` assumptions at the text-layout boundary while
      keeping the current view API stable where possible.
- [ ] Make cursor and selection rectangles derive from the shaped layout result.
- [ ] Add IME preedit geometry and composition styling inputs.
- [ ] Define clipping and viewport behavior for visual lines.
- [ ] Preserve syntax highlighting as backend-neutral style spans.
- [ ] Add regression tests for reversed, collapsed, multiline, Unicode, and
      mixed-font selections.

**Checkpoint:** changing a glyph advance changes text, cursor, selection, and
hit-testing geometry consistently.

### 4. Build the `functors-widgets` foundation

- [ ] Add stable widget identity and a minimal compositional widget tree or
      equivalent description type.
- [ ] Define layout bounds, invalidation, z-order, clipping, and paint order.
- [ ] Define semantic states: hovered, pressed, focused, disabled, selected,
      and error.
- [ ] Define focus navigation and pointer capture without platform types.
- [ ] Define theme tokens for typography, spacing, colors, borders, radii, and
      emphasis.
- [ ] Define accessibility roles, names, states, and navigation metadata.
- [ ] Add backend-neutral paint descriptions for panels, menus, command bars,
      scrollbars, and status surfaces.

**Tests:** stable identity, layout bounds, z-order, hit testing at edges,
focus traversal, disabled controls, pointer capture, and theme changes.

**Checkpoint:** a widget interaction can be tested from semantic input to state
transition and paint description without starting a window or GPU.

### 5. Move existing interactions into widgets

- [ ] Route menu hit testing through widget bounds and semantic menu messages.
- [ ] Implement complete menu command behavior through widget actions.
- [ ] Implement scrollbar thumb dragging through pointer capture, clamped
      viewport updates, and redraw commands.
- [ ] Preserve horizontal and vertical scroll semantics for wheel and drag input.
- [ ] Add command-bar focus, submission, cancellation, and keyboard navigation.
- [ ] Keep filesystem and command effects at the existing application boundary.

**Checkpoint:** menu activation, command submission, scrollbar dragging, and
focus changes have headless tests and no renderer-specific event branches.

### 6. Extend `FrameDescription` deliberately

- [ ] Add named or grouped visual layers only where text/widget contracts need
      them.
- [ ] Document the draw order for background, panels, gutters, selections,
      text, controls, cursors, diagnostics, and overlays.
- [ ] Add scene-input tests for layer order, clipping, resize, and empty states.
- [ ] Ensure `view` remains deterministic and does not inspect renderer state.
- [ ] Confirm no Vello, WGPU, font, or window types leak into the contract.

**Checkpoint:** the current renderer can still consume the frame description,
even if the new Vello scene builder is not yet enabled.

### 7. Handoff to Vello

- [ ] Pin compatible Vello and WGPU versions and record required GPU features.
- [ ] Translate widget paint descriptions into Vello scene primitives.
- [ ] Emit `functors-text` glyph runs through the validated Vello text path.
- [ ] Compare Vello output with deterministic frame-input expectations.
- [ ] Validate resize, surface recovery, clipping, DPI scaling, cache reuse, and
      frame time on the primary Windows target.
- [ ] Remove Skia/fontdue only after the Vello acceptance criteria pass.

## Validation Commands

For module, layout, or dependency changes:

```text
cargo fmt --check
cargo check
cargo test
```

For rendering or visible interaction changes, also run:

```text
cargo run
```

The desktop smoke test should cover startup, resize, minimize/restore, text
input, IME input where available, cursor movement, selection, menu commands,
scrollbar dragging, command-bar focus, and clean shutdown.

## Reference Material

- [Roadmap](../../roadmap.md)
- [Module ownership](../../module_ownership.md)
- [Functors text and widget ADR](../../decisions/0007-functors-text-and-widgets.md)
- [Rendering architecture](../../rendering_architecture.md)
- [Testing strategy](../../testing.md)
- [Floem evaluation](../using_floem.md)
