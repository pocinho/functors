# Functors Rendering Architecture

This document defines the implementation path from the current WGPU prototype to
Vello-based rendering. It turns the proposals in `docs/notes/vello_1.md` through
`docs/notes/vello_3.md` into repository guidance that can be implemented incrementally.

The target is a custom-rendered Functors surface: editor geometry, text,
diagnostics, panels, notebook cells, overlays, and eventually application
chrome are composed into a Vello scene and presented through WGPU.

## Decision Summary

Use the following ownership boundary:

```text
winit -> input adapter -> semantic Message -> pure update -> Model
                                                   |
                                                   v
                                      backend-neutral view/layout
                                                   |
                                                   v
                                           FrameDescription
                                                   |
                                                   v
                                      Vello scene builder
                                                   |
                                                   v
                                             WGPU surface
```

The important boundary is `FrameDescription`. It is the inspectable contract
between the pure view and the GPU renderer. `Model`, `mvu`, and `view` must not
contain `vello`, `wgpu`, window handles, font objects, glyph caches, or other
renderer resources.

Vello is the vector composition and GPU rendering engine. It is not, by itself,
the complete widget toolkit, accessibility layer, HTML runtime, focus system,
or window-management implementation. Those concerns remain explicit Functors
boundaries.

## Current State

Phase 1 currently has these responsibilities:

- `main.rs` owns the winit application lifecycle and dispatches redraws.
- `input.rs` maps platform events to semantic messages.
- `mvu.rs` performs pure model transitions.
- `model.rs` owns document and editor invariants.
- `view.rs` builds deterministic `FrameDescription` geometry.
- `renderer.rs` owns the WGPU surface, geometry pipeline, and transitional text
  resources.

The current renderer draws colored rectangles and text-derived pixel quads.
The first Vello implementation should preserve the existing view contract and
replace only the renderer-side composition path. Do not redesign the document,
MVU loop, or input routing as part of the renderer migration.

## Target Runtime

A frame follows this sequence:

1. A platform event becomes a semantic message.
2. The pure update function returns the next model and commands.
3. The pure view computes layout and backend-neutral frame data.
4. The renderer creates or resets a Vello scene for the frame.
5. A scene builder emits fills, strokes, clips, paths, and text from the frame.
6. Vello encodes and renders the scene using the WGPU device and queue.
7. The renderer presents the acquired surface texture.

The `vello::Scene` is a per-frame render description, not application state.
The long-lived renderer owns the Vello renderer, WGPU resources, surface
configuration, and renderer-side caches.

The exact Vello API, render-target path, supported texture formats, required
WGPU features, and antialiasing configuration must be confirmed against one
pinned crate version before the implementation is treated as stable. Examples
in design notes are architectural pseudocode, not an API compatibility promise.

## Scene Contract

The existing `FrameDescription` should remain the first scene contract. Its
fields represent semantic visual layers and geometry, for example:

- background and menu bands;
- gutter, line numbers, and editor text runs;
- selections and cursors;
- scrollbars;
- command and settings panels;
- overlay text.

The scene builder owns the translation from these values to Vello primitives.
Its draw order must be explicit and tested:

```text
background
  -> menu and panels
  -> gutter and scrollbars
  -> selection highlights
  -> line numbers and text
  -> cursors and overlays
```

The scene builder must not infer missing state by inspecting `Model`. If a
visual layer needs new state, add a backend-neutral field or semantic record to
the view contract first.

Once the current frame is rendering through Vello, the contract can be evolved
toward named scene descriptions such as `AppScene`, `EditorScene`, and
`OverlayScene`. That extraction is justified when the corresponding ownership
boundary has real behavior and tests; it is not required merely to mirror the
future UI tree.

## Text Architecture

Text is the highest-risk part of this migration. Vello provides scene
composition and GPU rendering, but Functors must verify which shaping, font
loading, glyph rasterization, color-font, and fallback facilities are supplied
by the selected Vello release and which must be provided separately.

Use three distinct layers:

1. **Text layout** computes shaped runs, advances, baselines, line breaks, caret
   positions, selection rectangles, and clipping geometry. It is testable
   without a GPU.
2. **Font and shaping services** load platform fonts, select fallback faces,
   shape Unicode text, and expose glyph identifiers and metrics. They are
   renderer-adjacent resources, not model state.
3. **Vello text emission** converts shaped runs into the selected Vello glyph
   or scene representation and uses renderer-owned caches.

Do not retain the current fixed-width assumption for production text. It is
acceptable as a temporary migration baseline, but mixed scripts, emoji, tabs,
and fallback fonts require shaped advances. Cursor and selection geometry must
come from the same layout result that positions glyphs.

The initial text spike must answer these questions with the pinned dependency
set:

- Which crate performs shaping and fallback on Windows and other targets?
- How are glyphs and color fonts represented in the Vello scene?
- Can the selected path render the required font formats and emoji behavior?
- How are glyph and atlas caches invalidated after resize, scale, or theme
  changes?
- Which surface format and alpha convention does the text path require?

Keep the existing transitional text path until these questions have executable
answers and visual tests. Remove Skia and fontdue only after the Vello path
passes the acceptance criteria below.

## Renderer Responsibilities

`renderer.rs` should evolve in small steps:

- retain WGPU instance, adapter, device, queue, surface, and configuration;
- create and retain the Vello renderer after device initialization;
- acquire and present the surface texture as it does today;
- build a fresh scene for each frame from `FrameDescription`;
- translate colors and rectangles through small renderer-local helpers;
- handle resize, surface loss, zero-sized windows, and render errors;
- keep font, shaping, glyph, and atlas caches out of the pure view;
- report recoverable surface failures through the existing render status path.

The renderer should not introduce a second independent layout system. It may
perform backend conversion, clipping, coordinate transforms, and resource
caching, but positions and semantic draw order should come from the view.

## Migration Plan

### 1. Establish the dependency spike

- Select and pin a Vello version compatible with the repository's WGPU and
  Rust versions.
- Add the smallest Vello dependency set needed for a device-backed renderer.
- Build a renderer-only experiment that clears a surface and draws one solid
  rectangle.
- Record required WGPU features, supported texture formats, and known platform
  limitations.

This spike is successful only if `cargo check` succeeds and a Windows
`cargo run` smoke test presents a nonblank frame.

### 2. Replace solid geometry

- Add a renderer-local `build_scene(&FrameDescription, &mut Scene)` function.
- Translate the background, menu bar, gutter, panels, scrollbars, selections,
  and cursor into Vello fills and paths.
- Keep `FrameDescription` unchanged unless a missing semantic layer is found.
- Delete the temporary WGPU vertex pipeline only after equivalent visual output
  is confirmed.

Add headless scene-input tests where possible. The scene itself is a GPU-facing
value and should not become the unit under test for model behavior.

### 3. Prove text separately

- Define a small backend-neutral shaped-text result containing glyph IDs,
  positions, advances, font identity, and bounds.
- Implement one visible line through the chosen shaping and Vello emission path.
- Test ASCII, combining marks, CJK or another fallback script, emoji, tabs,
  clipping, and fractional positioning.
- Compare glyph positions with cursor and selection rectangles from layout.
- Measure frame time and cache behavior before scaling to the entire document.

Do not make full-app theming or custom chrome a prerequisite for this step.

### 4. Add explicit visual layers

After geometry and text are stable, expand the backend-neutral contract for:

- diagnostics and source ranges;
- tabs, sidebars, status bars, and command surfaces;
- notebook cells and output surfaces;
- overlays and animation state.

Each layer needs deterministic layout tests and a documented draw order before
being added to the scene builder.

### 5. Decide custom chrome independently

Frameless windows are a product and platform decision, not an automatic result
of adopting Vello. If chosen, implement platform-specific drag, resize,
minimize, maximize, restore, system-menu, and accessibility behavior behind an
explicit boundary. The scene can draw the visual chrome, but winit and the
platform still own window operations.

## Testing and Acceptance

Every renderer change should run:

```text
cargo fmt --check
cargo check
cargo test
cargo run
```

The Vello migration is ready to replace the transitional renderer when:

- Windows presents a nonblank Vello frame through the existing winit surface;
- resize, zero-size windows, surface loss, and redraw requests remain correct;
- geometry matches the existing headless `FrameDescription` expectations;
- text remains sharp at the supported DPI scales;
- fallback fonts and emoji render without corrupting cursor or selection
  placement;
- clipping and scrolling do not draw outside their intended layers;
- repeated frames reuse renderer caches and stay within an agreed frame-time
  budget;
- no Vello, WGPU, font, or window types have leaked into model, MVU, or view;
- the old Skia and fontdue paths can be removed without reducing required
  behavior.

Visual validation should include screenshots at 100%, 150%, and 200% Windows
scales, a mixed ASCII/emoji/fallback-font document, a long scrolling document,
and an open command/settings overlay. Headless tests remain the authority for
model transitions and layout invariants.

## Non-Goals for This Migration

- building a complete widget toolkit inside the renderer;
- embedding HTML or a browser runtime in the Vello scene;
- moving document or theme resources into GPU-owned types;
- implementing notebooks before their semantic and output-surface contracts
  exist;
- replacing winit or WGPU;
- making the first Vello renderer support every future animation or canvas
  feature.

## References

- [Vello proposal 1](notes/vello_1.md)
- [Vello proposal 2](notes/vello_2.md)
- [Vello architecture proposal 3](notes/vello_3.md)
- [Vello rendering ADR](decisions/0006-vello-rendering-and-composition.md)
- [High-level architecture](architecture.md)
- [Current blockers](current_blockers.md)
